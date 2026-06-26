use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lru::LruCache;
use std::num::NonZeroUsize;

use media_sort_backend::media::audio_decoder::AudioPlayer;
use media_sort_core::history::History;
use media_sort_core::media_type::MediaType;
use media_sort_core::models::{FolderNode, MediaEntry, PinnedFolder};
use media_sort_core::settings::store::SettingsStore;

pub const PREFETCH_RADIUS: usize = 5;

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

    pub editing_keybinding: Option<usize>,
    pub waiting_for_key: bool,

    pub metadata_panel_expanded: bool,
    pub current_metadata: Option<BTreeMap<String, BTreeMap<String, String>>>,

    pub show_settings: bool,
    pub show_keybindings: bool,

    pub audio_player: Option<AudioPlayer>,

    pub thumbnail_cache: LruCache<PathBuf, Vec<u8>>,
    pub selected_folder: Option<PathBuf>,
    pub selected_image_bytes: Option<(PathBuf, Vec<u8>)>,
    pub renaming_path: Option<PathBuf>,
    pub rename_input_value: String,
    pub creating_folder_parent: Option<PathBuf>,
    pub create_folder_input: String,
    pub search_focused: bool,
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

        let metadata_panel_expanded = settings.metadata_panel.is_expanded;
        let _dark_mode = settings.general.dark_mode;

        let cache_size = NonZeroUsize::new((PREFETCH_RADIUS * 2).max(1))
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap());

        let audio_player = AudioPlayer::new().ok();

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
            editing_keybinding: None,
            waiting_for_key: false,
            metadata_panel_expanded,
            current_metadata: None,
            show_settings: false,
            show_keybindings: false,
            audio_player,
            thumbnail_cache: LruCache::new(cache_size),
            selected_folder: None,
            selected_image_bytes: None,
            renaming_path: None,
            rename_input_value: String::new(),
            creating_folder_parent: None,
            create_folder_input: String::new(),
            search_focused: false,
        }
    }

    pub fn open_folder(&mut self, path: &Path) {
        self.current_folder = Some(path.to_path_buf());
        self.history.clear();
        self.scan_media();
        self.build_folder_tree(path);
        self.selected_index = None;
        self.current_metadata = None;
        self.selected_folder = None;
        self.selected_image_bytes = None;
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

    pub fn pin_folder(&mut self, path: &Path) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        let pinned = PinnedFolder {
            path: path.to_path_buf(),
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

    pub fn move_pinned_folder_up(&mut self, path: &Path) {
        if let Some(pos) = self.pinned_folders.iter().position(|p| p.path == path) {
            if pos > 0 {
                self.pinned_folders.swap(pos, pos - 1);
                self.settings.pinned_folders.paths = self
                    .pinned_folders
                    .iter()
                    .map(|p| p.path.display().to_string())
                    .collect();
            }
        }
    }

    pub fn move_pinned_folder_down(&mut self, path: &Path) {
        if let Some(pos) = self.pinned_folders.iter().position(|p| p.path == path) {
            if pos < self.pinned_folders.len() - 1 {
                self.pinned_folders.swap(pos, pos + 1);
                self.settings.pinned_folders.paths = self
                    .pinned_folders
                    .iter()
                    .map(|p| p.path.display().to_string())
                    .collect();
            }
        }
    }

    #[allow(dead_code)]
    pub fn save_settings_task(&self) -> iced::Task<crate::message::Message> {
        let _ = self.settings.save();
        iced::Task::none()
    }

    #[allow(dead_code)]
    pub fn save_window_position(&mut self, position: iced::window::Position, size: iced::Size) {
        if let iced::window::Position::Specific(point) = position {
            self.settings.window_position.left = point.x as i32;
            self.settings.window_position.top = point.y as i32;
        }
        self.settings.window_position.width = size.width as u32;
        self.settings.window_position.height = size.height as u32;
        let _ = self.settings.save();
    }
}

pub(crate) fn build_children(parent: &Path, current: Option<&Path>) -> Vec<FolderNode> {
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

pub(crate) fn detect_media_type(ext: &str) -> MediaType {
    let ext = ext.to_lowercase();
    for ty in [MediaType::Image, MediaType::Video, MediaType::Audio] {
        if ty.extensions().contains(&ext.as_str()) {
            return ty;
        }
    }
    MediaType::Image
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_sort_core::media_type::MediaType;
    use media_sort_core::models::MediaEntry;
    use media_sort_core::settings::store::SettingsStore;
    use std::path::PathBuf;

    #[test]
    fn test_detect_media_type_image() {
        assert_eq!(detect_media_type("jpg"), MediaType::Image);
        assert_eq!(detect_media_type("png"), MediaType::Image);
        assert_eq!(detect_media_type("jpeg"), MediaType::Image);
        assert_eq!(detect_media_type("gif"), MediaType::Image);
    }

    #[test]
    fn test_detect_media_type_video() {
        assert_eq!(detect_media_type("mp4"), MediaType::Video);
        assert_eq!(detect_media_type("mkv"), MediaType::Video);
        assert_eq!(detect_media_type("webm"), MediaType::Video);
        assert_eq!(detect_media_type("mov"), MediaType::Video);
    }

    #[test]
    fn test_detect_media_type_audio() {
        assert_eq!(detect_media_type("mp3"), MediaType::Audio);
        assert_eq!(detect_media_type("flac"), MediaType::Audio);
        assert_eq!(detect_media_type("wav"), MediaType::Audio);
        assert_eq!(detect_media_type("ogg"), MediaType::Audio);
    }

    #[test]
    fn test_detect_media_type_unknown_fallback() {
        assert_eq!(detect_media_type("xyz"), MediaType::Image);
        assert_eq!(detect_media_type(""), MediaType::Image);
        assert_eq!(detect_media_type("doc"), MediaType::Image);
    }

    #[test]
    fn test_detect_media_type_case_insensitive() {
        assert_eq!(detect_media_type("JPG"), MediaType::Image);
        assert_eq!(detect_media_type("MP3"), MediaType::Audio);
        assert_eq!(detect_media_type("Mp4"), MediaType::Video);
    }

    #[test]
    fn test_filtered_media_entries_empty_query() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![
            MediaEntry {
                path: "/a.jpg".into(),
                media_type: MediaType::Image,
                file_name: "a.jpg".into(),
            },
            MediaEntry {
                path: "/b.png".into(),
                media_type: MediaType::Image,
                file_name: "b.png".into(),
            },
        ];
        state.search_query = String::new();
        let results = state.filtered_media_entries();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filtered_media_entries_with_query() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![
            MediaEntry {
                path: "/sunset.jpg".into(),
                media_type: MediaType::Image,
                file_name: "sunset.jpg".into(),
            },
            MediaEntry {
                path: "/mountain.png".into(),
                media_type: MediaType::Image,
                file_name: "mountain.png".into(),
            },
        ];
        state.search_query = "sun".into();
        let results = state.filtered_media_entries();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "sunset.jpg");
    }

    #[test]
    fn test_filtered_media_entries_case_insensitive() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![MediaEntry {
            path: "/SUNSET.jpg".into(),
            media_type: MediaType::Image,
            file_name: "SUNSET.jpg".into(),
        }];
        state.search_query = "sun".into();
        let results = state.filtered_media_entries();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_filtered_media_entries_no_match() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![MediaEntry {
            path: "/test.jpg".into(),
            media_type: MediaType::Image,
            file_name: "test.jpg".into(),
        }];
        state.search_query = "nonexistent".into();
        let results = state.filtered_media_entries();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new(SettingsStore::default());
        assert!(state.media_entries.is_empty());
        assert!(state.search_query.is_empty());
        assert!(state.selected_index.is_none());
        assert!(!state.should_exit);
        assert!(!state.show_settings);
        assert!(!state.metadata_panel_expanded);
        assert!(!state.waiting_for_key);
        assert!(state.editing_keybinding.is_none());
        assert_eq!(state.history.done_len(), 0);
    }

    #[test]
    fn test_pin_current_folder() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = PathBuf::from("/test/folder");
        state.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.pinned_folders.len(), 1);
        assert_eq!(state.pinned_folders[0].path, folder);
    }

    #[test]
    fn test_unpin_folder() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = PathBuf::from("/test/folder");
        state.current_folder = Some(folder.clone());
        state.pin_current_folder();
        state.unpin_folder(&folder);
        assert!(state.pinned_folders.is_empty());
    }

    #[test]
    fn test_toggle_expand_collapsed_node() {
        let mut root = FolderNode {
            path: PathBuf::from("/root"),
            name: "root".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
        };
        let child_path = PathBuf::from("/root/sub");
        let found = toggle_expand_recursive(&mut root.children, &child_path);
        assert!(!found);
        let child = FolderNode {
            path: child_path.clone(),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
        };
        root.children = vec![child];
        let found = toggle_expand_recursive(&mut root.children, &child_path);
        assert!(found);
        assert!(root.children[0].is_expanded);
    }

    #[test]
    fn test_toggle_expand_toggle_back() {
        let child = FolderNode {
            path: PathBuf::from("/root/sub"),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: true,
        };
        let mut children = vec![child];
        let found = toggle_expand_recursive(&mut children, &PathBuf::from("/root/sub"));
        assert!(found);
        assert!(!children[0].is_expanded);
    }

    #[test]
    fn test_toggle_expand_nested_path() {
        let grandchild = FolderNode {
            path: PathBuf::from("/root/sub/deep"),
            name: "deep".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
        };
        let child = FolderNode {
            path: PathBuf::from("/root/sub"),
            name: "sub".into(),
            children: vec![grandchild],
            is_current: false,
            is_expanded: false,
        };
        let mut children = vec![child];
        let found = toggle_expand_recursive(&mut children, &PathBuf::from("/root/sub/deep"));
        assert!(found);
        assert!(!children[0].is_expanded);
        assert!(children[0].children[0].is_expanded);
    }

    #[test]
    fn test_pin_current_folder_no_duplicate() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = std::path::PathBuf::from("/test/folder");
        state.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.pinned_folders.len(), 1);
        state.pin_current_folder();
        assert_eq!(state.pinned_folders.len(), 1);
    }

    #[test]
    fn test_pin_current_folder_syncs_settings() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = std::path::PathBuf::from("/test/folder");
        state.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.settings.pinned_folders.paths.len(), 1);
        assert_eq!(state.settings.pinned_folders.paths[0], "/test/folder");
    }

    #[test]
    fn test_pin_current_folder_no_current() {
        let mut state = AppState::new(SettingsStore::default());
        state.current_folder = None;
        state.pin_current_folder();
        assert!(state.pinned_folders.is_empty());
    }

    #[test]
    fn test_build_children_filters_files() {
        let dir = std::env::temp_dir().join(format!("mediasort_bc_filter_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir(&dir.join("subdir")).unwrap();
        std::fs::write(dir.join("file.txt"), b"data").unwrap();
        std::fs::write(dir.join("another.jpg"), b"image").unwrap();

        let children = build_children(&dir, None);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "subdir");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_build_children_nonexistent_dir() {
        let nonexistent = std::path::PathBuf::from("/nonexistent/dir_12345_xyz");
        let children = build_children(&nonexistent, None);
        assert!(children.is_empty());
    }

    #[test]
    fn test_build_children_is_current() {
        let dir = std::env::temp_dir().join(format!("mediasort_bc_current_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();

        let canonical_sub = sub.canonicalize().unwrap();
        let children = build_children(&dir, Some(&canonical_sub));
        assert_eq!(children.len(), 1);
        assert!(children[0].is_current);

        let children2 = build_children(&dir, None);
        assert!(!children2[0].is_current);

        std::fs::remove_dir_all(&dir).ok();
    }
}
