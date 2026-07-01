use image::GenericImageView;
use std::path::Path;

pub fn generate_thumbnail(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<Vec<u8>, image::ImageError> {
    let img = match extract_audio_cover(path) {
        Some(bytes) => image::load_from_memory(&bytes)?,
        None => image::open(path)?,
    };

    let thumb = img.thumbnail(max_width, max_height);
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

pub fn thumbnail_dimensions(path: &Path) -> Result<(u32, u32), image::ImageError> {
    let img = match extract_audio_cover(path) {
        Some(bytes) => image::load_from_memory(&bytes)?,
        None => image::open(path)?,
    };
    Ok(img.dimensions())
}

pub fn extract_audio_cover(path: &Path) -> Option<Vec<u8>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "mp3" | "aiff" | "wav" => {
            let tag = id3::Tag::read_from_path(path).ok()?;
            let cover = tag.pictures().next().map(|p| p.data.clone());
            cover
        }
        "flac" => {
            let tag = metaflac::Tag::read_from_path(path).ok()?;
            let cover = tag.pictures().next().map(|p| p.data.clone());
            cover
        }
        "mp4" | "m4a" | "m4v" | "mov" | "aac" => {
            let mut file = std::fs::File::open(path).ok()?;
            let tag = mp4ameta::Tag::read_from(&mut file).ok()?;
            tag.artwork().and_then(|data| match data {
                mp4ameta::Data::Jpeg(bytes) | mp4ameta::Data::Png(bytes) => Some(bytes),
                _ => None,
            })
        }
        _ => {
            use symphonia::core::formats::FormatOptions;
            use symphonia::core::io::MediaSourceStream;
            use symphonia::core::meta::MetadataOptions;
            use symphonia::core::probe::Hint;

            let file = std::fs::File::open(path).ok()?;
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let mut hint = Hint::new();
            if !ext.is_empty() {
                hint.with_extension(ext);
            }

            let probed = symphonia::default::get_probe()
                .format(
                    &hint,
                    mss,
                    &FormatOptions::default(),
                    &MetadataOptions::default(),
                )
                .ok()?;

            let mut format = probed.format;
            let mut metadata = format.metadata();

            if let Some(revision) = metadata.current() {
                if let Some(visual) = revision.visuals().first() {
                    return Some(visual.data.to_vec());
                }
            }

            while !metadata.is_latest() {
                if let Some(revision) = metadata.pop() {
                    if let Some(visual) = revision.visuals().first() {
                        return Some(visual.data.to_vec());
                    }
                }
            }
            None
        }
    }
}
