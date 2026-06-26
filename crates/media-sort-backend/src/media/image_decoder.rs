use std::path::Path;

use image::GenericImageView;

pub fn load_image(path: &Path) -> Result<image::DynamicImage, image::ImageError> {
    image::open(path)
}

pub fn decode_image_dimensions(path: &Path) -> Result<(u32, u32), image::ImageError> {
    let img = image::open(path)?;
    Ok(img.dimensions())
}

pub fn generate_thumbnail(
    path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<image::DynamicImage, image::ImageError> {
    let img = image::open(path)?;
    Ok(img.thumbnail(max_width, max_height))
}
