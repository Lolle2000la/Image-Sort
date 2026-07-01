use std::path::Path;

use image::AnimationDecoder;
use image::GenericImageView;

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
    let img = image::open(path)?;
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
    let img = image::open(path)?;
    let img = apply_orientation(img, path);
    Ok(img.thumbnail(max_width, max_height))
}

fn apply_orientation(mut img: image::DynamicImage, path: &Path) -> image::DynamicImage {
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
