use std::path::Path;

use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::AnimationDecoder;
use image::GenericImageView;

use super::thumbnail::calculate_thumbnail_dimensions;

pub fn is_animated_gif(path: &Path) -> Option<bool> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() != "gif")
        .unwrap_or(true)
    {
        return None;
    }

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let reader = std::io::BufReader::new(file);
    let decoder = match image::codecs::gif::GifDecoder::new(reader) {
        Ok(d) => d,
        Err(_) => return None,
    };

    let mut frames = decoder.into_frames();
    let animated = frames.next().is_some() && frames.next().is_some();
    Some(animated)
}

pub fn load_image(path: &Path) -> Result<image::DynamicImage, image::ImageError> {
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?;
    Ok(apply_orientation(img, path))
}

pub fn decode_image_dimensions(path: &Path) -> Result<(u32, u32), image::ImageError> {
    let img = load_image(path)?;
    Ok(img.dimensions())
}

pub fn generate_thumbnail(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<image::DynamicImage, image::ImageError> {
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?;
    let img = apply_orientation(img, path);

    let img_rgba = img.to_rgba8();
    let (src_w, src_h) = img_rgba.dimensions();
    let (dst_w, dst_h) = calculate_thumbnail_dimensions(src_w, src_h, max_width, max_height);

    if dst_w == 0 || dst_h == 0 {
        return Ok(image::DynamicImage::ImageRgba8(img_rgba));
    }

    let mut dst_image = Image::new(
        dst_w,
        dst_h,
        img_rgba
            .pixel_type()
            .unwrap_or(fast_image_resize::PixelType::U8x4),
    );
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

    let buffer = dst_image.into_vec();
    Ok(image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(dst_w, dst_h, buffer).ok_or_else(|| {
            image::ImageError::Limits(image::error::LimitError::from_kind(
                image::error::LimitErrorKind::DimensionError,
            ))
        })?,
    ))
}

pub fn apply_orientation(mut img: image::DynamicImage, path: &Path) -> image::DynamicImage {
    if let Ok(file) = std::fs::File::open(path) {
        let mut buf_reader = std::io::BufReader::new(&file);
        if let Ok(exif) = exif::Reader::new().read_from_container(&mut buf_reader)
            && let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
            && let Some(val) = field.value.get_uint(0)
        {
            match val {
                2 => img = img.fliph(),
                3 => img = img.rotate180(),
                4 => img = img.flipv(),
                5 => img = img.fliph().rotate270(),
                6 => img = img.rotate90(),
                7 => img = img.fliph().rotate90(),
                8 => img = img.rotate270(),
                _ => {}
            }
        }
    }
    img
}
