use std::path::{Path, PathBuf};

pub(crate) fn resolve_symlink(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or(path.to_path_buf())
}
