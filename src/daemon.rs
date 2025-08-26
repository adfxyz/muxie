use crate::config::{Config, read_config};
use anyhow::{Context, Result};
use std::sync::Mutex;

pub const DBUS_SERVICE: &str = "xyz.adf.Muxie";
pub const DBUS_INTERFACE: &str = "xyz.adf.Muxie1"; // Note: must match the dbus_interface attribute
pub const DBUS_PATH: &str = "/xyz/adf/Muxie";
pub const DBUS_METHOD_OPEN_URL: &str = "OpenUrl";
pub const DBUS_METHOD_RELOAD: &str = "ReloadConfig";

struct MuxieDaemon {
    cfg: Mutex<Config>,
    no_notify: bool,
    verbose: u8,
}

impl MuxieDaemon {
    fn new(cfg: Config, no_notify: bool, verbose: u8) -> Self {
        Self {
            cfg: Mutex::new(cfg),
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
    let daemon = MuxieDaemon::new(cfg, no_notify, verbose);

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

    // Block the main thread; zbus runs the object server internally.
    // We keep the process alive until killed by the user.
    loop {
        std::thread::park();
    }
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
        let d = MuxieDaemon::new(cfg_empty(), false, 0);
        let res = d.OpenUrl("   ");
        assert!(res.is_err());
    }

    #[test]
    fn open_url_propagates_error_on_invalid_cfg() {
        // With empty browsers, open_url_with returns an error; ensure it's mapped to fdo::Failed
        let d = MuxieDaemon::new(cfg_empty(), false, 0);
        let res = d.OpenUrl("https://example.com");
        assert!(res.is_err());
    }
}
