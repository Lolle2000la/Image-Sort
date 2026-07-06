use std::path::Path;

use super::vips_init;

pub fn load_image_vips(path: &Path) -> Result<libvips::VipsImage, String> {
    vips_init::ensure_init();
    load_image_vips_inner(path, false)
}

fn load_image_vips_inner(path: &Path, try_audio_cover: bool) -> Result<libvips::VipsImage, String> {
    if try_audio_cover && let Some(bytes) = extract_audio_cover(path) {
        return libvips::VipsImage::new_from_buffer(&bytes, "")
            .map_err(|e| format!("vips new_from_buffer: {e}"));
    }

    let path_str = path.to_str().ok_or("Invalid path")?;
    libvips::VipsImage::new_from_file(path_str).map_err(|e| format!("vips new_from_file: {e}"))
}

pub fn vips_image_to_rgba(img: &libvips::VipsImage) -> Result<(u32, u32, Vec<u8>), String> {
    let bands = img.get_bands();
    let rgba = if bands < 4 {
        let flat = libvips::ops::addalpha(img).map_err(|e| format!("vips addalpha: {e}"))?;
        let srgb = libvips::ops::colourspace(&flat, libvips::ops::Interpretation::Srgb)
            .map_err(|e| format!("vips colourspace: {e}"))?;
        libvips::ops::rawsave_buffer(&srgb).map_err(|e| format!("vips rawsave_buffer: {e}"))?
    } else {
        libvips::ops::rawsave_buffer(img).map_err(|e| format!("vips rawsave_buffer: {e}"))?
    };

    let w = img.get_width() as u32;
    let h = img.get_height() as u32;
    Ok((w, h, rgba))
}

pub fn generate_thumbnail(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), String> {
    let img = load_image_vips_inner(path, true)?;

    let opts = libvips::ops::ThumbnailImageOptions {
        height: max_height as i32,
        size: libvips::ops::Size::Down,
        ..Default::default()
    };

    let thumb = libvips::ops::thumbnail_image_with_opts(&img, max_width as i32, &opts)
        .map_err(|e| format!("vips thumbnail_image: {e}"))?;

    vips_image_to_rgba(&thumb)
}

pub fn thumbnail_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let img = load_image_vips(path)?;
    Ok((img.get_width() as u32, img.get_height() as u32))
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
