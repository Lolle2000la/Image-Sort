//! Per-format optimized image decoding pipeline shared by the thumbnail
//! generator and the preview loader. The per-format strategies are ported
//! from the benchmark-validated variants in `crates/benchmarks/src/variants.rs`:
//!
//! - jpg/jpeg: turbojpeg scaled decompress (largest scaling factor whose
//!   scaled dimensions are still >= the target) + fast_image_resize bilinear
//!   to the exact fit. EXIF orientation is parsed from the same in-memory
//!   bytes and applied *after* the resize.
//! - bmp/tga/qoi/ff/farbfeld: image-crate decode + fast_image_resize bilinear.
//!   These formats carry no EXIF, so no orientation handling and no
//!   re-opening of the file.
//! - avif: native dav1d decode (`avif-native` feature) with an ffmpeg pipe fallback.
//! - audio extensions: embedded cover art, resized with fast_image_resize.
//! - everything else: `image_decoder::load_image` (keeps EXIF orientation for
//!   jpeg-like formats) + the image crate's `thumbnail()`.

use std::io::Cursor;
use std::path::Path;

use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use super::thumbnail::calculate_thumbnail_dimensions;

fn extension_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn is_audio_extension(ext: &str) -> bool {
    media_sort_core::media_type::MediaType::Audio
        .extensions()
        .contains(&ext)
}

fn to_image_error(err: String) -> image::ImageError {
    image::ImageError::Decoding(image::error::DecodingError::new(
        image::error::ImageFormatHint::Unknown,
        err,
    ))
}

fn io_image_error(err: std::io::Error) -> image::ImageError {
    image::ImageError::IoError(std::io::Error::new(err.kind(), err.to_string()))
}

/// Decode `path` to fit inside the `max_width` x `max_height` box, preserving
/// aspect ratio. Returns `(width, height, rgba8)`.
pub(crate) fn process_image(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let ext = extension_lower(path);

    if is_audio_extension(&ext) {
        return audio_cover_image(path, max_width, max_height);
    }

    match ext.as_str() {
        "jpg" | "jpeg" => jpeg_image(path, max_width, max_height),
        "bmp" | "tga" | "qoi" | "ff" | "farbfeld" => decode_fir_image(path, max_width, max_height),
        "avif" => avif_image(path, max_width, max_height),
        _ => fallback_image(path, max_width, max_height),
    }
}

fn audio_cover_image(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let img = match super::thumbnail::extract_audio_cover(path) {
        Some(bytes) => image::load_from_memory(&bytes)?,
        None => super::image_decoder::load_image(path)?,
    };
    resize_with_fir(&img.to_rgba8(), max_width, max_height)
}

fn decode_fir_image(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?;
    resize_with_fir(&img.to_rgba8(), max_width, max_height)
}

fn resize_with_fir(
    img_rgba: &image::RgbaImage,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let (src_w, src_h) = img_rgba.dimensions();
    let (dst_w, dst_h) = calculate_thumbnail_dimensions(src_w, src_h, max_width, max_height);

    if dst_w == 0 || dst_h == 0 {
        return Ok((src_w, src_h, img_rgba.as_raw().clone()));
    }

    let resized =
        resize_rgba(img_rgba.as_raw(), src_w, src_h, dst_w, dst_h).map_err(to_image_error)?;
    Ok((dst_w, dst_h, resized))
}

fn jpeg_image(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let bytes = std::fs::read(path).map_err(io_image_error)?;
    let orientation = parse_exif_orientation_from_bytes(&bytes);
    let (src_w, src_h, decoded) =
        decode_jpeg_turbojpeg_scaled(&bytes, max_width).map_err(to_image_error)?;
    resize_and_orient(&decoded, src_w, src_h, orientation, max_width, max_height)
        .map_err(to_image_error)
}

fn avif_image(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    // Native decode via dav1d (`avif-native` feature) is in-process and fast.
    // The ffmpeg pipe only serves as a fallback for files the native decoder
    // rejects.
    match fallback_image(path, max_width, max_height) {
        Ok(result) => Ok(result),
        Err(_) => ffmpeg_pipe(path, max_width, max_height).map_err(to_image_error),
    }
}

