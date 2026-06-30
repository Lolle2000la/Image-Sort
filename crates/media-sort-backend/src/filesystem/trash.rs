use std::path::{Path, PathBuf};

use media_sort_core::actions::delete_action::TrashRestoreHandle;
use media_sort_core::actions::reversible::ActionError;

pub fn delete_to_trash(path: &Path) -> Result<Box<dyn TrashRestoreHandle>, ActionError> {
    let path = path.to_path_buf();

    trash::delete(&path).map_err(|e| ActionError::Io(std::io::Error::other(e.to_string())))?;

    Ok(Box::new(NativeTrashRestore {
        original_path: path,
        flushed: false,
    }))
}

struct NativeTrashRestore {
    original_path: PathBuf,
    flushed: bool,
}

impl TrashRestoreHandle for NativeTrashRestore {
    fn restore(&mut self) -> Result<(), ActionError> {
        if self.flushed {
            return Err(ActionError::RestorationFailed("already flushed".into()));
        }

        let items = trash::os_limited::list()
            .map_err(|e| ActionError::Io(std::io::Error::other(e.to_string())))?;

        let item = items
            .into_iter()
            .find(|i| i.original_path() == self.original_path)
            .ok_or_else(|| {
                ActionError::RestorationFailed("item not found in system trash".into())
            })?;

        trash::os_limited::restore_all([item])
            .map_err(|e| ActionError::Io(std::io::Error::other(e.to_string())))?;

        self.flushed = true;
        Ok(())
    }

    fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
        self.flushed = true;
        Ok(())
    }
}

impl Drop for NativeTrashRestore {
    fn drop(&mut self) {
        if !self.flushed {
            let _ = self.flush_to_native_trash();
        }
    }
}
