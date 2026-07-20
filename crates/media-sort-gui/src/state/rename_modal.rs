use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct RenameModalState {
    pub path: Option<PathBuf>,
    pub input_value: String,
    /// Transient error to display in the rename modal (e.g. illegal character).
    /// Cleared when the modal is dismissed or a valid rename is submitted.
    pub error: Option<String>,
    pub placeholder: String,
}
