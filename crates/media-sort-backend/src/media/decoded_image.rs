//! Shared decoded-image return type for the media decoding pipeline.

/// A decoded image frame: row-major RGBA8 pixels with dimensions.
///
/// `rgba.len()` is always `width * height * 4`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    /// Width of the image in pixels.
    pub width: u32,
    /// Height of the image in pixels.
    pub height: u32,
    /// Row-major RGBA8 pixel data (`width * height * 4` bytes).
    pub rgba: Vec<u8>,
}

impl DecodedImage {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Self {
        Self {
            width,
            height,
            rgba,
        }
    }

    /// `(width, height, rgba)` for call sites that need the plain parts
    /// (message plumbing, tests).
    pub fn into_parts(self) -> (u32, u32, Vec<u8>) {
        (self.width, self.height, self.rgba)
    }
}
