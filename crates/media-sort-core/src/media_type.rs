use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

impl MediaType {
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            MediaType::Image => &[
                "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "ico", "webp", "jxl", "heic",
                "heif", "avif",
            ],
            MediaType::Video => &["mp4", "mkv", "webm", "avi", "mov", "wmv", "flv", "m4v"],
            MediaType::Audio => &[
                "mp3", "flac", "ogg", "wav", "aac", "m4a", "wma", "opus", "aiff",
            ],
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
