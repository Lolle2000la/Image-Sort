use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::GenericImageView;
use std::path::Path;

pub(crate) fn calculate_thumbnail_dimensions(
    src_w: u32,
    src_h: u32,
    max_w: u32,
    max_h: u32,
) -> (u32, u32) {
    let ratio = src_w as f64 / src_h as f64;
    let max_ratio = max_w as f64 / max_h as f64;

    if ratio > max_ratio {
        (max_w, (max_w as f64 / ratio).round() as u32)
    } else {
        ((max_h as f64 * ratio).round() as u32, max_h)
    }
}

pub fn generate_thumbnail(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let img = match extract_audio_cover(path) {
        Some(bytes) => image::load_from_memory(&bytes)?,
        None => super::image_decoder::load_image(path)?,
    };

    let img_rgba = img.to_rgba8();
    let (src_w, src_h) = img_rgba.dimensions();
    let (dst_w, dst_h) = calculate_thumbnail_dimensions(src_w, src_h, max_width, max_height);

    if dst_w == 0 || dst_h == 0 {
        return Ok((src_w, src_h, img_rgba.into_raw()));
    }

    let mut dst_image = Image::new(dst_w, dst_h, fast_image_resize::PixelType::U8x4);
    let mut resizer = Resizer::new();
    resizer
        .resize(
            &img_rgba,
            &mut dst_image,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Bilinear)),
        )
        .map_err(|e| {
            image::ImageError::Decoding(image::error::DecodingError::new(
                image::error::ImageFormatHint::Unknown,
                e.to_string(),
            ))
        })?;

    Ok((dst_w, dst_h, dst_image.into_vec()))
}

pub fn thumbnail_dimensions(path: &Path) -> Result<(u32, u32), image::ImageError> {
    let img = match extract_audio_cover(path) {
        Some(bytes) => image::load_from_memory(&bytes)?,
        None => super::image_decoder::load_image(path)?,
    };
    Ok(img.dimensions())
}

pub fn extract_audio_cover(path: &Path) -> Option<Vec<u8>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "mp3" | "aiff" | "wav" => {
            let tag = id3::Tag::read_from_path(path).ok()?;

            tag.pictures().next().map(|p| p.data.clone())
        }
        "flac" => {
            let tag = metaflac::Tag::read_from_path(path).ok()?;

            tag.pictures().next().map(|p| p.data.clone())
        }
        "mp4" | "m4a" | "m4v" | "mov" | "aac" => {
            let mut file = std::fs::File::open(path).ok()?;
            let tag = mp4ameta::Tag::read_from(&mut file).ok()?;
            tag.artwork().map(|img| img.data.to_vec())
        }
        _ => {
            use symphonia::core::formats::FormatOptions;
            use symphonia::core::formats::probe::Hint;
            use symphonia::core::io::MediaSourceStream;
            use symphonia::core::meta::MetadataOptions;

            let file = std::fs::File::open(path).ok()?;
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let mut hint = Hint::new();
            if !ext.is_empty() {
                hint.with_extension(ext);
            }

            let mut format = symphonia::default::get_probe()
                .probe(
                    &hint,
                    mss,
                    FormatOptions::default(),
                    MetadataOptions::default(),
                )
                .ok()?;

            let mut metadata = format.metadata();

            if let Some(revision) = metadata.current()
                && let Some(visual) = revision.media.visuals.first()
            {
                return Some(visual.data.to_vec());
            }

            while !metadata.is_latest() {
                if let Some(revision) = metadata.pop()
                    && let Some(visual) = revision.media.visuals.first()
                {
                    return Some(visual.data.to_vec());
                }
            }
            None
        }
    }
}
