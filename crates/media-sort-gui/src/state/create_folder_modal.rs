use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct CreateFolderModalState {
    pub creating_folder_parent: Option<PathBuf>,
    pub create_folder_input: String,
    pub create_folder_placeholder: String,
}
