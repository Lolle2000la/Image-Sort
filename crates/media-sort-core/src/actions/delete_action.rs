use crate::actions::reversible::{ActionError, ReversibleAction};

pub trait TrashRestoreHandle: Send + Sync {
    fn restore(&mut self) -> Result<(), ActionError>;
    fn flush_to_native_trash(&mut self) -> Result<(), ActionError>;
}

pub struct DeleteAction {
    restore_handle: Option<Box<dyn TrashRestoreHandle>>,
    display_name: String,
}

impl DeleteAction {
    pub fn new(path: &std::path::Path, handle: Box<dyn TrashRestoreHandle>) -> Self {
        let display_name = format!(
            "Delete {}",
            path.file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default(),
        );
        Self {
            restore_handle: Some(handle),
            display_name,
        }
    }
}

impl ReversibleAction for DeleteAction {
    fn display_name(&self) -> &str {
        &self.display_name
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
