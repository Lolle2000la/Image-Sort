use std::path::Path;

use media_sort_core::media_type::MediaRegistry;
use walkdir::WalkDir;

pub fn scan_media_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let allowed = media_sort_core::media_type::SYSTEM_REGISTRY
        .get()
        .map(|r| r.allowed_extensions.clone())
        .unwrap_or_else(MediaRegistry::fallback_allowed_extensions);

    WalkDir::new(dir)
        .max_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| allowed.contains(&ext.to_lowercase()))
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}
