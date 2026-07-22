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
            guard.extend(paths);
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
            let slice = &entry_paths[start_idx..end_idx];
            target_paths.extend(slice.iter().cloned());
            load_queue.extend_from_slice(slice);
        }

        if let Ok(mut guard) = self.visible_paths.write() {
            *guard = target_paths;
        }

        load_queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_returns_false_before_deadline() {
        let mut tracker = ThumbnailVisibilityTracker::new(Duration::from_millis(500));
        tracker.handle_scroll();
        assert!(!tracker.tick(), "tick should be false before deadline");
    }

    #[test]
    fn test_tick_returns_true_after_deadline() {
        let mut tracker = ThumbnailVisibilityTracker::new(Duration::ZERO);
        tracker.handle_scroll();
        assert!(tracker.tick(), "tick should be true after deadline");
        assert!(!tracker.tick(), "second tick should be false");
    }

    #[test]
    fn test_cancel_debounce_prevents_tick() {
        let mut tracker = ThumbnailVisibilityTracker::new(Duration::ZERO);
        tracker.handle_scroll();
        tracker.cancel_debounce();
        assert!(!tracker.tick(), "tick should be false after cancel");
    }

    #[test]
    fn test_retain_paths_adds_entries() {
        let mut tracker = ThumbnailVisibilityTracker::new(Duration::from_secs(1));
        tracker.retain_paths([PathBuf::from("/a.jpg"), PathBuf::from("/b.jpg")]);

        let checker = tracker.clone_checker();
        let guard = checker.read().unwrap();
        assert!(guard.contains(&PathBuf::from("/a.jpg")));
        assert!(guard.contains(&PathBuf::from("/b.jpg")));
        assert_eq!(guard.len(), 2);
    }

    #[test]
    fn test_update_viewport_empty_entries_clears_set() {
        let mut tracker = ThumbnailVisibilityTracker::new(Duration::from_secs(1));
        let scroll = MediaGridScrollState {
            offset_x: 0.0,
            viewport_width: 200.0,
            content_width: 2000.0,
        };

        let load_queue = tracker.update_viewport(&scroll, &[], 800);
        assert!(load_queue.is_empty());

        let checker = tracker.clone_checker();
        let guard = checker.read().unwrap();
        assert!(guard.is_empty());
    }
}
