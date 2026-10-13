use std::path::{Path, PathBuf};

use crate::actions::reversible::{ActionError, ReversibleAction};

/// Moves `old_path` into `to_folder` without ever replacing an existing
/// destination.
///
/// The `TargetExists` refusal is enforced at construction and re-checked in
/// [`execute`](ReversibleAction::execute) and
/// [`rollback`](ReversibleAction::rollback): Undo/Redo re-run those paths on
/// paths that were freed by the action itself, and a file that appeared at
/// either path in the meantime must not be silently clobbered by `rename(2)`.
pub struct MoveAction {
    old_path: PathBuf,
    new_path: PathBuf,
    executed: bool,
}

impl MoveAction {
    pub fn new(file: &Path, to_folder: &Path) -> Result<Self, ActionError> {
        crate::actions::reversible::reject_symlink_source(file)?;
        let file = file
            .canonicalize()
            .map_err(|_| ActionError::SourceNotFound(file.to_path_buf()))?;
        // Defense in depth: re-check the resolved path in case canonicalize
        // ever stops short of a final link.
        crate::actions::reversible::reject_symlink_source(&file)?;
        let to_folder = to_folder
            .canonicalize()
            .map_err(|_| ActionError::DirectoryNotFound(to_folder.to_path_buf()))?;

        let file_name = file
            .file_name()
            .ok_or_else(|| ActionError::SourceNotFound(file.clone()))?;
        let new_path = to_folder.join(file_name);

        // A plain rename(2) silently REPLACES an existing destination, so a
        // move into a populated folder would destroy the pre-existing file
        // with no warning (rollback cannot restore it). Refuse instead,
        // matching CopyAction/RenameAction. symlink_metadata (not exists())
        // so a dangling destination symlink also counts as taken.
        if new_path.symlink_metadata().is_ok() {
            return Err(ActionError::TargetExists(new_path));
        }

        Ok(Self {
            old_path: file,
            new_path,
            executed: false,
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
    fn display_name(&self, l10n: &crate::l10n::Localization) -> String {
        let file_name = self
            .old_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        let directory = self
            .new_path
            .parent()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();
        l10n.get(
            "move-action-message",
            &[("file_name", file_name), ("directory", &directory)],
        )
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        // Redo re-runs this after rollback freed `new_path`; a file that
        // appeared there in the meantime must not be replaced by rename(2).
        if self.new_path.symlink_metadata().is_ok() {
            return Err(ActionError::TargetExists(self.new_path.clone()));
        }
        crate::path_utils::rename_or_copy_and_delete(&self.old_path, &self.new_path)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        // Undo renames back onto `old_path`, which may have been repopulated
        // since the move — refuse rather than silently replace it.
        if self.old_path.symlink_metadata().is_ok() {
            return Err(ActionError::TargetExists(self.old_path.clone()));
        }
        crate::path_utils::rename_or_copy_and_delete(&self.new_path, &self.old_path)?;
        self.executed = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::reversible::{ActionError, ReversibleAction};
    use crate::actions::test_utils::temp_subdir;
    use std::path::PathBuf;

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
        let l10n = crate::l10n::Localization::init("en");
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("display_name_test.txt");
        std::fs::write(&src_file, b"data").unwrap();

        let action = MoveAction::new(&src_file, &dst_dir).unwrap();
        let name = action.display_name(&l10n);
        assert!(!name.is_empty());
        assert!(name.contains("display_name_test.txt") && name.contains("Move"));
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

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::actions::test_utils::temp_subdir;

    #[test]
    fn test_move_target_exists_refused() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("overwrite_me.txt");
        std::fs::write(&src_file, b"new contents").unwrap();
        std::fs::write(dst_dir.join("overwrite_me.txt"), b"precious old").unwrap();

        let result = MoveAction::new(&src_file, &dst_dir);
        assert!(
            matches!(result, Err(ActionError::TargetExists(_))),
            "move into an existing file must be refused"
        );
        // The pre-existing file must be untouched.
        let old = std::fs::read_to_string(dst_dir.join("overwrite_me.txt")).unwrap();
        assert_eq!(old, "precious old");

        std::fs::remove_dir_all(&src_dir).ok();
        std::fs::remove_dir_all(&dst_dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_move_symlink_source_refused() {
        let root = temp_subdir();
        let src_dir = root.join("src");
        let dst_dir = root.join("dst");
        let outside = root.join("outside");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dst_dir).unwrap();
        std::fs::create_dir_all(&outside).unwrap();

        let victim = outside.join("victim.doc");
        std::fs::write(&victim, b"secret").unwrap();
        let link = src_dir.join("photo.jpg");
        std::os::unix::fs::symlink(&victim, &link).unwrap();

        let result = MoveAction::new(&link, &dst_dir);
        assert!(matches!(result, Err(ActionError::SourceIsSymlink(_))));
        // The victim must remain untouched.
        assert!(victim.exists());
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "secret");

        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_move_dangling_symlink_destination_refused() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("dangling_target.txt");
        std::fs::write(&src_file, b"data").unwrap();
        std::os::unix::fs::symlink(
            src_dir.join("missing.txt"),
            dst_dir.join("dangling_target.txt"),
        )
        .unwrap();

        let result = MoveAction::new(&src_file, &dst_dir);
        assert!(
            matches!(result, Err(ActionError::TargetExists(_))),
            "a dangling symlink at the destination must still refuse the move"
        );

        std::fs::remove_dir_all(&src_dir).ok();
        std::fs::remove_dir_all(&dst_dir).ok();
    }

    #[test]
    fn test_move_rollback_refuses_clobber() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("rollback_clobber.txt");
        std::fs::write(&src_file, b"moved contents").unwrap();

        let mut action = MoveAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        assert!(!src_file.exists());

        // A new file appears at the original location while the action sits
        // in the done stack: rollback must refuse to replace it.
        std::fs::write(&src_file, b"interloper").unwrap();

        let err = action.rollback().unwrap_err();
        assert!(matches!(&err, ActionError::TargetExists(_)));
        // The interloper survives and the moved file stays in the target
        // folder.
        assert_eq!(std::fs::read_to_string(&src_file).unwrap(), "interloper");
        assert!(dst_dir.join("rollback_clobber.txt").exists());

        std::fs::remove_dir_all(&src_dir).ok();
        std::fs::remove_dir_all(&dst_dir).ok();
    }

    #[test]
    fn test_move_redo_refuses_clobber() {
        let src_dir = temp_subdir();
        let dst_dir = temp_subdir();
        let src_file = src_dir.join("redo_clobber.txt");
        std::fs::write(&src_file, b"moved contents").unwrap();

        let mut action = MoveAction::new(&src_file, &dst_dir).unwrap();
        action.execute().unwrap();
        action.rollback().unwrap();
        assert!(src_file.exists());

        // A new file appears at the destination while the action sits in the
        // undone stack: redo must refuse to replace it.
        let dst_file = dst_dir.join("redo_clobber.txt");
        std::fs::write(&dst_file, b"interloper").unwrap();

        let err = action.execute().unwrap_err();
        assert!(matches!(&err, ActionError::TargetExists(_)));
        assert_eq!(std::fs::read_to_string(&dst_file).unwrap(), "interloper");
        assert!(src_file.exists());

        std::fs::remove_dir_all(&src_dir).ok();
        std::fs::remove_dir_all(&dst_dir).ok();
    }
}
