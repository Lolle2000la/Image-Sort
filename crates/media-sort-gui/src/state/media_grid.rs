use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use media_sort_core::models::MediaEntry;

/// Snapshot of the media grid's scrollable viewport. Updated whenever the
/// scrollable reports a new viewport via its `on_scroll` callback.
///
/// Kept in sync for diagnostic / debug purposes only; auto-scroll now uses
/// relative positions so it doesn't depend on this snapshot being current.
#[derive(Debug, Clone, Copy, Default)]
pub struct MediaGridScrollState {
    /// Current horizontal scroll offset in pixels.
    pub offset_x: f32,
    /// Width of the visible viewport in pixels.
    pub viewport_width: f32,
    /// Width of the scrollable content in pixels.
    pub content_width: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub focused: bool,
    pub placeholder: String,
}

#[derive(Default)]
pub struct MediaGridState {
    pub entries: Vec<MediaEntry>,
    pub selected_index: Option<usize>,
    pub search: SearchState,
    pub scroll: MediaGridScrollState,
    pub scan_receiver: Option<mpsc::Receiver<PathBuf>>,
    /// Index to select after the background scan completes.
    pub pending_select_index: Option<usize>,
}

impl fmt::Debug for MediaGridState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaGridState")
            .field("entries_len", &self.entries.len())
            .field("selected_index", &self.selected_index)
            .field("search", &self.search)
            .field("scroll", &self.scroll)
            .field("scan_receiver", &self.scan_receiver.is_some())
            .field("pending_select_index", &self.pending_select_index)
            .finish()
    }
}

impl MediaGridState {
    pub fn filtered_entries(&self) -> Vec<&MediaEntry> {
        if self.search.query.is_empty() {
            self.entries.iter().collect()
        } else {
            let query_lower = self.search.query.to_lowercase();
            self.entries
                .iter()
                .filter(|e| e.file_name.to_lowercase().contains(&query_lower))
                .collect()
        }
    }

    /// Synchronously scans `current_folder` for media files and populates
    /// `entries`. Clears any in-progress async scan receiver.
    pub fn scan_media(&mut self, current_folder: Option<&Path>, animate_gifs: bool) {
        self.scan_receiver = None;
        self.entries.clear();
        if let Some(folder) = current_folder {
            for p in media_sort_backend::filesystem::scanner::scan_media_files(folder) {
                let media_type = super::detect_media_type(&p, animate_gifs);
                let file_name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.display().to_string());
                self.entries.push(MediaEntry {
                    path: p,
                    media_type,
                    file_name,
                });
            }
        }
    }
}
