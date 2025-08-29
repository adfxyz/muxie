use crate::daemon::{DBUS_INTERFACE, DBUS_METHOD_OPEN_URL_FD, DBUS_PATH, DBUS_SERVICE};
use anyhow::{Context, Result};

/// Client interface to the Muxie daemon.
pub trait MuxieClient {
    fn open_url(&self, url: &str) -> Result<()>;
}

/// zbus-based client implementation.
pub struct ZbusClient {
    conn: zbus::blocking::Connection,
}

impl ZbusClient {
    pub(crate) fn new() -> Result<Self> {
        let conn =
            zbus::blocking::Connection::session().context("Failed to connect to session D-Bus")?;
        Ok(Self { conn })
    }

    /// Check if the daemon service name currently has an owner without activating it.
    pub(crate) fn is_running() -> Result<bool> {
        let conn =
            zbus::blocking::Connection::session().context("Failed to connect to session D-Bus")?;
        let proxy =
            zbus::blocking::fdo::DBusProxy::new(&conn).context("Failed to create DBusProxy")?;
        let name = zbus_names::BusName::try_from(DBUS_SERVICE)
            .context("Invalid service name for D-Bus")?;
        let has = proxy
            .name_has_owner(name)
            .context("Failed to query NameHasOwner")?;
        Ok(has)
    }

    pub(crate) fn reload() -> Result<bool> {
        use zbus::blocking::Proxy;
        let conn =
            zbus::blocking::Connection::session().context("Failed to connect to session D-Bus")?;
        let proxy = Proxy::new(&conn, DBUS_SERVICE, DBUS_PATH, DBUS_INTERFACE)
            .context("Failed to create daemon proxy")?;
        let res: bool = proxy
            .call_method(crate::daemon::DBUS_METHOD_RELOAD, &())
            .and_then(|reply| reply.body().deserialize::<bool>())
            .context("Failed to call ReloadConfig on daemon")?;
        Ok(res)
    }
}

impl MuxieClient for ZbusClient {
    fn open_url(&self, url: &str) -> Result<()> {
        use std::io::Write;
        use std::os::unix::io::{FromRawFd, RawFd};
        use zbus::blocking::Proxy;

        // Create a pipe and write URL to the write end
        let mut fds = [0 as libc::c_int; 2];
        let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        if rc != 0 {
            let os_err = std::io::Error::last_os_error();
            return Err(anyhow::anyhow!("pipe2 failed: {}", os_err));
        }
        let rfd: RawFd = fds[0];
        let wfd: RawFd = fds[1];
        // SAFETY: we immediately wrap raw fd and close when dropped
        let mut wfile = unsafe { std::fs::File::from_raw_fd(wfd) };
        wfile.write_all(url.as_bytes())?;
        // Drop write end to signal EOF to the daemon
        drop(wfile);

        // Wrap read end as OwnedFd for zbus
        // SAFETY: rfd comes from a new pipe we created above
        let std_owned = unsafe { std::os::fd::OwnedFd::from_raw_fd(rfd) };
        let zfd = zbus::zvariant::OwnedFd::from(std_owned);

        let proxy = Proxy::new(&self.conn, DBUS_SERVICE, DBUS_PATH, DBUS_INTERFACE)
            .context("Failed to create daemon proxy")?;
        match proxy.call_method(DBUS_METHOD_OPEN_URL_FD, &(zfd)) {
            Ok(_) => (),
            Err(e) => {
                let es = e.to_string();
                if es.contains(crate::open::CANCELED_ERR_MARKER) {
                    anyhow::bail!(crate::open::CANCELED_ERR_MARKER);
                }
                return Err(anyhow::Error::new(e).context("Failed to call OpenUrlFd on daemon"));
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct OkClient;
    impl MuxieClient for OkClient {
        fn open_url(&self, _url: &str) -> Result<()> {
            Ok(())
        }
    }

    struct ErrClient;
    impl MuxieClient for ErrClient {
        fn open_url(&self, _url: &str) -> Result<()> {
            anyhow::bail!("no daemon")
        }
    }

    // Local helper to test fallback behavior without touching main.rs orchestrations.
    fn attempt_open_via_daemon_or_fallback<C: MuxieClient, F: FnOnce() -> Result<()>>(
        client: &C,
        url: &str,
        fallback: F,
    ) -> Result<()> {
        match client.open_url(url) {
            Ok(()) => Ok(()),
            Err(_) => fallback(),
        }
    }

    #[test]
    fn uses_daemon_when_available() {
        let client = OkClient;
        let mut fallback_called = false;
        let res = attempt_open_via_daemon_or_fallback(&client, "https://example.com", || {
            fallback_called = true;
            Ok(())
        });
        assert!(res.is_ok());
        assert!(!fallback_called);
    }

    #[test]
    fn falls_back_when_daemon_unavailable() {
        let client = ErrClient;
        let mut fallback_called = false;
        let res = attempt_open_via_daemon_or_fallback(&client, "https://example.com", || {
            fallback_called = true;
            Ok(())
        });
        assert!(res.is_ok());
        assert!(fallback_called);
    }
}
