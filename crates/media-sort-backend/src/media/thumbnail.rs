use std::path::Path;

use super::vips_init;

pub fn load_image_vips(path: &Path) -> Result<libvips::VipsImage, String> {
    load_image_vips_inner(path, false)
}

fn load_image_vips_inner(path: &Path, try_audio_cover: bool) -> Result<libvips::VipsImage, String> {
    vips_init::ensure_init();
    if try_audio_cover && let Some(bytes) = extract_audio_cover(path) {
        return libvips::VipsImage::new_from_buffer(&bytes, "")
            .map_err(|e| format!("vips new_from_buffer: {e}"));
    }

    let path_str = path.to_str().ok_or("Invalid path")?;
    libvips::VipsImage::new_from_file(path_str).map_err(|e| format!("vips new_from_file: {e}"))
}

pub fn vips_image_to_rgba(img: &libvips::VipsImage) -> Result<(u32, u32, Vec<u8>), String> {
    let target = libvips::ops::colourspace(img, libvips::ops::Interpretation::Srgb)
        .map_err(|e| format!("vips colourspace: {e}"))?;

    let png_bytes =
        libvips::ops::pngsave_buffer(&target).map_err(|e| format!("vips pngsave: {e}"))?;

    decode_png_rgba(&png_bytes).ok_or_else(|| "png decode failed".to_string())
}

fn decode_png_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let data = &buf[..info.buffer_size()];

    let w = reader.info().width;
    let h = reader.info().height;
    let color = reader.output_color_type();

    let rgba = match color {
        (png::ColorType::Rgba, png::BitDepth::Eight) => data.to_vec(),
        (png::ColorType::Rgba, png::BitDepth::Sixteen) => {
            let mut out = Vec::with_capacity((w * h * 4) as usize);
            for chunk in data.chunks(8) {
                out.push(chunk[0]);
                out.push(chunk[2]);
                out.push(chunk[4]);
                out.push(chunk[6]);
            }
            out
        }
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            let mut out = vec![0u8; (w * h * 4) as usize];
            for (i, chunk) in data.chunks(3).enumerate() {
                let d = &mut out[i * 4..];
                d[0] = chunk[0];
                d[1] = chunk[1];
                d[2] = chunk[2];
                d[3] = 255;
            }
            out
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            let mut out = vec![0u8; (w * h * 4) as usize];
            for (i, &g) in data.iter().enumerate() {
                let d = &mut out[i * 4..];
                d[0] = g;
                d[1] = g;
                d[2] = g;
                d[3] = 255;
            }
            out
        }
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => {
            let mut out = vec![0u8; (w * h * 4) as usize];
            for (i, chunk) in data.chunks(2).enumerate() {
                let d = &mut out[i * 4..];
                d[0] = chunk[0];
                d[1] = chunk[0];
                d[2] = chunk[0];
                d[3] = chunk[1];
            }
            out
        }
        _ => return None,
    };

    Some((w, h, rgba))
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
