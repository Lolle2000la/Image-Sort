use std::path::{Path, PathBuf};

use thiserror::Error;

/// A reversible mutation on the file system.
pub trait ReversibleAction: Send + Sync {
    fn display_name(&self, l10n: &crate::l10n::Localization) -> String;
    fn execute(&mut self) -> Result<(), ActionError>;
    fn rollback(&mut self) -> Result<(), ActionError>;
}

/// Rejects sources that are symbolic links before any action is created.
///
/// `canonicalize()` on the source resolves the link, so an action would
/// silently operate on the link TARGET instead of the entry the user sees:
/// "moving" a symlinked photo relocates the victim file and leaves the link
/// behind; copying it exfiltrates the target's contents. The in-app scanner
/// skips symlinks, but drag & drop and external paths do not.
pub fn reject_symlink_source(path: &Path) -> Result<(), ActionError> {
    if crate::path_utils::is_symlink(path) {
        Err(ActionError::SourceIsSymlink(path.to_path_buf()))
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("source file not found: {0}")]
    SourceNotFound(PathBuf),

    #[error("target already exists: {0}")]
    TargetExists(PathBuf),

    #[error("illegal character {character:?} in filename stem \"{stem}\"")]
    IllegalCharacters { stem: String, character: char },

    #[error("directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("source is a symbolic link: {0}")]
    SourceIsSymlink(PathBuf),

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("restoration not possible: {0}")]
    RestorationFailed(String),
}
