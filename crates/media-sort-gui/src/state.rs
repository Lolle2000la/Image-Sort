mod audio;
mod cache;
mod create_folder_modal;
pub mod drag_drop;
mod folder;
pub mod media_errors;
mod media_grid;
mod metadata;
mod rename_modal;
mod settings_ui;
mod video;

pub use audio::AudioPlaybackState;
pub use cache::CacheState;
pub use create_folder_modal::CreateFolderModalState;
pub use drag_drop::DragDropState;
pub use folder::FolderState;
pub use media_grid::{MediaGridScrollState, MediaGridState, SearchState};
pub use metadata::MetadataPanelState;
pub use rename_modal::RenameModalState;
pub use settings_ui::SettingsUiState;
pub use video::VideoPlaybackState;

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use media_sort_core::history::History;
use media_sort_core::media_type::{MediaRegistry, MediaType};
use media_sort_core::models::{FolderNode, PinnedFolder};
use media_sort_core::settings::store::SettingsStore;

#[cfg_attr(feature = "demo", iced_automation::state(crate::message::Message))]
pub struct AppState {
    pub history: History,
    pub settings: SettingsStore,
    pub should_exit: bool,
    pub l10n: media_sort_core::l10n::Localization,
    pub show_credits: bool,

    pub folder: FolderState,
    pub media_grid: MediaGridState,
    pub rename: RenameModalState,
    pub create_folder: CreateFolderModalState,
    pub video: VideoPlaybackState,
    pub audio: AudioPlaybackState,
    pub cache: CacheState,
    pub metadata: MetadataPanelState,
    pub settings_ui: SettingsUiState,
    pub drag_drop: DragDropState,

    #[cfg(feature = "velopack")]
    pub pending_update: Option<velopack::UpdateInfo>,
    #[cfg(feature = "velopack")]
    pub show_update_prompt: bool,
}

impl AppState {
    pub fn new(settings: SettingsStore) -> Self {
        let pinned_folders: Vec<PinnedFolder> = settings
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

        let panel_expanded = settings.metadata_panel.is_expanded;

        let detected_locale = match &settings.general.locale {
            Some(locale) => locale.as_str(),
            None => media_sort_core::l10n::detect_locale(),
        };
        let l10n = media_sort_core::l10n::Localization::init(detected_locale);
        let search_placeholder = l10n.tr("keybindings-search-images");
        let rename_placeholder = l10n.tr("ui-enter-new-name");
        let create_folder_placeholder = l10n.tr("ui-folder-name-placeholder");

        let folder = FolderState {
            pinned_folders,
            ..Default::default()
        };

        let media_grid = MediaGridState {
            search: SearchState {
                placeholder: search_placeholder,
                ..Default::default()
            },
            ..Default::default()
        };

        let rename = RenameModalState {
            placeholder: rename_placeholder,
            ..Default::default()
        };

        let create_folder = CreateFolderModalState {
            create_folder_placeholder,
            ..Default::default()
        };

        let metadata = MetadataPanelState {
            panel_expanded,
            ..Default::default()
        };

        Self {
            history: History::new(),
            settings,
            should_exit: false,
            l10n,
            show_credits: false,
            folder,
            media_grid,
            rename,
            create_folder,
            video: VideoPlaybackState::default(),
            audio: AudioPlaybackState::new(),
            cache: CacheState::new(),
            metadata,
            settings_ui: SettingsUiState::default(),
            drag_drop: DragDropState::new(),
            #[cfg(feature = "velopack")]
            pending_update: None,
            #[cfg(feature = "velopack")]
            show_update_prompt: false,
            #[cfg(feature = "demo")]
            automation: Default::default(),
        }
    }

    pub fn open_folder(&mut self, path: &Path) {
        self.folder.current_folder = Some(path.to_path_buf());
        self.settings.general.last_opened_folder = Some(path.to_string_lossy().to_string());
        let _ = self.settings.save();
        self.history.clear();
        self.media_grid.entries.clear();
        self.build_folder_tree();
        self.media_grid.selected_index = None;
        self.metadata.current = None;
        self.folder.selected_folder = None;
        self.folder.selected_folder_idx = None;
        self.cache.selected_image = None;
        self.cache.image_cache.clear();
        if let Some(ref sender) = self.video.sender {
            let _ =
                sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
        }
        self.video.frame = None;
        self.video.position = 0.0;
        self.video.duration = 0.0;
        self.cache.media_errors.clear();
        if let Some(ref player) = self.audio.player {
            player.stop();
        }
        self.audio.playing = false;
        self.audio.position = 0.0;

        self.start_async_folder_tree();

        self.media_grid.scan_receiver = Some(
            media_sort_backend::filesystem::scanner::scan_media_files(path),
        );
        self.media_grid.pending_select_index = Some(0);
    }

    pub fn scan_media(&mut self) {
        let animate_gifs = self.settings.general.animate_gifs;
        let folder = self.folder.current_folder.as_deref();
        self.media_grid.scan_media(folder, animate_gifs);
    }

