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

/// Folder classification used for icon selection in the folder tree.
///
/// The variants are ordered by priority: [`FolderKind::Locked`] wins over
/// [`FolderKind::Symlink`], which wins over [`FolderKind::Git`]. The tree
/// additionally derives `pinned` (bookmark icon) and root-folder status from
/// the node's position/path, both of which override the kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FolderKind {
    #[default]
    Default,
    /// The folder contains a `.git` entry. Lowest priority; any other
    /// classification overrides it.
    Git,
    /// The folder is a symbolic link (followed transparently by the app).
    Symlink,
    /// Rights are restricted: the contents cannot be listed or the folder
    /// is not writable.
    Locked,
}

#[derive(Debug, Clone, Default)]
pub struct FolderNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<FolderNode>,
    pub is_current: bool,
    pub is_expanded: bool,
    pub is_parent_nav: bool,
    /// Icon classification (see [`FolderKind`]); `Default` = plain folder.
    pub kind: FolderKind,
    /// For symlinked folders: the resolved target path, shown next to the
    /// node name. `None` for regular folders.
    pub symlink_target: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PinnedFolder {
    pub path: PathBuf,
    pub name: String,
}
