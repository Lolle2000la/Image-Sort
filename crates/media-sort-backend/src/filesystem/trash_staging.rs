use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use media_sort_core::actions::delete_action::TrashRestoreHandle;
use media_sort_core::actions::reversible::ActionError;
use media_sort_core::path_utils;
use parking_lot::Mutex;

pub struct TrashStaging {
    staging_root: PathBuf,
    staged: Mutex<Vec<StagedFile>>,
}

struct StagedFile {
    staging_path: PathBuf,
}

impl TrashStaging {
    pub fn new() -> Result<Self, ActionError> {
        let root = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort")
            .join("trash");

        fs::create_dir_all(&root).map_err(ActionError::Io)?;

        Self::reconcile_orphaned_trash(&root);

        Ok(Self {
            staging_root: root,
            staged: Mutex::new(Vec::new()),
        })
    }

    pub fn reconcile_orphaned_trash(staging_root: &Path) {
        if !staging_root.exists() {
            return;
        }
        let entries = match fs::read_dir(staging_root) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            for file_entry in fs::read_dir(&entry_path).into_iter().flatten().flatten() {
                let file_path = file_entry.path();
                if file_path.is_file() {
                    let _ = trash::delete(&file_path);
                }
            }
            let _ = fs::remove_dir_all(&entry_path);
        }
    }

    pub fn stage_file(&self, path: &Path) -> Result<Box<dyn TrashRestoreHandle>, ActionError> {
        let hash = {
            let mut hasher = DefaultHasher::new();
            path.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };

        let file_name = path
            .file_name()
            .ok_or_else(|| ActionError::SourceNotFound(path.to_path_buf()))?;
        let staging_dir = self.staging_root.join(&hash);
        fs::create_dir_all(&staging_dir).map_err(ActionError::Io)?;

        let staging_path = staging_dir.join(file_name);
        path_utils::rename_or_copy_and_delete(path, &staging_path).map_err(ActionError::Io)?;

        let staged = StagedFile {
            staging_path: staging_path.clone(),
        };
        self.staged.lock().push(staged);

        Ok(Box::new(StagingRestoreHandle {
            original_path: path.to_path_buf(),
            staging_path,
            flushed: false,
        }))
    }

    pub fn flush_all_to_native(&self) {
        let mut staged = self.staged.lock();
        for item in staged.drain(..) {
            let _ = trash::delete(&item.staging_path);
        }
    }
}

struct StagingRestoreHandle {
    original_path: PathBuf,
    staging_path: PathBuf,
    flushed: bool,
}

impl TrashRestoreHandle for StagingRestoreHandle {
    fn restore(&mut self) -> Result<(), ActionError> {
        if self.flushed {
            return Err(ActionError::RestorationFailed("already flushed".into()));
        }
        path_utils::rename_or_copy_and_delete(&self.staging_path, &self.original_path)
            .map_err(ActionError::Io)?;
        Ok(())
    }

    fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
        if self.flushed {
            return Ok(());
        }
        trash::delete(&self.staging_path)
            .map_err(|e| ActionError::RestorationFailed(e.to_string()))?;
        self.flushed = true;
        Ok(())
    }
}

impl Drop for StagingRestoreHandle {
    fn drop(&mut self) {
        if !self.flushed {
            let _ = self.flush_to_native_trash();
        }
    }
}