    pub fn build_folder_tree(&mut self) {
        if self.folder.current_folder.is_none() {
            return;
        }
        self.folder.folder_tree_receiver = None;
        let expanded_paths = collect_expanded_paths(&self.folder.folder_tree);
        let root = self
            .folder
            .current_folder
            .clone()
            .expect("current_folder must be Some since we checked it is not None above");
        self.folder.folder_tree =
            build_tree_nodes_data(&root, &self.folder.pinned_folders, &expanded_paths);
        self.folder.sync_selected_idx();
    }

    pub fn start_async_folder_tree(&mut self) {
        let Some(ref current) = self.folder.current_folder else {
            return;
        };
        let root = current.clone();
        let pinned = self.folder.pinned_folders.clone();
        let expanded_paths = collect_expanded_paths(&self.folder.folder_tree);

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let tree = build_tree_nodes_data(&root, &pinned, &expanded_paths);
            let _ = tx.send(tree);
        });
        self.folder.folder_tree_receiver = Some(rx);
    }

    pub fn toggle_folder_expand(&mut self, path: &Path) {
        toggle_expand_recursive(
            &mut self.folder.folder_tree,
            path,
            self.folder.current_folder.as_deref(),
        );
        self.folder.sync_selected_idx();
    }

    #[allow(dead_code)]
    pub fn pin_current_folder(&mut self) {
        if let Some(ref folder) = self.folder.current_folder {
            let name = folder
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let pinned = PinnedFolder {
                path: folder.clone(),
                name,
                numeric_shortcut: None,
            };
            if !self
                .folder
                .pinned_folders
                .iter()
                .any(|p| p.path == pinned.path)
            {
                self.folder.pinned_folders.push(pinned);
                self.settings.pinned_folders.paths = self
                    .folder
                    .pinned_folders
                    .iter()
                    .map(|p| p.path.display().to_string())
                    .collect();
                self.build_folder_tree();
            }
        }
    }

    pub fn unpin_folder(&mut self, path: &Path) {
        self.folder.pinned_folders.retain(|p| p.path != path);
        self.settings.pinned_folders.paths = self
            .folder
            .pinned_folders
            .iter()
            .map(|p| p.path.display().to_string())
            .collect();
        self.build_folder_tree();
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
        if !self
            .folder
            .pinned_folders
            .iter()
            .any(|p| p.path == pinned.path)
        {
            self.folder.pinned_folders.push(pinned);
            self.settings.pinned_folders.paths = self
                .folder
                .pinned_folders
                .iter()
                .map(|p| p.path.display().to_string())
                .collect();
            self.build_folder_tree();
        }
    }

    pub fn move_pinned_folder_up(&mut self, path: &Path) {
        if let Some(pos) = self
            .folder
            .pinned_folders
            .iter()
            .position(|p| p.path == path)
            && pos > 0
        {
            self.folder.pinned_folders.swap(pos, pos - 1);
            // Index 0 is the current-folder root; pinned folders start at 1.
            if pos + 1 < self.folder.folder_tree.len() {
                self.folder.folder_tree.swap(pos + 1, pos);
            }
            self.settings.pinned_folders.paths = self
                .folder
                .pinned_folders
                .iter()
                .map(|p| p.path.display().to_string())
                .collect();
            let _ = self.settings.save();
        }
    }

    pub fn move_pinned_folder_down(&mut self, path: &Path) {
        if let Some(pos) = self
            .folder
            .pinned_folders
            .iter()
            .position(|p| p.path == path)
            && pos < self.folder.pinned_folders.len() - 1
        {
            self.folder.pinned_folders.swap(pos, pos + 1);
            // Index 0 is the current-folder root; pinned folders start at 1.
            if pos + 2 < self.folder.folder_tree.len() {
                self.folder.folder_tree.swap(pos + 1, pos + 2);
            }
            self.settings.pinned_folders.paths = self
                .folder
                .pinned_folders
                .iter()
                .map(|p| p.path.display().to_string())
                .collect();
            let _ = self.settings.save();
        }
    }

    pub fn swap_pinned_folders(&mut self, pos_a: usize, pos_b: usize) {
        self.folder.pinned_folders.swap(pos_a, pos_b);
        if pos_a + 1 < self.folder.folder_tree.len() && pos_b + 1 < self.folder.folder_tree.len() {
            self.folder.folder_tree.swap(pos_a + 1, pos_b + 1);
        }
        self.settings.pinned_folders.paths = self
            .folder
            .pinned_folders
            .iter()
            .map(|p| p.path.display().to_string())
            .collect();
    }
}

fn collect_expanded_paths(tree: &[FolderNode]) -> std::collections::HashSet<PathBuf> {
    let mut set = std::collections::HashSet::new();
    fn collect(nodes: &[FolderNode], set: &mut std::collections::HashSet<PathBuf>) {
        for node in nodes {
            if node.is_expanded {
                set.insert(node.path.clone());
            }
            collect(&node.children, set);
        }
    }
    collect(tree, &mut set);
    set
}

