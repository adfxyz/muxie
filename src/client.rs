use crate::daemon::{DBUS_INTERFACE, DBUS_METHOD_OPEN_URL, DBUS_PATH, DBUS_SERVICE};
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
}

impl MuxieClient for ZbusClient {
    fn open_url(&self, url: &str) -> Result<()> {
        use zbus::blocking::Proxy;
        let proxy = Proxy::new(&self.conn, DBUS_SERVICE, DBUS_PATH, DBUS_INTERFACE)
            .context("Failed to create daemon proxy")?;

        // Call OpenUrl method; ignore return (unit)
        proxy
            .call_method(DBUS_METHOD_OPEN_URL, &(url))
            .context("Failed to call OpenUrl on daemon")?;
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
