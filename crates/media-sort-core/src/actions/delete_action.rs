use crate::actions::reversible::{ActionError, ReversibleAction};

pub trait TrashRestoreHandle: Send + Sync {
    fn restore(&mut self) -> Result<(), ActionError>;
    fn flush_to_native_trash(&mut self) -> Result<(), ActionError>;
}

pub struct DeleteAction {
    restore_handle: Option<Box<dyn TrashRestoreHandle>>,
    file_name: String,
}

impl DeleteAction {
    pub fn new(path: &std::path::Path, handle: Box<dyn TrashRestoreHandle>) -> Self {
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            restore_handle: Some(handle),
            file_name,
        }
    }
}

impl ReversibleAction for DeleteAction {
    fn display_name(&self, l10n: &crate::l10n::Localization) -> String {
        l10n.get("delete-action-message", &[("file_name", &self.file_name)])
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        if let Some(mut handle) = self.restore_handle.take() {
            handle.restore()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DeleteAction, TrashRestoreHandle};
    use crate::actions::reversible::{ActionError, ReversibleAction};
    use std::fmt;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    struct MockRestoreHandle {
        restore_call_count: Arc<Mutex<u32>>,
        trash_called: Arc<Mutex<bool>>,
        restore_should_fail: bool,
    }

    impl fmt::Debug for MockRestoreHandle {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MockRestoreHandle").finish()
        }
    }

    impl MockRestoreHandle {
        fn new() -> Self {
            Self {
                restore_call_count: Arc::new(Mutex::new(0)),
                trash_called: Arc::new(Mutex::new(false)),
                restore_should_fail: false,
            }
        }

        fn failing(mut self) -> Self {
            self.restore_should_fail = true;
            self
        }
    }

    impl TrashRestoreHandle for MockRestoreHandle {
        fn restore(&mut self) -> Result<(), ActionError> {
            *self.restore_call_count.lock().unwrap() += 1;
            if self.restore_should_fail {
                Err(ActionError::RestorationFailed("mock restore failed".into()))
            } else {
                Ok(())
            }
        }

        fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
            *self.trash_called.lock().unwrap() = true;
            Ok(())
        }
    }

    #[test]
    fn test_delete_rollback() {
        let handle = MockRestoreHandle::new();
        let restore_call_count = Arc::clone(&handle.restore_call_count);
        let mut action = DeleteAction::new(Path::new("some/file.txt"), Box::new(handle));

        action.rollback().unwrap();
        assert_eq!(*restore_call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_delete_double_rollback() {
        let handle = MockRestoreHandle::new();
        let restore_call_count = Arc::clone(&handle.restore_call_count);
        let mut action = DeleteAction::new(Path::new("some/file.txt"), Box::new(handle));

        action.rollback().unwrap();

        let result = action.rollback();
        assert!(
            result.is_ok(),
            "second rollback is a no-op after handle consumed"
        );
        assert_eq!(
            *restore_call_count.lock().unwrap(),
            1,
            "restore should have been called exactly once; second rollback should not call it again"
        );
    }

    #[test]
    fn test_delete_failing_restore() {
        let handle = Box::new(MockRestoreHandle::new().failing());
        let mut action = DeleteAction::new(Path::new("some/file.txt"), handle);

        let result = action.rollback();
        assert!(result.is_err(), "rollback should propagate restore failure");
        assert!(matches!(&result, Err(ActionError::RestorationFailed(_))));
    }
}
