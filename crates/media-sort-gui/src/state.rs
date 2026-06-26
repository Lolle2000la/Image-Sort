use std::path::{Path, PathBuf};

use media_sort_core::history::History;
use media_sort_core::media_type::MediaType;
use media_sort_core::models::{FolderNode, MediaEntry, PinnedFolder};
use media_sort_core::settings::store::SettingsStore;

pub struct AppState {
    pub history: History,
    pub settings: SettingsStore,
    pub current_folder: Option<PathBuf>,
    pub should_exit: bool,

    pub folder_tree: Vec<FolderNode>,
    pub media_entries: Vec<MediaEntry>,
    pub selected_index: Option<usize>,
    pub search_query: String,
    pub pinned_folders: Vec<PinnedFolder>,
}

impl AppState {
    pub fn new(settings: SettingsStore) -> Self {
        let pinned_folders = settings
            .pinned_folders
            .paths
            .iter()
            .map(|p| {
                let path = PathBuf::from(p);
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.clone());
                PinnedFolder {
                    path,
                    name,
                    numeric_shortcut: None,
                }
            })
            .collect();

        Self {
            history: History::new(),
            settings,
            current_folder: None,
            should_exit: false,
            folder_tree: Vec::new(),
            media_entries: Vec::new(),
            selected_index: None,
            search_query: String::new(),
            pinned_folders,
        }
    }

    pub fn open_folder(&mut self, path: &Path) {
        self.current_folder = Some(path.to_path_buf());
        self.history.clear();
        self.scan_media();
        self.build_folder_tree(path);
        self.selected_index = None;
    }

    pub fn scan_media(&mut self) {
        self.media_entries.clear();
        if let Some(ref folder) = self.current_folder {
            let paths = media_sort_backend::filesystem::scanner::scan_media_files(folder);
            for p in paths {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                let media_type = detect_media_type(ext);
                let file_name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.display().to_string());
                self.media_entries.push(MediaEntry {
                    path: p,
                    media_type,
                    file_name,
                });
            }
        }
    }

    pub fn filtered_media_entries(&self) -> Vec<&MediaEntry> {
        if self.search_query.is_empty() {
            self.media_entries.iter().collect()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.media_entries
                .iter()
                .filter(|e| e.file_name.to_lowercase().contains(&query_lower))
                .collect()
        }
    }

    pub fn build_folder_tree(&mut self, root: &Path) {
        self.folder_tree.clear();

        let root_node = FolderNode {
            path: root.to_path_buf(),
            name: root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| root.display().to_string()),
            children: build_children(root, self.current_folder.as_deref()),
            is_current: self
                .current_folder
                .as_deref()
                .is_some_and(|c| media_sort_core::path_utils::paths_equal(c, root)),
            is_expanded: true,
        };
        self.folder_tree.push(root_node);
    }

    pub fn toggle_folder_expand(&mut self, path: &Path) {
        toggle_expand_recursive(&mut self.folder_tree, path);
    }

    pub fn pin_current_folder(&mut self) {
        if let Some(ref folder) = self.current_folder {
            let name = folder
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let pinned = PinnedFolder {
                path: folder.clone(),
                name,
                numeric_shortcut: None,
            };
            if !self.pinned_folders.iter().any(|p| p.path == pinned.path) {
                self.pinned_folders.push(pinned);
                self.settings.pinned_folders.paths = self
                    .pinned_folders
                    .iter()
                    .map(|p| p.path.display().to_string())
                    .collect();
            }
        }
    }

    pub fn unpin_folder(&mut self, path: &Path) {
        self.pinned_folders.retain(|p| p.path != path);
        self.settings.pinned_folders.paths = self
            .pinned_folders
            .iter()
            .map(|p| p.path.display().to_string())
            .collect();
    }
}

fn build_children(parent: &Path, current: Option<&Path>) -> Vec<FolderNode> {
    let mut children = Vec::new();
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let is_current =
                    current.is_some_and(|c| media_sort_core::path_utils::paths_equal(c, &path));
                let node = FolderNode {
                    path,
                    name,
                    children: Vec::new(),
                    is_current,
                    is_expanded: false,
                };
                children.push(node);
            }
        }
    }
    children.sort_by_key(|a| a.name.to_lowercase());
    children
}

fn toggle_expand_recursive(nodes: &mut [FolderNode], path: &Path) -> bool {
    for node in nodes.iter_mut() {
        if node.path == path {
            node.is_expanded = !node.is_expanded;
            if node.is_expanded && node.children.is_empty() && node.path.is_dir() {
                let current = if node.is_current {
                    Some(node.path.as_path())
                } else {
                    None
                };
                node.children = build_children(&node.path, current);
            }
            return true;
        }
        if toggle_expand_recursive(&mut node.children, path) {
            return true;
        }
    }
    false
}

fn detect_media_type(ext: &str) -> MediaType {
    let ext = ext.to_lowercase();
    for ty in [MediaType::Image, MediaType::Video, MediaType::Audio] {
        if ty.extensions().contains(&ext.as_str()) {
            return ty;
        }
    }
    MediaType::Image
}
