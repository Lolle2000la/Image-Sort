use std::path::{Path, PathBuf};

use crate::actions::reversible::{ActionError, ReversibleAction};

pub struct RenameAction {
    old_path: PathBuf,
    new_path: PathBuf,
    executed: bool,
    display_name: String,
}

impl RenameAction {
    pub fn new(path: &Path, new_stem: &str) -> Result<Self, ActionError> {
        let path = path
            .canonicalize()
            .map_err(|_| ActionError::SourceNotFound(path.to_path_buf()))?;

        if new_stem.contains('\\')
            || new_stem.contains('/')
            || new_stem.contains(':')
            || new_stem.contains('*')
            || new_stem.contains('?')
            || new_stem.contains('"')
            || new_stem.contains('<')
            || new_stem.contains('>')
            || new_stem.contains('|')
        {
            return Err(ActionError::TargetExists(PathBuf::from(new_stem)));
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let ext = path.extension().unwrap_or_default();
        let new_path = if ext.is_empty() {
            parent.join(new_stem)
        } else {
            parent.join(format!("{}.{}", new_stem, ext.to_string_lossy()))
        };

        if new_path.exists() {
            return Err(ActionError::TargetExists(new_path));
        }

        let display_name = format!(
            "Rename {} to {}",
            path.file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default(),
            new_path
                .file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default(),
        );

        Ok(Self {
            old_path: path,
            new_path,
            executed: false,
            display_name,
        })
    }

    pub fn new_path(&self) -> &Path {
        &self.new_path
    }
}

impl ReversibleAction for RenameAction {
    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        crate::path_utils::rename_or_copy_and_delete(&self.old_path, &self.new_path)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        crate::path_utils::rename_or_copy_and_delete(&self.new_path, &self.old_path)?;
        self.executed = false;
        Ok(())
    }
}
