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

#[cfg(test)]
mod tests {
    use super::RenameAction;
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
    fn test_rename_execute() {
        let dir = temp_subdir();
        let file = dir.join("original.txt");
        std::fs::write(&file, b"rename me").unwrap();

        let mut action = RenameAction::new(&file, "renamed").unwrap();
        action.execute().unwrap();

        assert!(!file.exists());
        assert!(dir.join("renamed.txt").exists());
    }

    #[test]
    fn test_rename_rollback() {
        let dir = temp_subdir();
        let file = dir.join("before.txt");
        std::fs::write(&file, b"original content").unwrap();

        let mut action = RenameAction::new(&file, "after").unwrap();
        action.execute().unwrap();
        assert!(dir.join("after.txt").exists());
        assert!(!file.exists());

        action.rollback().unwrap();
        assert!(file.exists());
        assert!(!dir.join("after.txt").exists());

        let contents = std::fs::read_to_string(&file).unwrap();
        assert_eq!(contents, "original content");
    }

    #[test]
    fn test_rename_illegal_characters() {
        let dir = temp_subdir();
        let file = dir.join("legal.txt");
        std::fs::write(&file, b"data").unwrap();

        let result = RenameAction::new(&file, "bad/name");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_target_exists() {
        let dir = temp_subdir();
        let file1 = dir.join("first.txt");
        let file2 = dir.join("second.txt");
        std::fs::write(&file1, b"first").unwrap();
        std::fs::write(&file2, b"second").unwrap();

        let result = RenameAction::new(&file1, "second");
        assert!(result.is_err());
        assert!(matches!(&result, Err(ActionError::TargetExists(_))));
    }

    #[test]
    fn test_rename_preserves_extension() {
        let dir = temp_subdir();
        let file = dir.join("photo.jpg");
        std::fs::write(&file, b"jpeg data").unwrap();

        let mut action = RenameAction::new(&file, "new_name").unwrap();
        action.execute().unwrap();

        assert!(dir.join("new_name.jpg").exists());
        assert!(!file.exists());
    }

    #[test]
    fn test_rename_no_extension() {
        let dir = temp_subdir();
        let file = dir.join("noext");
        std::fs::write(&file, b"content").unwrap();

        let mut action = RenameAction::new(&file, "newname").unwrap();
        assert!(action.display_name().contains("newname"));

        action.execute().unwrap();
        let new_file = dir.join("newname");
        assert!(new_file.exists());
        assert!(!file.exists());

        action.rollback().unwrap();
        assert!(file.exists());
        assert!(!new_file.exists());
    }

    #[test]
    fn test_rename_all_illegal_characters() {
        let dir = temp_subdir();
        let file = dir.join("test.txt");
        std::fs::write(&file, b"content").unwrap();

        for c in &["/", "\\", ":", "*", "?", "\"", "<", ">", "|"] {
            let result = RenameAction::new(&file, &format!("bad{}name", c));
            assert!(result.is_err(), "Expected error for character: {}", c);
        }
    }
}
