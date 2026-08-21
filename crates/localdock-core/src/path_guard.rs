use crate::LocalDockError;
use std::path::{Path, PathBuf};

/// Strip Windows verbatim `\\?\` / `\\?\UNC\` prefixes so path prefix checks behave consistently.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path.to_path_buf()
}

pub fn display_path(path: &Path) -> String {
    strip_verbatim_prefix(path).display().to_string()
}

pub fn assert_under_roots(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, LocalDockError> {
    let canon = std::fs::canonicalize(path).map_err(LocalDockError::Io)?;
    let canon_norm = strip_verbatim_prefix(&canon);
    for root in roots {
        let root_canon = std::fs::canonicalize(root).map_err(LocalDockError::Io)?;
        let root_norm = strip_verbatim_prefix(&root_canon);
        if canon_norm.starts_with(&root_norm) {
            return Ok(canon);
        }
    }
    Err(LocalDockError::PathNotAllowed(canon.display().to_string()))
}
