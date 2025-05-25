use std::path::{Path, PathBuf};

pub fn resolve_symlink(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or(path.to_path_buf())
}
