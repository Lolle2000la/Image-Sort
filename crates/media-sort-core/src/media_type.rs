use std::collections::HashSet;
use std::sync::OnceLock;

use strum::EnumIter;

pub static NATIVE_IMAGE_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "tiff", "tif", "ico", "webp", "jxl", "heic", "heif", "avif",
];
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
            MediaType::Image => NATIVE_IMAGE_EXTS,
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

        for ext in NATIVE_IMAGE_EXTS.iter().chain(NATIVE_AUDIO_EXTS.iter()) {
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
        for ext in NATIVE_IMAGE_EXTS.iter().chain(NATIVE_AUDIO_EXTS.iter()) {
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

        if NATIVE_IMAGE_EXTS.contains(&ext_lower.as_str()) {
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
        for ext in NATIVE_IMAGE_EXTS {
            assert!(fallback.contains(*ext), "missing native image ext {ext}");
        }
        for ext in NATIVE_AUDIO_EXTS {
            assert!(fallback.contains(*ext), "missing native audio ext {ext}");
        }
    }
}
