use std::path::{Path, PathBuf};

use image::{DynamicImage, ExtendedColorType, ImageReader};
use rayon::prelude::*;

/// Bump when fixture generation parameters change so stale caches are
/// detected via the VERSION marker and regenerated (this also invalidates
/// the CI fixtures cache, which is keyed on this file's hash).
const FIXTURE_VERSION: u32 = 1;

/// All 15 formats in image 0.25 `default-formats`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchFormat {
    Avif,
    Bmp,
    Dds,
    Exr,
    Farbfeld,
    Gif,
    Hdr,
    Ico,
    Jpeg,
    Png,
    Pnm,
    Qoi,
    Tga,
    Tiff,
    WebP,
}

impl BenchFormat {
    pub const ALL: &[BenchFormat] = &[
        BenchFormat::Avif,
        BenchFormat::Bmp,
        BenchFormat::Dds,
        BenchFormat::Exr,
        BenchFormat::Farbfeld,
        BenchFormat::Gif,
        BenchFormat::Hdr,
        BenchFormat::Ico,
        BenchFormat::Jpeg,
        BenchFormat::Png,
        BenchFormat::Pnm,
        BenchFormat::Qoi,
        BenchFormat::Tga,
        BenchFormat::Tiff,
        BenchFormat::WebP,
    ];

    pub fn extension(&self) -> &str {
        match self {
            BenchFormat::Avif => "avif",
            BenchFormat::Bmp => "bmp",
            BenchFormat::Dds => "dds",
            BenchFormat::Exr => "exr",
            BenchFormat::Farbfeld => "ff",
            BenchFormat::Gif => "gif",
            BenchFormat::Hdr => "hdr",
            BenchFormat::Ico => "ico",
            BenchFormat::Jpeg => "jpg",
            BenchFormat::Png => "png",
            BenchFormat::Pnm => "pnm",
            BenchFormat::Qoi => "qoi",
            BenchFormat::Tga => "tga",
            BenchFormat::Tiff => "tiff",
            BenchFormat::WebP => "webp",
        }
    }

    pub fn file_name(&self) -> String {
        format!("mock_1.{}", self.extension())
    }

    /// Formats that the image crate CAN round-trip (encode + decode).
    pub fn is_roundtrippable(&self) -> bool {
        !matches!(
            self,
            BenchFormat::Dds // image 0.25 DDS decoder is DXT-only, not uncompressed
        )
    }
}

/// Fixtures live under the workspace `target/` dir: persistent across local
/// runs, restorable by the CI fixtures cache, and never committed.
fn fixture_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/media-sort-fixtures"
    ))
}

fn cache_valid() -> bool {
    std::fs::read_to_string(fixture_dir().join("VERSION"))
        .map(|v| v.trim().parse::<u32>() == Ok(FIXTURE_VERSION))
        .unwrap_or(false)
}

fn write_version() {
    let _ = std::fs::write(
        fixture_dir().join("VERSION"),
        format!("{FIXTURE_VERSION}\n"),
    );
}

pub fn fixture_path(fmt: BenchFormat) -> PathBuf {
    fixture_dir().join(fmt.file_name())
}

fn source_image() -> DynamicImage {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/MockState"
    ))
    .join("mock 1.jpg");
    ImageReader::open(&path)
        .expect("failed to open source image")
        .decode()
        .expect("failed to decode source image")
}

pub fn ensure_all_fixtures() -> Vec<(BenchFormat, PathBuf, DynamicImage)> {
    let dir = fixture_dir();
    if dir.exists() && !cache_valid() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    std::fs::create_dir_all(&dir).ok();

    let source = source_image();
    let rgba = source.to_rgba8();
    let (src_w, src_h) = rgba.dimensions();

    // Generate in parallel: every format is independent and the expensive
    // encoders (AVIF via ffmpeg, WebP, the float-converting EXR/HDR, ...)
    // otherwise serialize into a multi-minute single-threaded loop. Writes
    // go through a unique temp + rename so concurrent processes (nextest
    // runs one process per test) cannot corrupt a shared fixture; on a
    // cache hit nothing is written at all.
    let generated: Vec<(BenchFormat, PathBuf)> = BenchFormat::ALL
        .par_iter()
        .filter_map(|&fmt| {
            let path = fixture_path(fmt);
            if !path.exists()
                && let Err(e) = regenerate(fmt, &path, &source, (&rgba, src_w, src_h))
            {
                eprintln!("  SKIP {fmt:?}: {e}");
                return None;
            }
            Some((fmt, path))
        })
        .collect();

    if !cache_valid() {
        write_version();
    }

    // Decode check in parallel as well.
    generated
        .into_par_iter()
        .filter_map(|(fmt, path)| match decode_fixture(&path) {
            Ok(img) => Some((fmt, path, img)),
            Err(e) => {
                eprintln!("  SKIP {fmt:?} decode check failed: {e}");
                None
            }
        })
        .collect()
}