fn fallback_image(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let img = super::image_decoder::load_image(path)?;
    let thumbnail = img.thumbnail(max_width, max_height).to_rgba8();
    let (w, h) = thumbnail.dimensions();
    Ok((w, h, thumbnail.into_raw()))
}

// ── EXIF orientation (ported from benchmarks variants.rs) ─────────────

fn parse_exif_orientation_from_bytes(bytes: &[u8]) -> Option<u32> {
    let mut cursor = Cursor::new(bytes);
    let exif = exif::Reader::new().read_from_container(&mut cursor).ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    field.value.get_uint(0)
}

fn apply_orientation_to_rgba(w: u32, h: u32, rgba: &mut Vec<u8>, orientation: u32) -> (u32, u32) {
    match orientation {
        2 => {
            flip_horizontal_inplace(w, h, rgba);
            (w, h)
        }
        3 => {
            rotate_180_inplace(w, h, rgba);
            (w, h)
        }
        4 => {
            flip_vertical_inplace(w, h, rgba);
            (w, h)
        }
        5 => {
            flip_horizontal_inplace(w, h, rgba);
            let (nw, nh, ndata) = rotate_rgba_270_owned(w, h, rgba);
            *rgba = ndata;
            (nw, nh)
        }
        6 => {
            let (nw, nh, ndata) = rotate_rgba_90_owned(w, h, rgba);
            *rgba = ndata;
            (nw, nh)
        }
        7 => {
            flip_horizontal_inplace(w, h, rgba);
            let (nw, nh, ndata) = rotate_rgba_90_owned(w, h, rgba);
            *rgba = ndata;
            (nw, nh)
        }
        8 => {
            let (nw, nh, ndata) = rotate_rgba_270_owned(w, h, rgba);
            *rgba = ndata;
            (nw, nh)
        }
        _ => (w, h),
    }
}

fn flip_horizontal_inplace(w: u32, _h: u32, rgba: &mut [u8]) {
    let stride = (w * 4) as usize;
    for row in rgba.chunks_exact_mut(stride) {
        for x in 0..(w as usize / 2) {
            let left = x * 4;
            let right = (w as usize - 1 - x) * 4;
            for i in 0..4 {
                row.swap(left + i, right + i);
            }
        }
    }
}

fn flip_vertical_inplace(w: u32, h: u32, rgba: &mut [u8]) {
    let stride = (w * 4) as usize;
    for y in 0..(h as usize / 2) {
        let top = y * stride;
        let bottom = (h as usize - 1 - y) * stride;
        for x in 0..stride {
            rgba.swap(top + x, bottom + x);
        }
    }
}

fn rotate_180_inplace(_w: u32, _h: u32, rgba: &mut [u8]) {
    let len = rgba.len();
    for i in 0..len / 2 {
        rgba.swap(i, len - 1 - i);
    }
}

