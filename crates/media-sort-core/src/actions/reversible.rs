use std::path::PathBuf;

use thiserror::Error;

/// A reversible mutation on the file system.
pub trait ReversibleAction: Send + Sync {
    fn display_name(&self) -> &str;
    fn execute(&mut self) -> Result<(), ActionError>;
    fn rollback(&mut self) -> Result<(), ActionError>;
}

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("source file not found: {0}")]
    SourceNotFound(PathBuf),

    #[error("target already exists: {0}")]
    TargetExists(PathBuf),

    #[error("directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("restoration not possible: {0}")]
    RestorationFailed(String),
}
