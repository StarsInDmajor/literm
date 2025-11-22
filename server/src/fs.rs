use crate::error::AppError;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct FsService {
    root: Mutex<PathBuf>,
}

impl FsService {
    pub fn new(root: impl AsRef<Path>) -> std::io::Result<Self> {
        let canonical_root = std::fs::canonicalize(root)?;
        Ok(Self {
            root: Mutex::new(canonical_root),
        })
    }

    pub fn root(&self) -> PathBuf {
        self.root.lock().unwrap().clone()
    }

    pub fn set_root(&self, new_root: impl AsRef<Path>) -> Result<PathBuf, AppError> {
        let canonical_root = std::fs::canonicalize(new_root)?;
        *self.root.lock().unwrap() = canonical_root.clone();
        Ok(canonical_root)
    }

    /// Resolve a user-provided relative path against the configured root, ensuring it cannot escape the sandbox.
    pub fn resolve_path(&self, relative: &str) -> Result<PathBuf, AppError> {
        let root = self.root();
        let mut candidate = root.clone();
        if !relative.is_empty() {
            candidate.push(relative);
        }

        let canonical = candidate.canonicalize()?;
        if !canonical.starts_with(&root) {
            return Err(AppError::BadRequest("path escapes root_dir".into()));
        }

        Ok(canonical)
    }

    pub fn to_relative(&self, absolute: &Path) -> Option<String> {
        let root = self.root();
        absolute
            .strip_prefix(&root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }
}
