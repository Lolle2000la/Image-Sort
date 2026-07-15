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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::reversible::{ActionError, ReversibleAction};
    use std::path::PathBuf;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("media-sort-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }
    fn rand_u32() -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }
    fn temp_subdir() -> std::path::PathBuf {
        let dir = temp_dir().join(format!("sub-{}", rand_u32()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn test_move_execute() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();

        let src_file = src_dir.join("test_move_file.txt");
        std::fs::write(&src_file, b"hello move").unwrap();

        let mut action = MoveAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();

        assert!(!src_file.exists());
        assert!(dst_dir.join("test_move_file.txt").exists());
    }

    #[test]
    fn test_move_rollback() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();

        let src_file = src_dir.join("test_rollback_file.txt");
        std::fs::write(&src_file, b"rollback me").unwrap();

        let mut action = MoveAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        assert!(!src_file.exists());

        action.rollback().unwrap();
        assert!(src_file.exists());
        assert!(!dst_dir.join("test_rollback_file.txt").exists());

        let contents = std::fs::read_to_string(&src_file).unwrap();
        assert_eq!(contents, "rollback me");
    }

    #[test]
    fn test_move_source_not_found() {
        let dst_dir = temp_subdir();
        let missing = PathBuf::from("/nonexistent/file_that_does_not_exist_12345.txt");
        let result = MoveAction::new(&missing, &dst_dir);
        assert!(result.is_err());
        assert!(matches!(&result, Err(ActionError::SourceNotFound(_))));
    }

    #[test]
    fn test_move_directory_not_found() {
        let src_dir = temp_subdir();
        let src_file = src_dir.join("exists.txt");
        std::fs::write(&src_file, b"data").unwrap();

        let missing_dir = PathBuf::from("/nonexistent/directory_xyz_12345");
        let result = MoveAction::new(&src_file, &missing_dir);
        assert!(result.is_err());
        assert!(matches!(&result, Err(ActionError::DirectoryNotFound(_))));
    }

    #[test]
    fn test_move_display_name() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("display_name_test.txt");
        std::fs::write(&src_file, b"data").unwrap();

        let action = MoveAction::new(&src_file, &dst_dir).unwrap();
        let name = action.display_name();
        assert!(!name.is_empty());
        assert!(name.contains("display_name_test.txt") || name.contains("Move"));
    }

    #[test]
    fn test_move_action_accessors() {
        let dir = temp_subdir();
        let src_file = dir.join("accessor_test.txt");
        let dst_dir = dir.join("dest");
        std::fs::create_dir(&dst_dir).unwrap();
        std::fs::write(&src_file, b"test").unwrap();

        let action = MoveAction::new(&src_file, &dst_dir).unwrap();
        assert_eq!(action.old_path(), src_file.canonicalize().unwrap());
        assert_eq!(
            action.new_path().parent().unwrap(),
            dst_dir.canonicalize().unwrap()
        );
    }
}
