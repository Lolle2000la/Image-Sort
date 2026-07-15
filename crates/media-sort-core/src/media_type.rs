use std::collections::HashSet;
use std::sync::OnceLock;

use strum::EnumIter;

fn native_image_extensions() -> &'static [&'static str] {
    static EXTS: OnceLock<Vec<&'static str>> = OnceLock::new();
    EXTS.get_or_init(|| {
        image::ImageFormat::all()
            .filter(|f| f.can_read() && !matches!(f, image::ImageFormat::Gif))
            .flat_map(|f| f.extensions_str().iter().copied())
            .collect()
    })
}

pub static NATIVE_AUDIO_EXTS: &[&str] = &[
    "mp3", "flac", "ogg", "wav", "aac", "m4a", "wma", "opus", "aiff",
];

pub static SYSTEM_REGISTRY: OnceLock<MediaRegistry> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

impl MediaType {
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            MediaType::Image => native_image_extensions(),
            MediaType::Video => &[
                "mp4", "mkv", "webm", "avi", "mov", "wmv", "flv", "m4v", "gif",
            ],
            MediaType::Audio => NATIVE_AUDIO_EXTS,
        }
    }

    pub fn all_extensions() -> Vec<&'static str> {
        let mut exts = Vec::new();
        for ty in [Self::Image, Self::Video, Self::Audio] {
            exts.extend(ty.extensions());
        }
        exts
    }
}

pub struct MediaRegistry {
    pub allowed_extensions: HashSet<String>,
    pub mpv_extensions: HashSet<String>,
}

impl MediaRegistry {
    /// Initializes the global registry with the set of extensions discovered in the
    /// underlying `libmpv` build. Native image and audio formats are always included
    /// first so the native stack wins any overlap. Subsequent calls are ignored — the
    /// registry can only be set once per process.
    pub fn init(mpv_discovered: HashSet<String>) {
        let mut allowed = HashSet::new();

        for ext in native_image_extensions()
            .iter()
            .chain(NATIVE_AUDIO_EXTS.iter())
        {
            allowed.insert((*ext).to_string());
        }

        for ext in &mpv_discovered {
            allowed.insert(ext.clone());
        }

        let _ = SYSTEM_REGISTRY.set(MediaRegistry {
            allowed_extensions: allowed,
            mpv_extensions: mpv_discovered,
        });
    }

    /// Returns the set of extensions that would be allowed even when the global
    /// registry has not been initialized yet. Used as a safety net for the filesystem
    /// scanner and tests so behavior remains sensible if `init` was never called.
    pub fn fallback_allowed_extensions() -> HashSet<String> {
        let mut set = HashSet::new();
        for ext in native_image_extensions()
            .iter()
            .chain(NATIVE_AUDIO_EXTS.iter())
        {
            set.insert((*ext).to_string());
        }
        for ext in MediaType::Video.extensions() {
            set.insert((*ext).to_string());
        }
        set
    }

    /// Determines the media type for the given file extension using a strict priority
    /// order:
    /// 1. Native image formats
    /// 2. Native audio formats
    /// 3. Anything supported by the discovered `libmpv` build is treated as `Video`
    /// 4. Returns `None` when the extension does not match any known handler.
    pub fn determine_type(ext: &str) -> Option<MediaType> {
        let ext_lower = ext.to_lowercase();

        if native_image_extensions().contains(&ext_lower.as_str()) {
            return Some(MediaType::Image);
        }
        if NATIVE_AUDIO_EXTS.contains(&ext_lower.as_str()) {
            return Some(MediaType::Audio);
        }

        let registry = SYSTEM_REGISTRY.get()?;
        if registry.mpv_extensions.contains(&ext_lower) {
            return Some(MediaType::Video);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_type_native_image_wins_over_mpv() {
        assert_eq!(MediaRegistry::determine_type("jpg"), Some(MediaType::Image));
        assert_eq!(MediaRegistry::determine_type("PNG"), Some(MediaType::Image));
    }

    #[test]
    fn test_determine_type_native_audio_wins_over_mpv() {
        assert_eq!(MediaRegistry::determine_type("mp3"), Some(MediaType::Audio));
        assert_eq!(
            MediaRegistry::determine_type("FLAC"),
            Some(MediaType::Audio)
        );
    }

    #[test]
    fn test_determine_type_unknown_returns_none_without_registry() {
        assert_eq!(MediaRegistry::determine_type("xyz"), None);
    }

    #[test]
    fn test_fallback_allowed_extensions_includes_native() {
        let fallback = MediaRegistry::fallback_allowed_extensions();
        for ext in native_image_extensions() {
            assert!(fallback.contains(*ext), "missing native image ext {ext}");
        }
        for ext in NATIVE_AUDIO_EXTS {
            assert!(fallback.contains(*ext), "missing native audio ext {ext}");
        }
    }

    #[test]
    fn test_image_extensions() {
        let exts = MediaType::Image.extensions();
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"webp"));
    }

