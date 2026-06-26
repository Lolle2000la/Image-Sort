use std::path::{Path, PathBuf};

use crate::actions::reversible::{ActionError, ReversibleAction};

pub struct MoveAction {
    old_path: PathBuf,
    new_path: PathBuf,
    executed: bool,
    display_name: String,
}

impl MoveAction {
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
        let new_path = to_folder.join(file_name);
        let display_name = format!(
            "Move {} to {}",
            file_name.to_string_lossy(),
            to_folder.display()
        );

        Ok(Self {
            old_path: file,
            new_path,
            executed: false,
            display_name,
        })
    }

    pub fn old_path(&self) -> &Path {
        &self.old_path
    }

    pub fn new_path(&self) -> &Path {
        &self.new_path
    }
}

impl ReversibleAction for MoveAction {
    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        std::fs::rename(&self.old_path, &self.new_path)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        std::fs::rename(&self.new_path, &self.old_path)?;
        self.executed = false;
        Ok(())
    }
}
