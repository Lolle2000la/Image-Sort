use image::GenericImageView;
use std::path::Path;

pub fn generate_thumbnail(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<Vec<u8>, image::ImageError> {
    let img = image::open(path)?;
    let thumb = img.thumbnail(max_width, max_height);
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

pub fn thumbnail_dimensions(path: &Path) -> Result<(u32, u32), image::ImageError> {
    let img = image::open(path)?;
    Ok(img.dimensions())
}
