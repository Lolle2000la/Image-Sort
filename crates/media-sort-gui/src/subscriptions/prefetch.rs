use media_sort_core::media_type::MediaType;
use std::path::PathBuf;
use std::sync::LazyLock;

use tracing;

type ThumbnailResult = Result<(u32, u32, Vec<u8>), String>;
type ThumbnailRequest = (PathBuf, std::sync::mpsc::Sender<ThumbnailResult>);

static VIDEO_THUMBNAIL_WORKER: LazyLock<
    std::sync::Mutex<std::sync::mpsc::Sender<ThumbnailRequest>>,
> = LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::channel::<ThumbnailRequest>();
    std::thread::spawn(move || {
        let mut player = match media_sort_backend::media::mpv_context::MpvContext::new() {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Video thumbnail worker: failed to create MpvContext: {e}");
                while let Ok((_path, response)) = rx.recv() {
                    let _ = response.send(Err(format!("MpvContext initialization failed: {e}")));
                }
                return;
            }
        };

        while let Ok((path, response)) = rx.recv() {
            let result = generate_video_thumbnail_frame(&mut player, &path);
            let _ = response.send(result);
        }
    });
    std::sync::Mutex::new(tx)
});

fn generate_video_thumbnail_frame(
    player: &mut media_sort_backend::media::mpv_context::MpvContext,
    path: &std::path::Path,
) -> ThumbnailResult {
    player.stop();

    if let Err(e) = player.load_file(path) {
        return Err(format!("Failed to load video in mpv player: {e}"));
    }
    player.set_paused(true);

    let mut result = Err(format!(
        "Timed out generating video frame thumbnail for {}",
        path.display()
    ));
    let start = std::time::Instant::now();

    while start.elapsed() < std::time::Duration::from_millis(1000) {
        if player.has_frame_ready()
            && let Some(current_p_str) = player.get_current_path()
        {
            let current_p = std::path::PathBuf::from(current_p_str);
            let paths_match =
                current_p == path || current_p.canonicalize().ok() == path.canonicalize().ok();

            if paths_match {
                let (w, h) = player.get_video_size();
                if w > 0 && h > 0 {
                    let max_w = 128.0;
                    let max_h = 128.0;
                    let scale = (max_w / w as f64).min(max_h / h as f64).min(1.0);

                    let render_w = ((w as f64 * scale) as i32) & !1;
                    let render_h = ((h as f64 * scale) as i32) & !1;

                    if render_w > 0 && render_h > 0 {
                        let mut buffer = vec![0u8; (render_w * render_h * 4) as usize];
                        if player.render_frame(render_w, render_h, &mut buffer).is_ok() {
                            result = Ok((render_w as u32, render_h as u32, buffer));
                            break;
                        }
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    player.stop();
    result
}

/// Generate a thumbnail for the given file. GIFs are always decoded as
/// images here regardless of animation settings — the image path is much
/// lighter on resources for grid thumbnails.
pub fn generate_thumbnail(path: &PathBuf) -> ThumbnailResult {
    let media_type = crate::state::detect_media_type(path, false);

    if media_type == MediaType::Audio {
        return media_sort_backend::media::thumbnail::generate_thumbnail(path, 128, 128)
            .map_err(|e| format!("Audio cover thumbnail error: {e}"));
    }

    if media_type == MediaType::Video {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        let sender = VIDEO_THUMBNAIL_WORKER
            .lock()
            .expect("VIDEO_THUMBNAIL_WORKER lock is not poisoned")
            .clone();
        sender
            .send((path.clone(), response_tx))
            .map_err(|e| format!("Failed to queue video thumbnail request: {e}"))?;
        return response_rx
            .recv()
            .map_err(|e| format!("Failed to receive video thumbnail result: {e}"))?;
    }

    if path.extension().and_then(|e| e.to_str()) == Some("ico") {
        return generate_ico_thumbnail(path);
    }

    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let buf_reader = std::io::BufReader::new(file);
    let reader = image::ImageReader::new(buf_reader)
        .with_guessed_format()
        .map_err(|e| format!("Could not guess image format: {e}"))?;
    let img = reader
        .decode()
        .map_err(|e| format!("Image decoding failed: {e}"))?;

    let thumbnail = img.thumbnail(128, 128).to_rgba8();
    let (w, h) = thumbnail.dimensions();
    Ok((w, h, thumbnail.into_raw()))
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

        let result = generate_thumbnail(&path);
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

        let result = generate_thumbnail(&path);
        assert!(result.is_ok());
        let (w, h, rgba) = result.unwrap();
        assert_eq!((w, h), (32, 32));
        assert!(!rgba.is_empty());
        assert_eq!(rgba.len(), (32 * 32 * 4) as usize);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_nonexistent() {
        let result = generate_thumbnail(&std::path::PathBuf::from("/nonexistent/image_xyz.jpg"));
        assert!(result.is_err());
    }
}
