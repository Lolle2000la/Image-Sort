use std::collections::HashSet;
use std::sync::OnceLock;

use strum::EnumIter;

pub static NATIVE_IMAGE_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "tiff", "tif", "ico", "webp", "jxl", "heic", "heif", "avif",
];
pub static NATIVE_AUDIO_EXTS: &[&str] = &[
    "mp3", "flac", "ogg", "wav", "aac", "m4a", "wma", "opus", "aiff",
];

static DYNAMIC_IMAGE_EXTS: OnceLock<HashSet<String>> = OnceLock::new();
static DYNAMIC_AUDIO_EXTS: OnceLock<HashSet<String>> = OnceLock::new();

pub static SYSTEM_REGISTRY: OnceLock<MediaRegistry> = OnceLock::new();

/// Override the static native image extension list with a runtime-discovered set
/// (typically from libvips' `vips_foreign_get_suffixes`). Once set, all lookups
/// use this instead of [`NATIVE_IMAGE_EXTS`].
pub fn set_native_image_extensions(exts: impl IntoIterator<Item = String>) {
    let _ = DYNAMIC_IMAGE_EXTS.set(exts.into_iter().collect());
}

/// Override the static native audio extension list.
pub fn set_native_audio_extensions(exts: impl IntoIterator<Item = String>) {
    let _ = DYNAMIC_AUDIO_EXTS.set(exts.into_iter().collect());
}

/// Returns the effective native image extensions (dynamic override wins, otherwise
/// falls back to the static list).
pub fn native_image_extensions() -> impl Iterator<Item = String> + 'static {
    DYNAMIC_IMAGE_EXTS
        .get()
        .map(|set| set.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_else(|| NATIVE_IMAGE_EXTS.iter().map(|s| s.to_string()).collect())
        .into_iter()
}

fn native_audio_extensions() -> impl Iterator<Item = String> + 'static {
    DYNAMIC_AUDIO_EXTS
        .get()
        .map(|set| set.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_else(|| NATIVE_AUDIO_EXTS.iter().map(|s| s.to_string()).collect())
        .into_iter()
}

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

        for ext in native_image_extensions().chain(native_audio_extensions()) {
            allowed.insert(ext);
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
        for ext in native_image_extensions().chain(native_audio_extensions()) {
            set.insert(ext);
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

        if native_image_extensions().any(|e| e == ext_lower) {
            return Some(MediaType::Image);
        }
        if native_audio_extensions().any(|e| e == ext_lower) {
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