fn rotate_rgba_90_owned(w: u32, h: u32, rgba: &[u8]) -> (u32, u32, Vec<u8>) {
    let dst_w = h;
    let dst_h = w;
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    let src_stride = (w * 4) as usize;
    let dst_stride = (dst_w * 4) as usize;
    for dst_y in 0..dst_h {
        let src_x = dst_y;
        for dst_x in 0..dst_w {
            let src_y = h - 1 - dst_x;
            let src_idx = (src_y as usize * src_stride) + (src_x as usize * 4);
            let dst_idx = (dst_y as usize * dst_stride) + (dst_x as usize * 4);
            dst[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
        }
    }
    (dst_w, dst_h, dst)
}

fn rotate_rgba_270_owned(w: u32, h: u32, rgba: &[u8]) -> (u32, u32, Vec<u8>) {
    let dst_w = h;
    let dst_h = w;
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    let src_stride = (w * 4) as usize;
    let dst_stride = (dst_w * 4) as usize;
    for dst_y in 0..dst_h {
        let src_x = w - 1 - dst_y;
        for dst_x in 0..dst_w {
            let src_y = dst_x;
            let src_idx = (src_y as usize * src_stride) + (src_x as usize * 4);
            let dst_idx = (dst_y as usize * dst_stride) + (dst_x as usize * 4);
            dst[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
        }
    }
    (dst_w, dst_h, dst)
}

fn is_orientation_swap(o: u32) -> bool {
    o == 6 || o == 8 || o == 5 || o == 7
}

fn oriented_src_dims(src_w: u32, src_h: u32, orientation: Option<u32>) -> (u32, u32) {
    if let Some(o) = orientation
        && is_orientation_swap(o)
    {
        (src_h, src_w)
    } else {
        (src_w, src_h)
    }
}

fn resize_and_orient(
    src_rgba: &[u8],
    src_w: u32,
    src_h: u32,
    orientation: Option<u32>,
    max_w: u32,
    max_h: u32,
) -> Result<(u32, u32, Vec<u8>), String> {
    let (eff_w, eff_h) = oriented_src_dims(src_w, src_h, orientation);
    let (final_w, final_h) = calculate_thumbnail_dimensions(eff_w, eff_h, max_w, max_h);

    if final_w == 0 || final_h == 0 {
        return Ok((src_w, src_h, src_rgba.to_vec()));
    }

    let (resize_w, resize_h) = if let Some(o) = orientation
        && is_orientation_swap(o)
    {
        (final_h, final_w)
    } else {
        (final_w, final_h)
    };

    let mut resized = resize_rgba(src_rgba, src_w, src_h, resize_w, resize_h)?;

    if let Some(o) = orientation {
        let (fw, fh) = apply_orientation_to_rgba(resize_w, resize_h, &mut resized, o);
        Ok((fw, fh, resized))
    } else {
        Ok((resize_w, resize_h, resized))
    }
}

fn resize_rgba(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Result<Vec<u8>, String> {
    if dst_w == 0 || dst_h == 0 {
        return Ok(Vec::new());
    }
    let src_img = image::RgbaImage::from_raw(src_w, src_h, src.to_vec())
        .ok_or_else(|| "failed to create RgbaImage from raw data".to_string())?;
    let mut dst_img = Image::new(dst_w, dst_h, PixelType::U8x4);
    let mut resizer = Resizer::new();
    resizer
        .resize(
            &src_img,
            &mut dst_img,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Bilinear)),
        )
        .map_err(|e| format!("{e}"))?;
    Ok(dst_img.into_vec())
}

// ── turbojpeg scaled JPEG decompress (ported from variants.rs) ────────

fn choose_turbojpeg_scaling_factor(
    width: usize,
    height: usize,
    target_min: u32,
) -> turbojpeg::ScalingFactor {
    let t = target_min as usize;
    let candidates = [
        turbojpeg::ScalingFactor::ONE_EIGHTH,
        turbojpeg::ScalingFactor::ONE_HALF,
        turbojpeg::ScalingFactor::ONE,
    ];
    for &factor in &candidates {
        if factor.scale(width) >= t && factor.scale(height) >= t {
            return factor;
        }
    }
    turbojpeg::ScalingFactor::ONE
}

fn decode_jpeg_turbojpeg_scaled(
    bytes: &[u8],
    target_min: u32,
) -> Result<(u32, u32, Vec<u8>), String> {
    let mut decompressor =
        turbojpeg::Decompressor::new().map_err(|e| format!("turbojpeg init: {e}"))?;
    let header = decompressor
        .read_header(bytes)
        .map_err(|e| format!("turbojpeg header: {e}"))?;
    let factor = choose_turbojpeg_scaling_factor(header.width, header.height, target_min);
    let scaled = header.scaled(factor);
    decompressor
        .set_scaling_factor(factor)
        .map_err(|e| format!("turbojpeg set_scale: {e}"))?;
    let pitch = scaled.width * 4;
    let mut image = turbojpeg::Image {
        pixels: vec![0u8; scaled.height * pitch],
        width: scaled.width,
        pitch,
        height: scaled.height,
        format: turbojpeg::PixelFormat::RGBA,
    };
    decompressor
        .decompress(bytes, image.as_deref_mut())
        .map_err(|e| format!("turbojpeg decompress: {e}"))?;
    Ok((scaled.width as u32, scaled.height as u32, image.pixels))
}

// ── ffmpeg subprocess pipe (delegates to shared ffmpeg_pipe module) ──

fn ffmpeg_pipe(path: &Path, max_w: u32, max_h: u32) -> Result<(u32, u32, Vec<u8>), String> {
    super::ffmpeg_pipe::extract_frame(path, max_w, max_h)
}