/// Generates `fmt` into a per-process temp path and atomically promotes it
/// onto `path`. Losing the promotion race is not an error: the content is
/// deterministic, so whichever process wins writes identical bytes.
fn regenerate(
    fmt: BenchFormat,
    path: &Path,
    source: &DynamicImage,
    (rgba, src_w, src_h): (&image::RgbaImage, u32, u32),
) -> Result<(), String> {
    let tmp = unique_temp_path(path);
    if let Err(e) = generate_fixture(fmt, &tmp, source, (rgba, src_w, src_h)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(_) if path.exists() => {
            let _ = std::fs::remove_file(&tmp);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(format!("promote: {e}"))
        }
    }
}

fn unique_temp_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fixture");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("tmp");
    // Keep the real extension so format-guessing writers (image::save_buffer)
    // still infer the codec from the temp path.
    path.with_file_name(format!("{stem}.{}.{ext}", std::process::id()))
}

fn generate_fixture(
    fmt: BenchFormat,
    path: &Path,
    source: &DynamicImage,
    (rgba, src_w, src_h): (&image::RgbaImage, u32, u32),
) -> Result<(), String> {
    match fmt {
        BenchFormat::Jpeg => generate_jpeg_quality92(path, rgba)?,
        BenchFormat::Png => save_as(path, source, image::ImageFormat::Png)?,
        BenchFormat::Gif => {
            let small = source.resize(1024, 1024, image::imageops::FilterType::Lanczos3);
            save_as(path, &small, image::ImageFormat::Gif)?;
        }
        BenchFormat::Bmp => save_raw(path, rgba, src_w, src_h, ExtendedColorType::Rgba8)?,
        BenchFormat::Ico => generate_ico(path, rgba, src_w, src_h)?,
        BenchFormat::Pnm => save_raw(path, rgba, src_w, src_h, ExtendedColorType::Rgba8)?,
        BenchFormat::Tga => save_raw(path, rgba, src_w, src_h, ExtendedColorType::Rgba8)?,
        BenchFormat::Tiff => save_raw(path, rgba, src_w, src_h, ExtendedColorType::Rgba8)?,
        BenchFormat::Qoi => save_raw(path, rgba, src_w, src_h, ExtendedColorType::Rgba8)?,
        BenchFormat::WebP => {
            image::save_buffer(path, rgba, src_w, src_h, ExtendedColorType::Rgba8)
                .map_err(|e| format!("webp: {e}"))?;
        }
        BenchFormat::Hdr => generate_hdr(path, rgba, src_w, src_h)?,
        BenchFormat::Exr => generate_exr(path, rgba, src_w, src_h)?,
        BenchFormat::Farbfeld => generate_farbfeld(path, rgba, src_w, src_h)?,
        BenchFormat::Avif => generate_avif_ffmpeg(path, source)?,
        BenchFormat::Dds => {
            return Err(
                "image 0.25 DDS decoder requires DXT compression; uncompressed DDS not supported"
                    .to_string(),
            );
        }
    }
    Ok(())
}

// ── Format-specific generators ────────────────────────────────────────

fn save_as(path: &Path, img: &DynamicImage, format: image::ImageFormat) -> Result<(), String> {
    img.save_with_format(path, format)
        .map_err(|e| format!("{format:?}: {e}"))
}

fn save_raw(path: &Path, data: &[u8], w: u32, h: u32, ct: ExtendedColorType) -> Result<(), String> {
    image::save_buffer(path, data, w, h, ct).map_err(|e| format!("{e}"))
}

fn generate_jpeg_quality92(path: &Path, rgba: &image::RgbaImage) -> Result<(), String> {
    // turbojpeg (SIMD, hardware-scaled) replaces image's JpegEncoder, the
    // slowest encoder in the fixture set on the 5260x3511 source image.
    let (w, h) = rgba.dimensions();
    let image = turbojpeg::Image {
        pixels: rgba.as_raw().as_slice(),
        width: w as usize,
        pitch: (w * 4) as usize,
        height: h as usize,
        format: turbojpeg::PixelFormat::RGBA,
    };
    let jpeg = turbojpeg::compress(image, 92, turbojpeg::Subsamp::Sub2x2)
        .map_err(|e| format!("turbojpeg: {e}"))?;
    std::fs::write(path, jpeg).map_err(|e| format!("write jpeg: {e}"))
}

fn generate_hdr(path: &Path, rgba: &image::RgbaImage, w: u32, h: u32) -> Result<(), String> {
    // HDR encoder needs Rgb32F (f32 RGB)
    let pixel_count = (w * h) as usize;
    let mut f32_rgb = Vec::with_capacity(pixel_count * 3);
    for pixel in rgba.pixels() {
        f32_rgb.push(pixel[0] as f32 / 255.0);
        f32_rgb.push(pixel[1] as f32 / 255.0);
        f32_rgb.push(pixel[2] as f32 / 255.0);
    }
    let bytes: &[u8] = bytemuck::cast_slice(&f32_rgb);
    image::save_buffer(path, bytes, w, h, ExtendedColorType::Rgb32F)
        .map_err(|e| format!("hdr: {e}"))
}

