use std::path::PathBuf;

pub(crate) fn which_in_path(cmd: &str) -> Option<PathBuf> {
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        let p = PathBuf::from(cmd);
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
        return None;
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
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
