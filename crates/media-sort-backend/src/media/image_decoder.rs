use std::path::Path;

use super::{thumbnail, vips_init};

pub fn is_animated_gif(path: &Path) -> Option<bool> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() != "gif")
        .unwrap_or(true)
    {
        return None;
    }

    vips_init::ensure_init();

    let path_str = path.to_str()?;
    let img = libvips::VipsImage::new_from_file(path_str).ok()?;
    let pages = img.get_n_pages();
    Some(pages > 1)
}

pub fn load_image(path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let img = thumbnail::load_image_vips(path)?;
    thumbnail::vips_image_to_rgba(&img)
}

pub fn decode_image_dimensions(path: &Path) -> Result<(u32, u32), String> {
    thumbnail::thumbnail_dimensions(path)
}
