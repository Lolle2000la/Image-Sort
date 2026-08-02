use std::path::PathBuf;

use crate::media_type::MediaType;

#[derive(Debug, Clone)]
pub struct MediaEntry {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub file_name: String,
    /// Cached `is_animated_gif` result, populated at scan time. `Some(_)`
    /// for GIF files (so the second invocation in the thumbnail pipeline
    /// becomes a field read instead of a file open + 2-frame GIF decode);
    /// `None` for non-GIFs where the value is irrelevant.
    pub animated: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct FolderNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<FolderNode>,
    pub is_current: bool,
    pub is_expanded: bool,
    pub is_parent_nav: bool,
}

#[derive(Debug, Clone)]
pub struct PinnedFolder {
    pub path: PathBuf,
    pub name: String,
    pub numeric_shortcut: Option<u8>,
}
