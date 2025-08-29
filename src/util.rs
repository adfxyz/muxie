use std::ffi::OsStr;
use std::path::PathBuf;

/// Resolve `cmd` in the given PATH, or the process PATH if `path` is None.
pub(crate) fn which_in_path(cmd: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        let p = PathBuf::from(cmd);
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
        return None;
    }
    use std::borrow::Cow;
    let cow: Cow<OsStr> = if let Some(p) = path {
        Cow::Borrowed(p)
    } else if let Some(envp) = std::env::var_os("PATH") {
        Cow::Owned(envp)
    } else {
        return None;
    };
    for dir in std::env::split_paths(&cow) {
        let candidate = dir.join(cmd);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

pub(crate) fn is_executable(p: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .ok()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
