use std::path::{Path, PathBuf};
use std::sync::mpsc;

use media_sort_core::media_type::MediaRegistry;
use walkdir::WalkDir;

pub fn scan_media_files(dir: &Path) -> mpsc::Receiver<PathBuf> {
    let dir = dir.to_path_buf();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let allowed = media_sort_core::media_type::SYSTEM_REGISTRY
            .get()
            .map(|r| r.allowed_extensions.clone())
            .unwrap_or_else(MediaRegistry::fallback_allowed_extensions);

        for entry in WalkDir::new(&dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| allowed.contains(&ext.to_lowercase()))
            })
        {
            if tx.send(entry.path().to_path_buf()).is_err() {
                break;
            }
        }
    });

    rx
}
