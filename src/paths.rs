use std::path::PathBuf;

#[cfg(feature = "self-install")]
pub(crate) fn dbus_service_dir() -> PathBuf {
    let mut p = dirs::data_dir().expect("Failed to get user data directory");
    p.push("dbus-1");
    p.push("services");
    p
}

#[cfg(feature = "self-install")]
pub(crate) fn dbus_service_path() -> PathBuf {
    let mut p = dbus_service_dir();
    p.push(format!("{}.service", crate::daemon::DBUS_SERVICE));
    p
}

#[cfg(feature = "self-install")]
pub fn desktop_entry_path() -> PathBuf {
    let mut path = dirs::data_dir().expect("Failed to get user data directory");
    path.push("applications");
    path.push("muxie.desktop");
    path
}

#[cfg(feature = "self-install")]
pub fn icon_path(size: &str, name: &str) -> PathBuf {
    {
        let mut p = dirs::data_dir().expect("Failed to get user data directory");
        p.push("icons");
        p.push("hicolor");
        p.push(size);
        p.push("apps");
        p.push(name);
        p
    }
}

#[cfg(feature = "self-install")]
pub fn icon_paths() -> Vec<PathBuf> {
    {
        use crate::asset::Icon;
        let mut out = Vec::new();
        for icon in Icon::iter() {
            if let Some((size, name)) = icon.as_ref().split_once('/') {
                out.push(icon_path(size, name));
            }
        }
        out
    }
}

pub fn config_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().expect("Failed to get user config directory");
    config_dir.push("muxie");
    config_dir.push("muxie.toml");
    config_dir
}

#[cfg(feature = "self-install")]
pub fn state_path() -> PathBuf {
    let mut p = dirs::state_dir().expect("Failed to get user state directory");
    p.push("muxie");
    p.push("state.toml");
    p
}
