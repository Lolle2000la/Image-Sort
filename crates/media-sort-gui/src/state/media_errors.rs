use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Detailed record of a media file read or decoding failure.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaReadError {
    pub path: PathBuf,
    pub message: String,
}

impl MediaReadError {
    pub fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Specialized tracker for managing media read/decoding errors across the application.
#[derive(Debug, Clone, Default)]
pub struct MediaErrorTracker {
    errors: HashMap<PathBuf, MediaReadError>,
}

/// Upper bound on tracked entries: a folder of a million undecodable files
/// must not turn into a multi-hundred-MB in-memory map. When the cap is
/// hit, the map is cleared and only the currently recorded entry survives;
/// cards that previously showed an error badge lose it until their next
/// failed load re-records it.
const MAX_TRACKED_ERRORS: usize = 10_000;

#[allow(dead_code)]
impl MediaErrorTracker {
    /// Creates a new, empty `MediaErrorTracker`.
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Record a failure for a specific media file path along with the error explanation.
    pub fn record(&mut self, path: impl Into<PathBuf>, error: impl Into<String>) {
        let p = path.into();
        let msg = error.into();
        if self.errors.len() >= MAX_TRACKED_ERRORS && !self.errors.contains_key(&p) {
            self.errors.clear();
        }
        self.errors.insert(p.clone(), MediaReadError::new(p, msg));
    }

    /// Retrieve the detailed error message for a path, if it failed to load.
    pub fn get_error(&self, path: &Path) -> Option<&str> {
        self.errors.get(path).map(|e| e.message.as_str())
    }

    /// Retrieve the full `MediaReadError` for a path, if it failed to load.
    pub fn get_read_error(&self, path: &Path) -> Option<&MediaReadError> {
        self.errors.get(path)
    }

    /// Returns `true` if the path has a recorded error.
    pub fn has_error(&self, path: &Path) -> bool {
        self.errors.contains_key(path)
    }

    /// Removes an error entry for a path (e.g. if media reloaded successfully).
    pub fn remove(&mut self, path: &Path) {
        self.errors.remove(path);
    }

    /// Clears all recorded media read errors.
    pub fn clear(&mut self) {
        self.errors.clear();
    }

    /// Returns the number of recorded media errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns `true` if there are no recorded errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns an iterator over recorded errors as `(&PathBuf, &MediaReadError)`.
    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &MediaReadError)> {
        self.errors.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_error_tracker_record_and_get() {
        let mut tracker = MediaErrorTracker::new();
        let path = PathBuf::from("/tmp/corrupt.png");
        assert!(!tracker.has_error(&path));
        assert!(tracker.is_empty());

        tracker.record(path.clone(), "Image decoding error: Invalid PNG header");
        assert!(tracker.has_error(&path));
        assert_eq!(
            tracker.get_error(&path),
            Some("Image decoding error: Invalid PNG header")
        );
        assert_eq!(
            tracker.get_read_error(&path),
            Some(&MediaReadError::new(
                &path,
                "Image decoding error: Invalid PNG header"
            ))
        );
        assert_eq!(tracker.len(), 1);
        assert_eq!(tracker.iter().count(), 1);

        tracker.remove(&path);
        assert!(!tracker.has_error(&path));
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_media_error_tracker_clear() {
        let mut tracker = MediaErrorTracker::new();
        tracker.record("/tmp/1.jpg", "Error 1");
        tracker.record("/tmp/2.jpg", "Error 2");
        assert_eq!(tracker.len(), 2);

        tracker.clear();
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_media_error_tracker_cap_bounds_map() {
        let mut tracker = MediaErrorTracker::new();
        let paths: Vec<PathBuf> = (0..MAX_TRACKED_ERRORS + 5)
            .map(|i| PathBuf::from(format!("/tmp/bad_{i}.jpg")))
            .collect();
        for (i, p) in paths.iter().enumerate() {
            tracker.record(p.clone(), format!("Error {i}"));
        }

        // Documented behavior: once the cap is hit, the map is cleared and
        // only the entry that triggered the clear survives (plus whatever
        // was recorded after it). The first recorded paths were wiped...
        assert!(!tracker.has_error(&paths[0]));
        // ...the cap-triggering entry (index MAX_TRACKED_ERRORS) survived...
        assert!(tracker.has_error(&paths[MAX_TRACKED_ERRORS]));
        // ...and the map stays bounded — never above MAX_TRACKED_ERRORS.
        assert_eq!(tracker.len(), paths.len() - MAX_TRACKED_ERRORS);
        assert!(tracker.len() <= MAX_TRACKED_ERRORS);
        assert!(tracker.has_error(paths.last().unwrap()));
    }
}
