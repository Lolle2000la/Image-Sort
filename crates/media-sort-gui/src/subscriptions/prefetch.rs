use media_sort_core::media_type::MediaType;
use std::path::PathBuf;
use std::sync::LazyLock;

use tracing;

type ThumbnailResult = Result<(u32, u32, Vec<u8>), String>;
type VideoThumbnailRequest = (PathBuf, std::sync::mpsc::Sender<ThumbnailResult>);

/// Bounded work-queue capacity per pool. A scroll-storm through a folder of
/// video files used to feed unbounded `mpsc::channel` queues (and if every
/// mpv worker failed to start, requesters blocked forever on `recv()`);
/// `sync_channel` applies backpressure and the response timeouts below
/// un-hang requesters when a pool is dead.
const THUMBNAIL_QUEUE_CAPACITY: usize = 256;
const FFMPEG_RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
const MPV_RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

static VIDEO_THUMBNAIL_WORKER: LazyLock<
    std::sync::Mutex<std::sync::mpsc::SyncSender<VideoThumbnailRequest>>,
> = LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::sync_channel::<VideoThumbnailRequest>(THUMBNAIL_QUEUE_CAPACITY);
    let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));
    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 4);

    for i in 0..num_workers {
        let rx = rx.clone();
        std::thread::spawn(move || {
            let mut player = match iced_mpv::MpvContext::new_thumbnail_player() {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Video thumbnail worker {i}: failed to create MpvContext: {e}");
                    return;
                }
            };

            loop {
                let request = {
                    let guard = rx.lock().unwrap();
                    guard.recv().ok()
                };
                let Some((path, response)) = request else {
                    break;
                };
                let result = generate_video_thumbnail_frame(&mut player, &path);
                let _ = response.send(result);
            }
        });
    }
    std::sync::Mutex::new(tx)
});

// Bounded ffmpeg worker pool — sized larger (clamp 2..16) than the mpv pool
// (clamp 2..4) because ffmpeg is a short-lived lightweight subprocess (~70 ms,
// ~40 MB RSS, no persistent state) while each MpvContext holds GPU resources
// and a libmpv handle. cap=16 lets a typical visible set (~20–30 video cards)
// render in 1–2 batches on most machines while still bounding pathological
// scroll-storm fan-out. Without this cap, scrolling through a folder of N
// video files can spawn N simultaneous ffmpeg processes (default tokio
// blocking pool ceiling is 512). See `benches/perf_candidates.rs` Group G.
static FFMPEG_THUMBNAIL_WORKER: LazyLock<
    std::sync::Mutex<std::sync::mpsc::SyncSender<VideoThumbnailRequest>>,
> = LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::sync_channel::<VideoThumbnailRequest>(THUMBNAIL_QUEUE_CAPACITY);
    let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));
    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 16);

    for i in 0..num_workers {
        let rx = rx.clone();
        std::thread::spawn(move || {
            loop {
                let request = {
                    let guard = rx.lock().unwrap();
                    guard.recv().ok()
                };
                let Some((path, response)) = request else {
                    break;
                };
                let result =
                    match media_sort_backend::media::ffmpeg_pipe::extract_frame(&path, 128, 128) {
                        Ok(decoded) => Ok(decoded.into_parts()),
                        Err(e) => Err(e),
                    };
                if response.send(result).is_err() {
                    tracing::warn!(
                        "FFMPEG thumbnail worker {i}: response channel closed for {}",
                        path.display()
                    );
                }
            }
        });
    }
    std::sync::Mutex::new(tx)
});

fn generate_video_thumbnail_frame(
    player: &mut iced_mpv::MpvContext,
    path: &std::path::Path,
) -> ThumbnailResult {
    player
        .capture_frame(path, 128, 128, std::time::Duration::from_millis(1000))
        .map_err(|e| e.to_string())
}

