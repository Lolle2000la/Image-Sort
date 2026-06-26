use std::path::{Path, PathBuf};

use image::GenericImageView;
use media_sort_backend::filesystem::scanner;
use media_sort_backend::filesystem::trash_staging::TrashStaging;
use media_sort_backend::media::image_decoder;
use media_sort_backend::media::thumbnail;
use media_sort_backend::metadata::audio_meta::extract_audio_metadata;
use media_sort_backend::metadata::image_meta::extract_image_metadata;
use media_sort_backend::metadata::video_meta::extract_video_metadata;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

#[test]
fn test_extract_exif_from_jpeg() {
    let path = fixtures_dir().join("test_image.jpg");
    let result = extract_image_metadata(&path);
    assert!(result.is_ok());
}

#[test]
fn test_extract_from_png() {
    let path = fixtures_dir().join("test_image.png");
    let result = extract_image_metadata(&path);
    assert!(result.is_ok());
}

#[test]
fn test_file_not_found() {
    let path = Path::new("/nonexistent/path/to/file.jpg");
    let result = extract_image_metadata(path);
    assert!(result.is_err());
}

#[test]
fn test_load_jpeg() {
    let path = fixtures_dir().join("test_image.jpg");
    let img = image_decoder::load_image(&path).unwrap();
    assert_eq!(img.dimensions(), (64, 64));
}

#[test]
fn test_load_png() {
    let path = fixtures_dir().join("test_image.png");
    let img = image_decoder::load_image(&path).unwrap();
    assert_eq!(img.dimensions(), (32, 32));
}

#[test]
fn test_decode_dimensions() {
    let path = fixtures_dir().join("test_image.jpg");
    let dims = image_decoder::decode_image_dimensions(&path).unwrap();
    assert_eq!(dims, (64, 64));
}

#[test]
fn test_generate_thumbnail() {
    let path = fixtures_dir().join("test_image.jpg");
    let thumb = image_decoder::generate_thumbnail(&path, 32, 32).unwrap();
    let (w, h) = thumb.dimensions();
    assert!(w <= 32, "width {w} exceeds max 32");
    assert!(h <= 32, "height {h} exceeds max 32");
}

#[test]
fn test_load_file_not_found() {
    let path = Path::new("/nonexistent/image.png");
    let result = image_decoder::load_image(path);
    assert!(result.is_err());
}

#[test]
fn test_thumbnail_dimensions() {
    let path = fixtures_dir().join("test_image.jpg");
    let dims = thumbnail::thumbnail_dimensions(&path).unwrap();
    assert_eq!(dims, (64, 64));
}

#[test]
fn test_thumbnail_generation() {
    let path = fixtures_dir().join("test_image.jpg");
    let png_bytes = thumbnail::generate_thumbnail(&path, 32, 32).unwrap();
    assert_eq!(&png_bytes[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
}

#[test]
fn test_thumbnail_respects_max() {
    let path = fixtures_dir().join("test_image.jpg");
    let png_bytes = thumbnail::generate_thumbnail(&path, 16, 16).unwrap();
    let img = image::load_from_memory(&png_bytes).unwrap();
    let (w, h) = img.dimensions();
    assert!(w <= 16, "width {w} exceeds max 16");
    assert!(h <= 16, "height {h} exceeds max 16");
}

#[test]
fn test_thumbnail_aspect_ratio() {
    let img = image::RgbImage::from_pixel(100, 50, image::Rgb([255, 0, 0]));
    let tmp_path = std::env::temp_dir().join("test_thumbnail_aspect_ratio.jpg");
    img.save_with_format(&tmp_path, image::ImageFormat::Jpeg)
        .unwrap();

    let png_bytes = thumbnail::generate_thumbnail(&tmp_path, 20, 20).unwrap();
    let thumb = image::load_from_memory(&png_bytes).unwrap();
    let (w, h) = thumb.dimensions();
    assert!(w <= 20 && h <= 20);
    assert!(
        w == 20 || h == 20,
        "Expected one dimension to hit max, got {w}x{h}"
    );

    std::fs::remove_file(&tmp_path).ok();
}

#[test]
fn test_extract_mp3_metadata() {
    let path = fixtures_dir().join("test_audio.mp3");
    let result = extract_audio_metadata(&path);
    assert!(result.is_ok());
}

#[test]
fn test_extract_flac_metadata() {
    let path = fixtures_dir().join("test_audio.flac");
    let result = extract_audio_metadata(&path);
    assert!(result.is_ok());
}

#[test]
fn test_extract_video_from_non_video() {
    let path = fixtures_dir().join("test_image.jpg");
    let result = extract_video_metadata(&path);
    assert!(result.is_ok());
}

// ============================================================
// Additional tests from audit
// ============================================================

#[test]
fn test_extract_audio_metadata_unknown_extension() {
    let dir = std::env::temp_dir().join("mediasort_audio_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.xyz");
    std::fs::write(&path, b"not audio data").unwrap();
    let result = extract_audio_metadata(&path);
    assert!(result.is_ok());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_extract_audio_metadata_nonexistent() {
    let result = extract_audio_metadata(Path::new("/nonexistent/audio_xyz.mp3"));
    assert!(result.is_err());
}

#[test]
fn test_extract_video_metadata_nonexistent() {
    let result = extract_video_metadata(Path::new("/nonexistent/video_xyz.mp4"));
    assert!(result.is_err());
}

// ============================================================
// Scanner edge cases
// ============================================================

#[test]
fn test_scan_nonexistent_directory() {
    let results = scanner::scan_media_files(Path::new("/nonexistent/dir_12345_xyz"));
    assert!(results.is_empty());
}

// ============================================================
// Image decoder error paths
// ============================================================

#[test]
fn test_decode_dimensions_nonexistent() {
    let result = image_decoder::decode_image_dimensions(Path::new("/nonexistent/img.jpg"));
    assert!(result.is_err());
}

#[test]
fn test_image_generate_thumbnail_nonexistent() {
    let result = image_decoder::generate_thumbnail(Path::new("/nonexistent/img.jpg"), 32, 32);
    assert!(result.is_err());
}

// ============================================================
// Thumbnail error paths
// ============================================================

#[test]
fn test_thumbnail_generate_nonexistent() {
    let result = thumbnail::generate_thumbnail(Path::new("/nonexistent/img.jpg"), 32, 32);
    assert!(result.is_err());
}

#[test]
fn test_thumbnail_dimensions_nonexistent() {
    let result = thumbnail::thumbnail_dimensions(Path::new("/nonexistent/img.jpg"));
    assert!(result.is_err());
}

// ============================================================
// Trash staging: stage_file with missing source
// ============================================================

#[test]
fn test_trash_stage_nonexistent_file() {
    let staging = TrashStaging::new().unwrap();
    let result = staging.stage_file(Path::new("/nonexistent/file_xyz.txt"));
    assert!(result.is_err());
}