    #[test]
    fn test_video_extensions() {
        let exts = MediaType::Video.extensions();
        assert!(exts.contains(&"mp4"));
        assert!(exts.contains(&"mkv"));
        assert!(exts.contains(&"webm"));
        assert!(exts.contains(&"avi"));
        assert!(exts.contains(&"mov"));
        assert!(exts.contains(&"gif"));
    }

    #[test]
    fn test_audio_extensions() {
        let exts = MediaType::Audio.extensions();
        assert!(exts.contains(&"mp3"));
        assert!(exts.contains(&"flac"));
        assert!(exts.contains(&"wav"));
        assert!(exts.contains(&"ogg"));
        assert!(exts.contains(&"aac"));
        assert!(exts.contains(&"opus"));
    }

    #[test]
    fn test_all_extensions_no_duplicates() {
        let all = MediaType::all_extensions();
        let mut sorted = all.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(all.len(), sorted.len(), "duplicate extensions found");
    }

    #[test]
    fn test_all_extensions_covers_each_type() {
        let all = MediaType::all_extensions();
        let image_exts = MediaType::Image.extensions();
        let video_exts = MediaType::Video.extensions();
        let audio_exts = MediaType::Audio.extensions();

        for ext in image_exts {
            assert!(all.contains(ext), "image ext {} not in all_extensions", ext);
        }
        for ext in video_exts {
            assert!(all.contains(ext), "video ext {} not in all_extensions", ext);
        }
        for ext in audio_exts {
            assert!(all.contains(ext), "audio ext {} not in all_extensions", ext);
        }

        assert_eq!(
            all.len(),
            image_exts.len() + video_exts.len() + audio_exts.len(),
            "all_extensions count mismatch"
        );
    }

    #[test]
    fn test_gif_not_in_image_extensions() {
        let image_exts = MediaType::Image.extensions();
        assert!(
            !image_exts.contains(&"gif"),
            "GIF should not be in image extensions"
        );
    }

    #[test]
    fn test_gif_in_video_extensions() {
        let video_exts = MediaType::Video.extensions();
        assert!(
            video_exts.contains(&"gif"),
            "GIF should be in video extensions"
        );
    }

    #[test]
    fn test_determine_type_empty_extension() {
        let result = MediaRegistry::determine_type("");
        assert!(result.is_none());
    }

    #[test]
    fn test_determine_type_no_extension() {
        let result = MediaRegistry::determine_type("noextension");
        assert!(result.is_none());
    }

    #[test]
    fn test_determine_type_mixed_case() {
        assert_eq!(MediaRegistry::determine_type("JpG"), Some(MediaType::Image));
        assert_eq!(
            MediaRegistry::determine_type("FlAc"),
            Some(MediaType::Audio)
        );
    }

    #[test]
    fn test_media_registry_init_and_determine_type() {
        use std::collections::HashSet;

        let mpv_exts: HashSet<String> = ["mkv".into(), "webm".into()].into_iter().collect();
        MediaRegistry::init(mpv_exts);

        assert_eq!(MediaRegistry::determine_type("jpg"), Some(MediaType::Image));
        assert_eq!(MediaRegistry::determine_type("png"), Some(MediaType::Image));
        assert_eq!(MediaRegistry::determine_type("mp3"), Some(MediaType::Audio));
        assert_eq!(
            MediaRegistry::determine_type("flac"),
            Some(MediaType::Audio)
        );

        assert_eq!(MediaRegistry::determine_type("xyz"), None);

        let was_init_by_us = SYSTEM_REGISTRY
            .get()
            .is_some_and(|r| r.mpv_extensions.contains("mkv"));
        if was_init_by_us {
            assert_eq!(MediaRegistry::determine_type("mkv"), Some(MediaType::Video));
            assert_eq!(
                MediaRegistry::determine_type("webm"),
                Some(MediaType::Video)
            );
        }

        let novel: HashSet<String> = ["novel_init_idempotent".into()].into_iter().collect();
        MediaRegistry::init(novel);
        if was_init_by_us {
            assert_eq!(MediaRegistry::determine_type("novel_init_idempotent"), None);
        }
    }
}
