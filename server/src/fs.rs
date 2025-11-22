use crate::error::AppError;
use std::path::{Path, PathBuf};

pub struct FsService {
    root: PathBuf,
}

impl FsService {
    pub fn new(root: impl AsRef<Path>) -> std::io::Result<Self> {
        let canonical_root = std::fs::canonicalize(root)?;
        Ok(Self {
            root: canonical_root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a user-provided relative path against the configured root, ensuring it cannot escape the sandbox.
    pub fn resolve_path(&self, relative: &str) -> Result<PathBuf, AppError> {
        let mut candidate = self.root.clone();
        if !relative.is_empty() {
            candidate.push(relative);
        }

        let canonical = candidate.canonicalize()?;
        if !canonical.starts_with(&self.root) {
            return Err(AppError::BadRequest("path escapes root_dir".into()));
        }

        Ok(canonical)
    }

    pub fn to_relative(&self, absolute: &Path) -> Option<String> {
        absolute
            .strip_prefix(&self.root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }
}