fn generate_exr(path: &Path, rgba: &image::RgbaImage, w: u32, h: u32) -> Result<(), String> {
    let pixel_count = (w * h) as usize;
    let mut f32_rgb = Vec::with_capacity(pixel_count * 3);
    for pixel in rgba.pixels() {
        f32_rgb.push(pixel[0] as f32 / 255.0);
        f32_rgb.push(pixel[1] as f32 / 255.0);
        f32_rgb.push(pixel[2] as f32 / 255.0);
    }
    let bytes: &[u8] = bytemuck::cast_slice(&f32_rgb);
    image::save_buffer(path, bytes, w, h, ExtendedColorType::Rgb32F)
        .map_err(|e| format!("exr: {e}"))
}

fn generate_farbfeld(path: &Path, rgba: &image::RgbaImage, w: u32, h: u32) -> Result<(), String> {
    let pixel_count = (w * h) as usize;
    let mut u16_rgba = Vec::with_capacity(pixel_count * 4);
    for pixel in rgba.pixels() {
        // farbfeld is big-endian u16; ImageEncoder expects native endian.
        // We write native-endian u16 values.
        u16_rgba.push((pixel[0] as u16) * 257);
        u16_rgba.push((pixel[1] as u16) * 257);
        u16_rgba.push((pixel[2] as u16) * 257);
        u16_rgba.push((pixel[3] as u16) * 257);
    }
    let bytes: &[u8] = bytemuck::cast_slice(&u16_rgba);
    image::save_buffer(path, bytes, w, h, ExtendedColorType::Rgba16)
        .map_err(|e| format!("farbfeld: {e}"))
}

fn generate_ico(path: &Path, rgba: &image::RgbaImage, _w: u32, _h: u32) -> Result<(), String> {
    // Downscale to 256 and encode via the ico crate
    let img = DynamicImage::ImageRgba8(rgba.clone());
    let small = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let small_rgba = small.to_rgba8();
    let (sw, sh) = small_rgba.dimensions();

    let entry = ico::IconDirEntry::encode_as_bmp(&ico::IconImage::from_rgba_data(
        sw,
        sh,
        small_rgba.into_raw(),
    ))
    .map_err(|e| format!("ico encode: {e}"))?;

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(entry);
    let mut file = std::fs::File::create(path).map_err(|e| format!("create ico: {e}"))?;
    icon_dir
        .write(&mut file)
        .map_err(|e| format!("ico write: {e}"))
}

fn has_ffmpeg() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn generate_avif_ffmpeg(path: &Path, source: &DynamicImage) -> Result<(), String> {
    if !has_ffmpeg() {
        return Err("ffmpeg not found on PATH".to_string());
    }

    // The temp PNG derives from the per-process temp path so concurrent
    // processes generating the same fixture never share an intermediate.
    let temp_png = path.with_extension(format!("png.{}", std::process::id()));
    source
        .save_with_format(&temp_png, image::ImageFormat::Png)
        .map_err(|e| format!("temp png: {e}"))?;

    let status = std::process::Command::new("ffmpeg")
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(&temp_png)
        .args([
            "-c:v",
            "libsvtav1",
            "-crf",
            "30",
            "-preset",
            "12",
            "-still-picture",
            "1",
            "-f",
            "avif",
        ])
        .arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("ffmpeg avif: {e}"))?;

    std::fs::remove_file(&temp_png).ok();

    if !status.success() {
        return Err(format!("ffmpeg avif failed with status {status}"));
    }
    Ok(())
}

pub fn decode_fixture(path: &Path) -> Result<DynamicImage, String> {
    let reader = ImageReader::open(path).map_err(|e| format!("{e}"))?;
    match reader.with_guessed_format() {
        Ok(r) => r.decode().map_err(|e| format!("{e}")),
        Err(_) => {
            let file = std::fs::read(path).map_err(|e| format!("read avif: {e}"))?;
            image::load_from_memory_with_format(&file, image::ImageFormat::Avif)
                .map_err(|e| format!("avif force: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn test_generate_and_decode_all_format_fixtures() {
        let results = ensure_all_fixtures();
        assert!(!results.is_empty(), "should have at least one fixture");
        // ensure_all_fixtures already decode-checks every fixture it returns.
        println!("Generated and verified {} fixtures", results.len());
    }

    #[test]
    fn test_jpeg_quality_92_roundtrip() {
        // Regenerate the JPEG from scratch (bypassing the shared cache) into
        // a process-private path so concurrent test processes cannot
        // interfere with the cached fixture.
        let tmp = std::env::temp_dir().join(format!(
            "mediasort_jpeg92_roundtrip_{}.jpg",
            std::process::id()
        ));
        let source = source_image();
        let rgba = source.to_rgba8();
        let (sw, sh) = rgba.dimensions();
        generate_jpeg_quality92(&tmp, &rgba).unwrap();
        let decoded = decode_fixture(&tmp).unwrap();
        assert_eq!(decoded.dimensions(), (sw, sh));
        let _ = std::fs::remove_file(&tmp);
    }
}
