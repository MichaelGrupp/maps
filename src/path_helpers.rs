use std::path::PathBuf;

pub fn resolve_symlink(path: &PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path.clone())
}