fn build_tree_nodes_data(
    root: &Path,
    pinned_folders: &[PinnedFolder],
    expanded_paths: &std::collections::HashSet<PathBuf>,
) -> Vec<FolderNode> {
    fn restore_expansion(nodes: &mut [FolderNode], set: &std::collections::HashSet<PathBuf>) {
        for node in nodes {
            if set.contains(&node.path) {
                node.is_expanded = true;
            }
            restore_expansion(&mut node.children, set);
        }
    }

    let mut tree = Vec::new();

    let mut children: Vec<_> = build_parent_chain(root)
        .into_iter()
        .rev()
        .chain(build_children(root, Some(root)))
        .collect();
    restore_expansion(&mut children, expanded_paths);
    tree.push(FolderNode {
        path: root.to_path_buf(),
        name: root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.display().to_string()),
        children,
        is_current: true,
        is_expanded: expanded_paths.is_empty() || expanded_paths.contains(root),
        is_parent_nav: false,
    });

    for pinned in pinned_folders {
        if media_sort_core::path_utils::paths_equal(root, &pinned.path) {
            continue;
        }
        let mut pinned_children: Vec<_> = build_parent_chain(&pinned.path)
            .into_iter()
            .rev()
            .chain(build_children(&pinned.path, Some(root)))
            .collect();
        restore_expansion(&mut pinned_children, expanded_paths);
        tree.push(FolderNode {
            path: pinned.path.clone(),
            name: pinned.name.clone(),
            children: pinned_children,
            is_current: false,
            is_expanded: expanded_paths.contains(&pinned.path),
            is_parent_nav: false,
        });
    }

    tree
}

fn first_visible_child(nodes: &[FolderNode], path: &Path) -> Option<PathBuf> {
    for node in nodes {
        if node.path.as_os_str().is_empty() {
            continue;
        }
        if node.path == path {
            return node
                .children
                .iter()
                .find(|c| !c.path.as_os_str().is_empty())
                .map(|c| c.path.clone());
        }
        if let Some(res) = first_visible_child(&node.children, path) {
            return Some(res);
        }
    }
    None
}

pub(crate) fn build_children(parent: &Path, current: Option<&Path>) -> Vec<FolderNode> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };

    let mut children: Vec<FolderNode> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_dir()))
        .map(|entry| {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let is_current =
                current.is_some_and(|c| media_sort_core::path_utils::paths_equal(c, &path));

            let has_child_dir = std::fs::read_dir(&path).is_ok_and(|sub_entries| {
                sub_entries
                    .flatten()
                    .any(|e| e.file_type().is_ok_and(|ft| ft.is_dir()))
            });

            let node_children = if has_child_dir {
                vec![FolderNode {
                    path: PathBuf::new(),
                    name: String::new(),
                    children: Vec::new(),
                    is_current: false,
                    is_expanded: true,
                    is_parent_nav: false,
                }]
            } else {
                Vec::new()
            };

            FolderNode {
                path,
                name,
                children: node_children,
                is_current,
                is_expanded: false,
                is_parent_nav: false,
            }
        })
        .collect();

    children.sort_by_key(|a| a.name.to_lowercase());
    children
}

fn is_dummy_or_empty(children: &[FolderNode]) -> bool {
    children.is_empty() || (children.len() == 1 && children[0].path.as_os_str().is_empty())
}

fn build_parent_chain(current: &Path) -> Vec<FolderNode> {
    let ancestors: Vec<std::path::PathBuf> =
        std::iter::successors(current.parent(), |p| p.parent())
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .collect();

    if ancestors.is_empty() {
        return Vec::new();
    }

    ancestors
        .into_iter()
        .rev()
        .fold(None, |prev: Option<FolderNode>, ancestor| {
            let name = ancestor
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ancestor.display().to_string());

            Some(FolderNode {
                path: ancestor,
                name,
                children: prev.map(|p| vec![p]).unwrap_or_default(),
                is_current: false,
                is_expanded: false,
                is_parent_nav: true,
            })
        })
        .map(|rootmost| vec![rootmost])
        .unwrap_or_default()
}

fn toggle_expand_recursive(
    nodes: &mut [FolderNode],
    path: &Path,
    current_folder: Option<&Path>,
) -> bool {
    for node in nodes.iter_mut() {
        if node.path == path {
            if node.path.exists() && node.children.is_empty() && !node.is_parent_nav {
                return true;
            }
            node.is_expanded = !node.is_expanded;
            if node.is_expanded
                && (is_dummy_or_empty(&node.children) || node.is_parent_nav)
                && node.path.is_dir()
            {
                let current = if node.is_current {
                    Some(node.path.as_path())
                } else {
                    current_folder
                };

                let parent_nav_nodes: Vec<FolderNode> = node
                    .children
                    .drain(..)
                    .filter(|c| c.is_parent_nav)
                    .collect();

                let mut new_children = build_children(&node.path, current);

                new_children.splice(0..0, parent_nav_nodes);

                node.children = new_children;
            }
            return true;
        }
        if toggle_expand_recursive(&mut node.children, path, current_folder) {
            return true;
        }
    }
    false
}

pub(crate) fn collect_visible_folders_recursive(nodes: &[FolderNode], list: &mut Vec<PathBuf>) {
    for node in nodes {
        if node.path.as_os_str().is_empty() {
            continue;
        }
        list.push(node.path.clone());
        if node.is_expanded {
            collect_visible_folders_recursive(&node.children, list);
        }
    }
}

