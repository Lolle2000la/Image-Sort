use std::fs;
use std::path::{Path, PathBuf};

use crate::actions::reversible::{ActionError, ReversibleAction};

pub struct CopyAction {
    source: PathBuf,
    destination: PathBuf,
    executed: bool,
    display_name: String,
}

impl CopyAction {
    pub fn new(file: &Path, to_folder: &Path) -> Result<Self, ActionError> {
        let file = file
            .canonicalize()
            .map_err(|_| ActionError::SourceNotFound(file.to_path_buf()))?;
        let to_folder = to_folder
            .canonicalize()
            .map_err(|_| ActionError::DirectoryNotFound(to_folder.to_path_buf()))?;

        let file_name = file
            .file_name()
            .ok_or_else(|| ActionError::SourceNotFound(file.clone()))?;
        let destination = to_folder.join(file_name);

        if destination.exists() {
            return Err(ActionError::TargetExists(destination));
        }

        let display_name = format!(
            "Copy {} to {}",
            file_name.to_string_lossy(),
            to_folder.display()
        );

        Ok(Self {
            source: file,
            destination,
            executed: false,
            display_name,
        })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn destination(&self) -> &Path {
        &self.destination
    }
}

impl ReversibleAction for CopyAction {
    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        fs::copy(&self.source, &self.destination)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        fs::remove_file(&self.destination)?;
        self.executed = false;
        Ok(())
    }
}
