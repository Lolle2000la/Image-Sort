use std::path::PathBuf;

use crate::media_type::MediaType;

#[derive(Debug, Clone)]
pub struct MediaEntry {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub struct FolderNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<FolderNode>,
    pub is_current: bool,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct PinnedFolder {
    pub path: PathBuf,
    pub name: String,
    pub numeric_shortcut: Option<u8>,
}