fn set_expand_recursive(
    nodes: &mut [FolderNode],
    path: &Path,
    expand: bool,
    current_folder: Option<&Path>,
) -> bool {
    for node in nodes.iter_mut() {
        if node.path == path {
            if expand && node.path.exists() && node.children.is_empty() && !node.is_parent_nav {
                return true;
            }
            if node.is_expanded != expand {
                node.is_expanded = expand;
                if node.is_expanded
                    && (is_dummy_or_empty(&node.children) || node.is_parent_nav)
                    && node.path.is_dir()
                {
                    let current = if node.is_current {
                        Some(node.path.as_path())
                    } else {
                        current_folder
                    };

                    let parent_nav_nodes: Vec<FolderNode> = node
                        .children
                        .drain(..)
                        .filter(|c| c.is_parent_nav)
                        .collect();

                    let mut new_children = build_children(&node.path, current);

                    new_children.splice(0..0, parent_nav_nodes);

                    node.children = new_children;
                }
            }
            return true;
        }
        if set_expand_recursive(&mut node.children, path, expand, current_folder) {
            return true;
        }
    }
    false
}

fn find_node_expanded(nodes: &[FolderNode], path: &Path) -> Option<bool> {
    for node in nodes {
        if node.path.as_os_str().is_empty() {
            continue;
        }
        if node.path == path {
            return Some(node.is_expanded);
        }
        if let Some(res) = find_node_expanded(&node.children, path) {
            return Some(res);
        }
    }
    None
}

