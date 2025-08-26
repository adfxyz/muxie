use crate::config::{Config, read_config};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const DBUS_SERVICE: &str = "xyz.adf.Muxie";
pub const DBUS_INTERFACE: &str = "xyz.adf.Muxie1"; // Note: must match the dbus_interface attribute
pub const DBUS_PATH: &str = "/xyz/adf/Muxie";
pub const DBUS_METHOD_OPEN_URL: &str = "OpenUrl";
pub const DBUS_METHOD_RELOAD: &str = "ReloadConfig";

struct MuxieDaemon {
    cfg: Arc<Mutex<Config>>,
    no_notify: bool,
    verbose: u8,
}

impl MuxieDaemon {
    fn new(cfg: Arc<Mutex<Config>>, no_notify: bool, verbose: u8) -> Self {
        Self {
            cfg,
            no_notify,
            verbose,
        }
    }
}

#[zbus::dbus_interface(name = "xyz.adf.Muxie1")]
impl MuxieDaemon {
    #[allow(non_snake_case)]
    fn OpenUrl(&self, url: &str) -> zbus::fdo::Result<()> {
        let u = url.trim();
        if u.is_empty() {
            return Err(zbus::fdo::Error::Failed("empty URL".to_string()));
        }
        if self.verbose >= 1 {
            eprintln!("[daemon] Received OpenUrl: {u}");
        }
        let opener = crate::open::DefaultOpener;
        let notifier = crate::notify::DefaultNotifier;
        let cfg_guard = self.cfg.lock().unwrap();
        match crate::open::open_url_with(
            &cfg_guard,
            &opener,
            &notifier,
            u,
            self.no_notify,
            self.verbose,
        ) {
            Ok(()) => {
                if self.verbose >= 1 {
                    eprintln!("[daemon] Processed OpenUrl successfully");
                }
                Ok(())
            }
            Err(e) => {
                if self.verbose >= 1 {
                    eprintln!("[daemon] OpenUrl failed: {e}");
                }
                Err(zbus::fdo::Error::Failed(format!("{e}")))
            }
        }
    }

    #[allow(non_snake_case)]
    fn ReloadConfig(&self) -> zbus::fdo::Result<bool> {
        if self.verbose >= 1 {
            eprintln!("[daemon] Reloading configuration...");
        }
        match read_config() {
            Ok(new_cfg) => {
                let mut guard = self.cfg.lock().unwrap();
                *guard = new_cfg;
                if self.verbose >= 1 {
                    eprintln!("[daemon] Reloaded configuration successfully");
                }
                Ok(true)
            }
            Err(e) => {
                if self.verbose >= 1 {
                    eprintln!("[daemon] Reload failed: {e}");
                }
                Ok(false)
            }
        }
    }
}

/// Run the Muxie daemon
pub fn run(no_notify: bool, verbose: u8) -> Result<()> {
    let cfg = read_config().context("Failed to read configuration at startup")?;
    let cfg_arc = Arc::new(Mutex::new(cfg));
    let daemon = MuxieDaemon::new(cfg_arc.clone(), no_notify, verbose);

    // Build a blocking zbus connection, own the well-known name, and export the object
    let _conn = zbus::blocking::ConnectionBuilder::session()
        .context("Failed to connect to session D-Bus")?
        .name(DBUS_SERVICE)
        .context(format!("Failed to own D-Bus name {DBUS_SERVICE}"))?
        .serve_at(DBUS_PATH, daemon)
        .context(format!("Failed to export daemon object at {DBUS_PATH}"))?
        .build()
        .context("Failed to start D-Bus object server")?;

    if verbose >= 1 {
        eprintln!(
            "[daemon] Started. Service={DBUS_SERVICE}, Object={DBUS_PATH}, Interface={DBUS_INTERFACE}"
        );
    }

    // Start auto-reload watcher in the background
    if let Err(e) = start_config_watcher(cfg_arc, verbose) {
        eprintln!("[daemon] Warning: failed to start config watcher: {e}");
    }

    // Block the main thread; zbus runs the object server internally.
    // We keep the process alive until killed by the user.
    loop {
        std::thread::park();
    }
}

fn start_config_watcher(cfg: Arc<Mutex<Config>>, verbose: u8) -> Result<()> {
    let cfg_path = crate::paths::config_path();
    let parent: PathBuf = cfg_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let (tx, rx) = std::sync::mpsc::channel::<Result<::notify::Event, ::notify::Error>>();
    let mut watcher = ::notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    use ::notify::{RecursiveMode, Watcher};
    watcher.watch(&parent, RecursiveMode::NonRecursive)?;

    if verbose >= 1 {
        eprintln!("[daemon] Watching config directory: {}", parent.display());
    }

    std::thread::spawn(move || {
        let debounce = Duration::from_millis(400);
        let max_interval = Duration::from_secs(2);
        let target_name = cfg_path.file_name().map(|s| s.to_owned());

        loop {
            let Ok(res) = rx.recv() else { break };
            let Ok(event) = res else { continue };
            if !event_is_relevant(&event, &cfg_path, target_name.as_deref()) {
                continue;
            }
            let mut last_relevant = Instant::now();
            let start = Instant::now();
            // Coalesce until quiet period or max interval
            loop {
                match rx.recv_timeout(Duration::from_millis(150)) {
                    Ok(Ok(ev)) => {
                        if event_is_relevant(&ev, &cfg_path, target_name.as_deref()) {
                            last_relevant = Instant::now();
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if last_relevant.elapsed() >= debounce || start.elapsed() >= max_interval {
                            break;
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            // Attempt reload (best effort)
            match read_config() {
                Ok(new_cfg) => {
                    if let Ok(mut guard) = cfg.lock() {
                        *guard = new_cfg;
                        if verbose >= 1 {
                            eprintln!("[daemon] Auto-reload: configuration updated");
                        }
                    }
                }
                Err(e) => {
                    if verbose >= 1 {
                        eprintln!("[daemon] Auto-reload failed: {e}");
                    }
                }
            }
        }
    });

    // Keep watcher alive by preventing drop
    std::mem::forget(watcher);
    Ok(())
}

fn event_is_relevant(
    ev: &::notify::Event,
    cfg_path: &std::path::Path,
    target_name: Option<&std::ffi::OsStr>,
) -> bool {
    for p in &ev.paths {
        if p == cfg_path {
            return true;
        }
        if let (Some(tn), Some(pn)) = (target_name, p.file_name()) {
            if pn == tn {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_empty() -> Config {
        Config {
            version: 1,
            browsers: vec![],
            patterns: vec![],
            notifications: crate::config::Notifications::default(),
        }
    }

    #[test]
    fn open_url_rejects_empty() {
        let d = MuxieDaemon::new(Arc::new(Mutex::new(cfg_empty())), false, 0);
        let res = d.OpenUrl("   ");
        assert!(res.is_err());
    }

    #[test]
    fn open_url_propagates_error_on_invalid_cfg() {
        // With empty browsers, open_url_with returns an error; ensure it's mapped to fdo::Failed
        let d = MuxieDaemon::new(Arc::new(Mutex::new(cfg_empty())), false, 0);
        let res = d.OpenUrl("https://example.com");
        assert!(res.is_err());
    }
}
