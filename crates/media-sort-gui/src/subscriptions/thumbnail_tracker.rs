use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::state::MediaGridScrollState;

pub struct ThumbnailVisibilityTracker {
    visible_paths: Arc<RwLock<HashSet<PathBuf>>>,
    debounce_deadline: Option<Instant>,
    debounce_duration: Duration,
}

impl ThumbnailVisibilityTracker {
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            visible_paths: Arc::new(RwLock::new(HashSet::new())),
            debounce_deadline: None,
            debounce_duration,
        }
    }

    pub fn clone_checker(&self) -> Arc<RwLock<HashSet<PathBuf>>> {
        self.visible_paths.clone()
    }

    pub fn handle_scroll(&mut self) {
        self.debounce_deadline = Some(Instant::now() + self.debounce_duration);
    }

    pub fn cancel_debounce(&mut self) {
        self.debounce_deadline = None;
    }

    pub fn tick(&mut self) -> bool {
        if let Some(deadline) = self.debounce_deadline
            && Instant::now() >= deadline
        {
            self.debounce_deadline = None;
            return true;
        }
        false
    }

    pub fn retain_paths<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        if let Ok(mut guard) = self.visible_paths.write() {
            for p in paths {
                guard.insert(p);
            }
        }
    }

    pub fn update_viewport(
        &mut self,
        scroll: &MediaGridScrollState,
        entry_paths: &[PathBuf],
        window_width: u32,
    ) -> Vec<PathBuf> {
        let total_items = entry_paths.len();
        let mut target_paths = HashSet::new();
        let mut load_queue = Vec::new();

        if total_items == 0 {
            if let Ok(mut guard) = self.visible_paths.write() {
                guard.clear();
            }
            return load_queue;
        }

        const CARD_STRIDE: f32 = crate::view::media_grid::MEDIA_GRID_CARD_WIDTH
            + crate::view::media_grid::MEDIA_GRID_CARD_SPACING;

        let visible_width = if scroll.viewport_width > 0.0 {
            scroll.viewport_width
        } else {
            window_width as f32
        };

        let (start_idx, end_idx) = {
            let s = (scroll.offset_x / CARD_STRIDE).floor() as usize;
            let e = ((scroll.offset_x + visible_width) / CARD_STRIDE).ceil() as usize;
            (s.saturating_sub(5), (e + 5).min(total_items))
        };

        if start_idx < total_items && start_idx < end_idx {
            for path in &entry_paths[start_idx..end_idx] {
                target_paths.insert(path.clone());
                load_queue.push(path.clone());
            }
        }

        if let Ok(mut guard) = self.visible_paths.write() {
            *guard = target_paths;
        }

        load_queue
    }
}