/// Detects the media type for a file path using a two-tier resolution:
///
/// 1. **Fast first pass** — checks the compile-time baseline extensions from
///    [`MediaType::extensions`]. This covers all native formats plus the
///    hardcoded video subset without touching the global registry.
/// 2. **Registry fallback** — if the extension didn't match the baseline,
///    queries [`MediaRegistry::determine_type`] which accounts for
///    mpv-discovered formats that may not be in the hardcoded list.
/// 3. **Default** — unknown extensions fall back to `MediaType::Image`.
///
/// After classification, GIF files get special handling: static GIFs and
/// GIFs with animation disabled are reclassified as `Image`.
pub(crate) fn detect_media_type(path: &std::path::Path, animate_gifs: bool) -> MediaType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let media_type = [MediaType::Image, MediaType::Video, MediaType::Audio]
        .into_iter()
        .find(|ty| ty.extensions().contains(&ext.as_str()))
        .or_else(|| MediaRegistry::determine_type(&ext))
        .unwrap_or(MediaType::Image);
    if media_type == MediaType::Video && ext == "gif" {
        if media_sort_backend::media::image_decoder::is_animated_gif(path) == Some(false) {
            return MediaType::Image;
        }
        if !animate_gifs {
            return MediaType::Image;
        }
    }
    media_type
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_sort_core::models::MediaEntry;
    use media_sort_core::settings::store::SettingsStore;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_detect_media_type_image() {
        assert_eq!(
            detect_media_type(Path::new("test.jpg"), true),
            MediaType::Image
        );
        assert_eq!(
            detect_media_type(Path::new("test.png"), true),
            MediaType::Image
        );
        assert_eq!(
            detect_media_type(Path::new("test.jpeg"), true),
            MediaType::Image
        );
        assert_eq!(
            detect_media_type(Path::new("test.bmp"), true),
            MediaType::Image
        );
    }

    #[test]
    fn test_detect_media_type_video() {
        assert_eq!(
            detect_media_type(Path::new("test.mp4"), true),
            MediaType::Video
        );
        assert_eq!(
            detect_media_type(Path::new("test.mkv"), true),
            MediaType::Video
        );
        assert_eq!(
            detect_media_type(Path::new("test.webm"), true),
            MediaType::Video
        );
        assert_eq!(
            detect_media_type(Path::new("test.mov"), true),
            MediaType::Video
        );
        assert_eq!(
            detect_media_type(Path::new("test.gif"), true),
            MediaType::Video
        );
    }

    #[test]
    fn test_detect_media_type_audio() {
        assert_eq!(
            detect_media_type(Path::new("test.mp3"), true),
            MediaType::Audio
        );
        assert_eq!(
            detect_media_type(Path::new("test.flac"), true),
            MediaType::Audio
        );
        assert_eq!(
            detect_media_type(Path::new("test.wav"), true),
            MediaType::Audio
        );
        assert_eq!(
            detect_media_type(Path::new("test.ogg"), true),
            MediaType::Audio
        );
    }

    #[test]
    fn test_detect_media_type_unknown_fallback() {
        assert_eq!(
            detect_media_type(Path::new("test.xyz"), true),
            MediaType::Image
        );
        assert_eq!(detect_media_type(Path::new("test"), true), MediaType::Image);
        assert_eq!(
            detect_media_type(Path::new("test.doc"), true),
            MediaType::Image
        );
    }

    #[test]
    fn test_detect_media_type_case_insensitive() {
        assert_eq!(
            detect_media_type(Path::new("test.JPG"), true),
            MediaType::Image
        );
        assert_eq!(
            detect_media_type(Path::new("test.MP3"), true),
            MediaType::Audio
        );
        assert_eq!(
            detect_media_type(Path::new("test.Mp4"), true),
            MediaType::Video
        );
    }

    #[test]
    fn test_detect_media_type_static_gif() {
        let dir = std::env::temp_dir().join("mediasort_test_static_gif");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("static.gif");

        let img = image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]));
        img.save(&path).unwrap();

        assert_eq!(detect_media_type(&path, true), MediaType::Image);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_filtered_media_entries_empty_query() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_grid.entries = vec![
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
        state.media_grid.search.query = String::new();
        let results = state.media_grid.filtered_entries();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filtered_media_entries_with_query() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_grid.entries = vec![
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
        state.media_grid.search.query = "sun".into();
        let results = state.media_grid.filtered_entries();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "sunset.jpg");
    }

    #[test]
    fn test_filtered_media_entries_case_insensitive() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_grid.entries = vec![MediaEntry {
            path: "/SUNSET.jpg".into(),
            media_type: MediaType::Image,
            file_name: "SUNSET.jpg".into(),
        }];
        state.media_grid.search.query = "sun".into();
        let results = state.media_grid.filtered_entries();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_filtered_media_entries_no_match() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_grid.entries = vec![MediaEntry {
            path: "/test.jpg".into(),
            media_type: MediaType::Image,
            file_name: "test.jpg".into(),
        }];
        state.media_grid.search.query = "nonexistent".into();
        let results = state.media_grid.filtered_entries();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new(SettingsStore::default());
        assert!(state.media_grid.entries.is_empty());
        assert!(state.media_grid.search.query.is_empty());
        assert!(state.media_grid.selected_index.is_none());
        assert!(!state.should_exit);
        assert!(matches!(state.settings_ui, SettingsUiState::Hidden));
        assert!(!state.metadata.panel_expanded);
        assert!(matches!(state.settings_ui, SettingsUiState::Hidden));
        assert_eq!(state.history.done_len(), 0);
    }

    #[test]
    fn test_pin_current_folder() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = PathBuf::from("/test/folder");
        state.folder.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.folder.pinned_folders.len(), 1);
        assert_eq!(state.folder.pinned_folders[0].path, folder);
    }

    #[test]
    fn test_unpin_folder() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = PathBuf::from("/test/folder");
        state.folder.current_folder = Some(folder.clone());
        state.pin_current_folder();
        state.unpin_folder(&folder);
        assert!(state.folder.pinned_folders.is_empty());
    }

    #[test]
    fn test_toggle_expand_collapsed_node() {
        let mut root = FolderNode {
            path: PathBuf::from("/root"),
            name: "root".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let child_path = PathBuf::from("/root/sub");
        let found = toggle_expand_recursive(&mut root.children, &child_path, None);
        assert!(!found);
        let child = FolderNode {
            path: child_path.clone(),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        root.children = vec![child];
        let found = toggle_expand_recursive(&mut root.children, &child_path, None);
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
            is_parent_nav: false,
        };
        let mut children = vec![child];
        let found = toggle_expand_recursive(&mut children, &PathBuf::from("/root/sub"), None);
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
            is_parent_nav: false,
        };
        let child = FolderNode {
            path: PathBuf::from("/root/sub"),
            name: "sub".into(),
            children: vec![grandchild],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let mut children = vec![child];
        let found = toggle_expand_recursive(&mut children, &PathBuf::from("/root/sub/deep"), None);
        assert!(found);
        assert!(!children[0].is_expanded);
        assert!(children[0].children[0].is_expanded);
    }

    #[test]
    fn test_toggle_expand_parent_nav_node() {
        let dir = std::env::temp_dir().join(format!("mediasort_test_nav_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub1 = dir.join("sub1");
        let sub2 = dir.join("sub2");
        std::fs::create_dir(&sub1).unwrap();
        std::fs::create_dir(&sub2).unwrap();

        let child_node = FolderNode {
            path: sub1.clone(),
            name: "sub1".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };

        let nav_node = FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![child_node],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
        };

        let mut tree = vec![nav_node];
        let found = toggle_expand_recursive(&mut tree, &dir, Some(&sub1));

        assert!(found);
        assert!(tree[0].is_expanded);
        assert_eq!(tree[0].children.len(), 2);
        assert!(tree[0].is_parent_nav);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_toggle_expand_parent_nav_preserves_chain() {
        let dir = std::env::temp_dir().join(format!("mediasort_test_chain_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub1 = dir.join("sub1");
        std::fs::create_dir(&sub1).unwrap();

        let grandparent_node = FolderNode {
            path: PathBuf::from("/grandparent"),
            name: "grandparent".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
        };

        let nav_node = FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![grandparent_node],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
        };

        let mut tree = vec![nav_node];
        let found = toggle_expand_recursive(&mut tree, &dir, Some(&sub1));

        assert!(found);
        assert!(tree[0].is_expanded);
        assert_eq!(tree[0].children.len(), 2);
        assert!(
            tree[0]
                .children
                .iter()
                .any(|c| c.path == std::path::Path::new("/grandparent") && c.is_parent_nav)
        );
        assert!(tree[0].children.iter().any(|c| c.path == sub1));
        assert!(tree[0].is_parent_nav);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_toggle_expand_parent_nav_retains_special_handling() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_test_handling_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub1 = dir.join("sub1");
        std::fs::create_dir(&sub1).unwrap();

        let nav_node = FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
        };

        let mut tree = vec![nav_node];
        let found = toggle_expand_recursive(&mut tree, &dir, Some(&sub1));

        assert!(found);
        assert!(tree[0].is_expanded);
        assert!(
            tree[0].is_parent_nav,
            "Folder lost its special parent navigation status upon expansion!"
        );
        assert!(
            !tree[0].children.is_empty(),
            "children should be populated after expand"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_pin_current_folder_no_duplicate() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = std::path::PathBuf::from("/test/folder");
        state.folder.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.folder.pinned_folders.len(), 1);
        state.pin_current_folder();
        assert_eq!(state.folder.pinned_folders.len(), 1);
    }

    #[test]
    fn test_pin_current_folder_syncs_settings() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = std::path::PathBuf::from("/test/folder");
        state.folder.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.settings.pinned_folders.paths.len(), 1);
        assert_eq!(state.settings.pinned_folders.paths[0], "/test/folder");
    }

    #[test]
    fn test_pin_current_folder_no_current() {
        let mut state = AppState::new(SettingsStore::default());
        state.folder.current_folder = None;
        state.pin_current_folder();
        assert!(state.folder.pinned_folders.is_empty());
    }

    #[test]
    fn test_build_children_filters_files() {
        let dir = std::env::temp_dir().join(format!("mediasort_bc_filter_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir(dir.join("subdir")).unwrap();
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

    #[test]
    fn test_build_children_no_subdirectories_no_dummy() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_test_nodummy_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let sub = dir.join("sub_with_only_files");
        std::fs::create_dir(&sub).unwrap();

        for i in 0..5 {
            std::fs::write(sub.join(format!("file_{}.jpg", i)), b"data").unwrap();
        }

        let children = build_children(&dir, None);

        assert_eq!(children.len(), 1);
        assert!(
            children[0].children.is_empty(),
            "Dummy node injected into a directory containing zero subfolders!"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_toggle_expand_parent_nav_idempotency() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_test_idempotency_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub = dir.join("sub1");
        std::fs::create_dir(&sub).unwrap();

        let grandparent_node = FolderNode {
            path: PathBuf::from("/grandparent"),
            name: "grandparent".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
        };

        let mut tree = vec![FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![grandparent_node],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
        }];

        toggle_expand_recursive(&mut tree, &dir, Some(&sub));
        assert_eq!(tree[0].children.len(), 2);

        toggle_expand_recursive(&mut tree, &dir, Some(&sub));
        assert!(!tree[0].is_expanded);

        toggle_expand_recursive(&mut tree, &dir, Some(&sub));
        assert!(tree[0].is_expanded);
        assert_eq!(
            tree[0].children.len(),
            2,
            "Re-expanding a parent navigation node duplicated or corrupted the child array!"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_build_parent_chain_linear_structure() {
        let deep_path = PathBuf::from("/a/b/c/d");
        let chain = build_parent_chain(&deep_path);

        assert_eq!(chain.len(), 1);
        assert!(chain[0].is_parent_nav);

        let mut current = &chain[0];
        let expected = ["/a/b/c", "/a/b", "/a", "/"];
        for exp in &expected {
            assert_eq!(current.path, PathBuf::from(exp), "at path {exp}");
            if current.children.len() == 1 {
                current = &current.children[0];
            }
        }
        assert!(current.children.is_empty());
    }

    #[test]
    fn test_folder_tree_navigation() {
        let mut state = AppState::new(SettingsStore::default());

        let p_root1 = PathBuf::from("/root1");
        let p_sub1 = PathBuf::from("/root1/sub1");
        let p_sub2 = PathBuf::from("/root1/sub2");
        let p_root2 = PathBuf::from("/root2");

        let node_root1 = FolderNode {
            path: p_root1.clone(),
            name: "root1".to_string(),
            children: vec![
                FolderNode {
                    path: p_sub1.clone(),
                    name: "sub1".to_string(),
                    children: Vec::new(),
                    is_current: false,
                    is_expanded: false,
                    is_parent_nav: false,
                },
                FolderNode {
                    path: p_sub2.clone(),
                    name: "sub2".to_string(),
                    children: Vec::new(),
                    is_current: false,
                    is_expanded: false,
                    is_parent_nav: false,
                },
            ],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };

        let node_root2 = FolderNode {
            path: p_root2.clone(),
            name: "root2".to_string(),
            children: Vec::new(),
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };

        state.folder.folder_tree = vec![node_root1, node_root2];

        state.folder.select_below();
        assert_eq!(state.folder.selected_folder, Some(p_root1.clone()));

        state.folder.select_below();
        assert_eq!(state.folder.selected_folder, Some(p_root2.clone()));

        state.folder.select_above();
        assert_eq!(state.folder.selected_folder, Some(p_root1.clone()));

        state.folder.expand_selected();
        assert!(state.folder.folder_tree[0].is_expanded);

        state.folder.select_below();
        assert_eq!(state.folder.selected_folder, Some(p_sub1.clone()));

        state.folder.select_below();
        assert_eq!(state.folder.selected_folder, Some(p_sub2.clone()));

        state.folder.collapse_selected();
        assert_eq!(state.folder.selected_folder, Some(p_root1.clone()));

        state.folder.collapse_selected();
        assert!(!state.folder.folder_tree[0].is_expanded);
        assert_eq!(state.folder.selected_folder, Some(p_root1.clone()));
    }

    #[test]
    fn test_select_folder_navigation_after_expansion() {
        let mut state = AppState::new(SettingsStore::default());
        let p_root = PathBuf::from("/root");
        let p_sub = PathBuf::from("/root/sub");

        let node = FolderNode {
            path: p_root.clone(),
            name: "root".into(),
            children: vec![FolderNode {
                path: p_sub.clone(),
                name: "sub".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            }],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        state.folder.folder_tree = vec![node];

        state.folder.set_selected(p_root.clone(), 0);
        assert_eq!(state.folder.selected_folder, Some(p_root.clone()));

        state.folder.folder_tree[0].is_expanded = true;

        state.folder.select_below();
        assert_eq!(state.folder.selected_folder, Some(p_sub));
    }

    #[test]
    fn test_collapse_already_collapsed_navigates_to_visible_parent() {
        let mut state = AppState::new(SettingsStore::default());
        let p_root = PathBuf::from("/root");
        let p_sub = PathBuf::from("/root/sub");

        let node_sub = FolderNode {
            path: p_sub.clone(),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let node_root = FolderNode {
            path: p_root.clone(),
            name: "root".into(),
            children: vec![node_sub],
            is_current: false,
            is_expanded: true,
            is_parent_nav: false,
        };
        state.folder.folder_tree = vec![node_root];

        state.folder.set_selected(p_sub.clone(), 1);
        state.folder.collapse_selected();
        assert_eq!(
            state.folder.selected_folder,
            Some(p_root.clone()),
            "collapsing an already-collapsed child should navigate to its visible parent"
        );
        assert_eq!(state.folder.selected_folder_idx, Some(0));
    }

    #[test]
    fn test_collapse_already_collapsed_does_not_select_invisible_parent() {
        let mut state = AppState::new(SettingsStore::default());
        let p_root = PathBuf::from("/root");
        let p_mid = PathBuf::from("/root/mid");
        let p_sub = PathBuf::from("/root/mid/sub");

        let node_sub = FolderNode {
            path: p_sub.clone(),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let node_mid = FolderNode {
            path: p_mid.clone(),
            name: "mid".into(),
            children: vec![node_sub],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let node_root = FolderNode {
            path: p_root.clone(),
            name: "root".into(),
            children: vec![node_mid],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        state.folder.folder_tree = vec![node_root];

        state.folder.set_selected(p_sub.clone(), 2);
        state.folder.collapse_selected();
        assert_eq!(
            state.folder.selected_folder,
            Some(p_root.clone()),
            "collapsing an already-collapsed child whose parent is hidden \
             should fall back to the first visible item (the root)"
        );
        assert_eq!(state.folder.selected_folder_idx, Some(0));
        assert_ne!(state.folder.selected_folder, Some(p_mid.clone()));
        assert_ne!(state.folder.selected_folder, Some(p_sub.clone()));
    }

    #[test]
    fn test_expand_already_expanded_navigates_to_first_child() {
        let mut state = AppState::new(SettingsStore::default());
        let p_root = PathBuf::from("/root");
        let p_sub1 = PathBuf::from("/root/sub1");
        let p_sub2 = PathBuf::from("/root/sub2");

        let node_sub2 = FolderNode {
            path: p_sub2.clone(),
            name: "sub2".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let node_sub1 = FolderNode {
            path: p_sub1.clone(),
            name: "sub1".into(),
            children: vec![node_sub2],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
        };
        let node_root = FolderNode {
            path: p_root.clone(),
            name: "root".into(),
            children: vec![node_sub1],
            is_current: false,
            is_expanded: true,
            is_parent_nav: false,
        };
        state.folder.folder_tree = vec![node_root];

        state.folder.set_selected(p_root.clone(), 0);
        state.folder.expand_selected();
        assert_eq!(
            state.folder.selected_folder,
            Some(p_sub1.clone()),
            "expanding an already-expanded folder should navigate to its first child"
        );
        assert_eq!(state.folder.selected_folder_idx, Some(1));
    }

    #[test]
    fn test_keyboard_navigation_with_duplicate_paths() {
        let mut state = AppState::new(SettingsStore::default());
        let path_a = PathBuf::from("/duplicate_path");
        let path_b = PathBuf::from("/other_path");

        state.folder.folder_tree = vec![
            FolderNode {
                path: path_a.clone(),
                name: "Instance 1".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
            FolderNode {
                path: path_b.clone(),
                name: "Other".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
            FolderNode {
                path: path_a.clone(),
                name: "Instance 2".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
        ];

        state.folder.selected_folder = Some(path_a.clone());
        state.folder.selected_folder_idx = Some(2);

        state.folder.select_above();

        assert_eq!(state.folder.selected_folder_idx, Some(1));
        assert_eq!(state.folder.selected_folder, Some(path_b));
    }

    #[test]
    fn test_pinned_folder_reordering_tree_integrity() {
        let mut state = AppState::new(SettingsStore::default());
        state.folder.current_folder = Some(PathBuf::from("/current"));

        state.folder.pinned_folders = vec![
            PinnedFolder {
                path: PathBuf::from("/pinned1"),
                name: "p1".into(),
                numeric_shortcut: None,
            },
            PinnedFolder {
                path: PathBuf::from("/pinned2"),
                name: "p2".into(),
                numeric_shortcut: None,
            },
        ];

        state.build_folder_tree();
        assert_eq!(state.folder.folder_tree.len(), 3);

        state.folder.pinned_folders.swap(0, 1);
        state.build_folder_tree();

        assert_eq!(state.folder.folder_tree.len(), 3);
        assert_eq!(state.folder.folder_tree[1].path, PathBuf::from("/pinned2"));
        assert_eq!(state.folder.folder_tree[2].path, PathBuf::from("/pinned1"));
    }

    #[test]
    fn test_swap_pinned_folders() {
        let mut state = AppState::new(SettingsStore::default());
        state.folder.current_folder = Some(PathBuf::from("/current"));
        state.folder.pinned_folders = vec![
            PinnedFolder {
                path: PathBuf::from("/pinned1"),
                name: "p1".into(),
                numeric_shortcut: None,
            },
            PinnedFolder {
                path: PathBuf::from("/pinned2"),
                name: "p2".into(),
                numeric_shortcut: None,
            },
        ];
        state.build_folder_tree();
        assert_eq!(state.folder.folder_tree.len(), 3);
        assert_eq!(state.folder.folder_tree[1].path, PathBuf::from("/pinned1"));
        assert_eq!(state.folder.folder_tree[2].path, PathBuf::from("/pinned2"));

        state.swap_pinned_folders(0, 1);

        assert_eq!(
            state.folder.pinned_folders[0].path,
            PathBuf::from("/pinned2")
        );
        assert_eq!(
            state.folder.pinned_folders[1].path,
            PathBuf::from("/pinned1")
        );
        assert_eq!(state.folder.folder_tree[1].path, PathBuf::from("/pinned2"));
        assert_eq!(state.folder.folder_tree[2].path, PathBuf::from("/pinned1"));
    }

    #[test]
    fn test_build_folder_tree_preserves_parent_nav_expansion() {
        let mut state = AppState::new(SettingsStore::default());
        let root = PathBuf::from("/a/b/c");
        state.folder.current_folder = Some(root);

        let parent_nav_path = PathBuf::from("/a/b");
        state.folder.folder_tree = vec![FolderNode {
            path: PathBuf::from("/a/b/c"),
            name: "c".into(),
            children: vec![FolderNode {
                path: parent_nav_path.clone(),
                name: "b".into(),
                children: vec![],
                is_current: false,
                is_expanded: true,
                is_parent_nav: true,
            }],
            is_current: true,
            is_expanded: true,
            is_parent_nav: false,
        }];

        state.build_folder_tree();

        let children = &state.folder.folder_tree[0].children;
        let b_node = children.iter().find(|c| c.path == parent_nav_path).unwrap();
        assert!(
            b_node.is_expanded,
            "Rebuilding the folder tree collapsed an expanded parent navigation node!"
        );
    }

    #[test]
    fn test_pin_selected_folder_updates_index_alignment() {
        let mut state = AppState::new(SettingsStore::default());
        let root = PathBuf::from("/workspace");
        state.folder.current_folder = Some(root.clone());

        let target_pin = PathBuf::from("/target_pin");
        state.folder.folder_tree = vec![
            FolderNode {
                path: root,
                name: "workspace".into(),
                children: vec![],
                is_current: true,
                is_expanded: true,
                is_parent_nav: false,
            },
            FolderNode {
                path: target_pin.clone(),
                name: "target_pin".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
        ];

        state.folder.set_selected(target_pin.clone(), 1);
        state.pin_folder(&target_pin);

        assert_eq!(state.folder.pinned_folders.len(), 1);
        assert_eq!(state.folder.selected_folder, Some(target_pin));
        assert!(
            state.folder.selected_folder_idx.is_some(),
            "Pin selection action decoupled layout tracking index references!"
        );
    }

    #[test]
    fn test_detect_media_type_animate_gifs_false() {
        assert_eq!(
            detect_media_type(Path::new("test.gif"), false),
            MediaType::Image
        );
    }

    #[test]
    fn test_detect_media_type_empty_extension() {
        assert_eq!(
            detect_media_type(Path::new("test."), true),
            MediaType::Image
        );
    }

    #[test]
    fn test_select_folder_above_at_zero() {
        let mut state = AppState::new(SettingsStore::default());
        let p_a = PathBuf::from("/a");
        let p_b = PathBuf::from("/b");
        state.folder.folder_tree = vec![
            FolderNode {
                path: p_a.clone(),
                name: "a".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
            FolderNode {
                path: p_b.clone(),
                name: "b".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
        ];
        state.folder.set_selected(p_a.clone(), 0);
        assert_eq!(state.folder.selected_folder_idx, Some(0));
        state.folder.select_above();
        assert_eq!(state.folder.selected_folder_idx, Some(0));
    }

    #[test]
    fn test_select_folder_below_at_end() {
        let mut state = AppState::new(SettingsStore::default());
        let p_a = PathBuf::from("/a");
        let p_b = PathBuf::from("/b");
        state.folder.folder_tree = vec![
            FolderNode {
                path: p_a.clone(),
                name: "a".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
            FolderNode {
                path: p_b.clone(),
                name: "b".into(),
                children: vec![],
                is_current: false,
                is_expanded: false,
                is_parent_nav: false,
            },
        ];
        state.folder.set_selected(p_b.clone(), 1);
        assert_eq!(state.folder.selected_folder_idx, Some(1));
        state.folder.select_below();
        assert_eq!(state.folder.selected_folder_idx, Some(1));
    }
}