/// Generate a thumbnail for the given file. Takes the entry's cached
/// `media_type` and `animated` (the latter `Some(_)` for GIFs, populated
/// at scan time) so the production code path no longer re-runs
/// `detect_media_type(path, false)` here — that call internally re-opens
/// / re-decodes GIF files when the extension is `.gif` to decide whether
/// they should be reclassified as `Image`. With the cached args threaded
/// through, the GIF "is_animated" check happens exactly once per scan.
///
/// `animated` is unused at dispatch today (the `media_type` arg already
/// carries the animate_gifs-driven classification) but is threaded through
/// so future callers can use it (e.g. a metadata-panel "Animated" field)
/// without reopening `is_animated_gif`.
pub fn generate_thumbnail(
    path: &std::path::Path,
    media_type: MediaType,
    _animated: Option<bool>,
) -> ThumbnailResult {
    if media_type == MediaType::Audio {
        return media_sort_backend::media::thumbnail::generate_thumbnail(path, 128, 128)
            .map(|d| d.into_parts())
            .map_err(|e| format!("Audio cover thumbnail error: {e}"));
    }

    if media_type == MediaType::Video {
        // ffmpeg is bundled in release packages and is ~2.4× faster than
        // the mpv poll loop. Route through a bounded worker pool
        // (`FFMPEG_THUMBNAIL_WORKER`, `available_parallelism().clamp(2, 16)`)
        // so a scroll-storm can't fan out N simultaneous ffmpeg
        // subprocesses. On ffmpeg miss / extract failure, fall back to the
        // existing mpv worker pool — same semantics as before, just with a
        // bounded upper bound on concurrency.
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if FFMPEG_THUMBNAIL_WORKER
            .lock()
            .expect("FFMPEG_THUMBNAIL_WORKER lock is not poisoned")
            .send((path.to_path_buf(), response_tx))
            .is_err()
        {
            return Err("Failed to queue ffmpeg video thumbnail request".to_string());
        }
        match response_rx.recv_timeout(FFMPEG_RESPONSE_TIMEOUT) {
            Ok(Ok(parts)) => return Ok(parts),
            Ok(Err(_)) | Err(_) => {
                // ffmpeg rejected the file, timed out on it, or its pool
                // is gone (disconnected workers). The mpv pool may still
                // extract a frame, so try it before giving up.
                let (mpv_tx, mpv_rx) = std::sync::mpsc::channel();
                let sender = VIDEO_THUMBNAIL_WORKER
                    .lock()
                    .expect("VIDEO_THUMBNAIL_WORKER lock is not poisoned")
                    .clone();
                sender
                    .send((path.to_path_buf(), mpv_tx))
                    .map_err(|e| format!("Failed to queue video thumbnail request: {e}"))?;
                return mpv_rx
                    .recv_timeout(MPV_RESPONSE_TIMEOUT)
                    .map_err(|e| match e {
                        std::sync::mpsc::RecvTimeoutError::Timeout => {
                            format!(
                                "Video thumbnail request timed out after {MPV_RESPONSE_TIMEOUT:?}"
                            )
                        }
                        std::sync::mpsc::RecvTimeoutError::Disconnected => {
                            "Failed to receive video thumbnail result".to_string()
                        }
                    })
                    .map_err(|e| format!("Video thumbnail error: {e}"))?;
            }
        }
    }

    if path.extension().and_then(|e| e.to_str()) == Some("ico") {
        return generate_ico_thumbnail(path);
    }

    media_sort_backend::media::thumbnail::generate_thumbnail(path, 128, 128)
        .map(|d| d.into_parts())
        .map_err(|e| format!("Image decoding failed: {e}"))
}

