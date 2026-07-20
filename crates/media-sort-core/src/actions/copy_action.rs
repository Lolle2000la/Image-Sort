use std::fs;
use std::path::{Path, PathBuf};

use crate::actions::reversible::{ActionError, ReversibleAction};

pub struct CopyAction {
    source: PathBuf,
    destination: PathBuf,
    executed: bool,
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

        Ok(Self {
            source: file,
            destination,
            executed: false,
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
    fn display_name(&self, l10n: &crate::l10n::Localization) -> String {
        let file_name = self
            .source
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        let directory = self
            .destination
            .parent()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();
        l10n.get(
            "copy-action-message",
            &[("file_name", file_name), ("directory", &directory)],
        )
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        fs::copy(&self.source, &self.destination)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        if !self.executed {
            return Ok(());
        }
        fs::remove_file(&self.destination)?;
        self.executed = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::actions::copy_action::CopyAction;
    use crate::actions::reversible::{ActionError, ReversibleAction};

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
    fn test_copy_execute() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("test_copy_file.txt");
        std::fs::write(&src_file, b"hello copy").unwrap();

        let mut action = CopyAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();

        assert!(src_file.exists(), "source should still exist after copy");
        assert!(dst_dir.join("test_copy_file.txt").exists());
    }

    #[test]
    fn test_copy_rollback() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("test_copy_rollback.txt");
        std::fs::write(&src_file, b"rollback copy").unwrap();

        let mut action = CopyAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        assert!(dst_dir.join("test_copy_rollback.txt").exists());

        action.rollback().unwrap();
        assert!(src_file.exists(), "source should survive rollback");
        assert!(!dst_dir.join("test_copy_rollback.txt").exists());

        let contents = std::fs::read_to_string(&src_file).unwrap();
        assert_eq!(contents, "rollback copy");
    }

    #[test]
    fn test_copy_source_not_found() {
        let dst_dir = temp_subdir();
        let missing = PathBuf::from("/nonexistent/copy_source_12345.txt");
        let result = CopyAction::new(&missing, &dst_dir);
        assert!(result.is_err());
        assert!(matches!(&result, Err(ActionError::SourceNotFound(_))));
    }

    #[test]
    fn test_copy_directory_not_found() {
        let src_dir = temp_subdir();
        let src_file = src_dir.join("exists_for_copy.txt");
        std::fs::write(&src_file, b"data").unwrap();
        let missing_dir = PathBuf::from("/nonexistent/copy_dst_xyz_12345");
        let result = CopyAction::new(&src_file, &missing_dir);
        assert!(result.is_err());
        assert!(matches!(&result, Err(ActionError::DirectoryNotFound(_))));
    }

    #[test]
    fn test_copy_target_exists() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("copy_target_file.txt");
        std::fs::write(&src_file, b"source content").unwrap();
        // create the target in the destination before constructing the action
        std::fs::write(dst_dir.join("copy_target_file.txt"), b"existing").unwrap();

        let result = CopyAction::new(&src_file, &dst_dir);
        assert!(result.is_err());
        assert!(matches!(&result, Err(ActionError::TargetExists(_))));
    }

    #[test]
    fn test_copy_display_name() {
        let l10n = crate::l10n::Localization::init("en");
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("display_copy_test.txt");
        std::fs::write(&src_file, b"data").unwrap();

        let action = CopyAction::new(&src_file, &dst_dir).unwrap();
        let name = action.display_name(&l10n);
        assert!(!name.is_empty());
        assert!(name.contains("display_copy_test.txt"));
    }

    #[test]
    fn test_copy_accessors() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("accessor_copy.txt");
        std::fs::write(&src_file, b"data").unwrap();

        let action = CopyAction::new(&src_file, &dst_dir).unwrap();
        assert_eq!(action.source(), src_file.canonicalize().unwrap());
        assert_eq!(
            action.destination().parent().unwrap(),
            dst_dir.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_copy_double_execute_idempotent() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("double_execute.txt");
        std::fs::write(&src_file, b"double execute content").unwrap();

        let mut action = CopyAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        action.execute().unwrap();

        assert!(
            src_file.exists(),
            "source should still exist after double execute"
        );
        assert!(dst_dir.join("double_execute.txt").exists());
    }

    #[test]
    fn test_copy_double_rollback_idempotent() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("double_rollback.txt");
        std::fs::write(&src_file, b"double rollback content").unwrap();

        let mut action = CopyAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        assert!(dst_dir.join("double_rollback.txt").exists());

        action.rollback().unwrap();
        action.rollback().unwrap();

        assert!(!dst_dir.join("double_rollback.txt").exists());
        assert!(
            src_file.exists(),
            "source should remain after double rollback"
        );
    }

    #[test]
    fn test_copy_rollback_without_execute() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("rollback_no_exec.txt");
        std::fs::write(&src_file, b"rollback without execute").unwrap();

        let mut action = CopyAction::new(&src_file, &dst_dir).unwrap();
        action.rollback().unwrap();

        assert!(src_file.exists(), "source should still exist");
        assert!(!dst_dir.join("rollback_no_exec.txt").exists());
    }

    #[test]
    fn test_copy_execute_then_double_execute_then_rollback() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("exec2_rollback.txt");
        std::fs::write(&src_file, b"original").unwrap();

        let mut action = CopyAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        let dest_path = dst_dir.join("exec2_rollback.txt");
        assert!(dest_path.exists());
        assert_eq!(std::fs::read_to_string(&dest_path).unwrap(), "original");

        std::fs::write(&src_file, b"modified").unwrap();
        action.execute().unwrap();
        assert_eq!(std::fs::read_to_string(&dest_path).unwrap(), "modified");

        action.rollback().unwrap();
        assert!(!dest_path.exists());
        assert!(src_file.exists(), "source should survive rollback");
        assert_eq!(std::fs::read_to_string(&src_file).unwrap(), "modified");
    }
}