fn generate_ico_thumbnail(path: &std::path::Path) -> ThumbnailResult {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open ICO file: {e}"))?;
    let icon_dir =
        ico::IconDir::read(file).map_err(|e| format!("Failed to parse ICO structure: {e}"))?;

    let entry = icon_dir
        .entries()
        .iter()
        .filter(|e| e.width() <= 128 && e.height() <= 128)
        .max_by_key(|e| e.width())
        .or_else(|| icon_dir.entries().iter().max_by_key(|e| e.width()))
        .ok_or_else(|| "No valid image entries found in ICO file".to_string())?;

    let decoded = entry
        .decode()
        .map_err(|e| format!("Failed to decode ICO entry: {e}"))?;
    let width = decoded.width();
    let height = decoded.height();
    let rgba = decoded.rgba_data().to_vec();

    Ok((width, height, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_thumbnail_valid_image() {
        let dir = std::env::temp_dir().join("mediasort_test_thumb");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");

        let img = image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]));
        img.save(&path).unwrap();

        let result = generate_thumbnail(&path, MediaType::Image, None);
        assert!(result.is_ok());
        let (w, h, rgba) = result.unwrap();
        assert!(w > 0 && h > 0);
        assert!(!rgba.is_empty());
        assert_eq!(rgba.len(), (w * h * 4) as usize);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_ico() {
        let dir = std::env::temp_dir().join("mediasort_test_ico");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ico");

        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
        icon_dir.add_entry(
            ico::IconDirEntry::encode_as_png(&ico::IconImage::from_rgba_data(
                32,
                32,
                vec![0u8; 32 * 32 * 4],
            ))
            .unwrap(),
        );
        let mut file = std::fs::File::create(&path).unwrap();
        icon_dir.write(&mut file).unwrap();

        let result = generate_thumbnail(&path, MediaType::Image, None);
        assert!(result.is_ok());
        let (w, h, rgba) = result.unwrap();
        assert_eq!((w, h), (32, 32));
        assert!(!rgba.is_empty());
        assert_eq!(rgba.len(), (32 * 32 * 4) as usize);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_nonexistent() {
        let result = generate_thumbnail(
            &std::path::PathBuf::from("/nonexistent/image_xyz.jpg"),
            MediaType::Image,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_thumbnail_exif_orientation() {
        let dir = std::env::temp_dir().join("mediasort_test_exif_thumb");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jpg");

        let img = image::RgbImage::from_pixel(100, 50, image::Rgb([255, 0, 0]));
        let mut jpeg_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut jpeg_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Jpeg).unwrap();

        let exif_app1: &[u8] = &[
            0xFF, 0xE1, // APP1 marker
            0x00, 0x22, // Length of segment (34 bytes)
            0x45, 0x78, 0x69, 0x66, 0x00, 0x00, // "Exif\0\0"
            0x49, 0x49, 0x2A, 0x00, // TIFF header "II", 0x002A
            0x08, 0x00, 0x00, 0x00, // Offset to 0th IFD (8)
            0x01, 0x00, // Number of fields (1)
            0x12, 0x01, // Tag: 0x0112 (Orientation)
            0x03, 0x00, // Type: SHORT (3)
            0x01, 0x00, 0x00, 0x00, // Count: 1
            0x06, 0x00, 0x00, 0x00, // Value: 6 (rotate 90 CW)
            0x00, 0x00, 0x00, 0x00, // Offset to next IFD (0)
        ];

        let mut oriented_jpeg = Vec::new();
        oriented_jpeg.extend_from_slice(&jpeg_bytes[..2]);
        oriented_jpeg.extend_from_slice(exif_app1);
        oriented_jpeg.extend_from_slice(&jpeg_bytes[2..]);

        std::fs::write(&path, &oriented_jpeg).unwrap();

        let result = generate_thumbnail(&path, MediaType::Image, None);
        assert!(result.is_ok());
        let (w, h, rgba) = result.unwrap();
        assert!(
            h > w,
            "Expected thumbnail height {h} > width {w} due to EXIF orientation 6 (90 deg CW)"
        );
        assert_eq!(rgba.len(), (w * h * 4) as usize);

        std::fs::remove_dir_all(&dir).ok();
    }
}
