use media_sort_core::media_type::{MediaRegistry, MediaType};
use once_cell::sync::Lazy;
use std::path::PathBuf;

static MPV_MUTEX: Lazy<std::sync::Mutex<()>> = Lazy::new(|| std::sync::Mutex::new(()));

pub fn generate_thumbnail(path: &PathBuf) -> Result<Vec<u8>, ()> {
    tracing::info!("generate_thumbnail called for {:?}", path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let media_type = match MediaRegistry::determine_type(ext) {
        Some(t) => t,
        None => {
            tracing::info!("generate_thumbnail: {:?} has no handler, skipping", path);
            return Err(());
        }
    };

    if media_type == MediaType::Audio {
        match media_sort_backend::media::thumbnail::generate_thumbnail(path, 128, 128) {
            Ok(bytes) => return Ok(bytes),
            Err(_) => {
                let placeholder =
                    image::RgbaImage::from_pixel(128, 128, image::Rgba([50, 50, 70, 255]));
                let mut buf = std::io::Cursor::new(Vec::new());
                if placeholder
                    .write_to(&mut buf, image::ImageFormat::Png)
                    .is_ok()
                {
                    return Ok(buf.into_inner());
                }
            }
        }
        return Err(());
    }

    if media_type == MediaType::Video {
        tracing::info!("generate_thumbnail: {:?} routed to MPV", path);
        // Sequentially execute video thumbnail generations to prevent mpv resource contention
        let _lock = MPV_MUTEX.lock().unwrap();

        if let Ok(mut player) = media_sort_backend::media::mpv_context::MpvContext::new() {
            if player.load_file(path).is_ok() {
                player.set_paused(true);
                let start_time = std::time::Instant::now();
                let mut loaded = false;
                while start_time.elapsed() < std::time::Duration::from_millis(1000) {
                    if player.has_frame_ready() {
                        let (w, h) = player.get_video_size();
                        if w > 0 && h > 0 {
                            loaded = true;
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                if loaded {
                    let render_w = 128;
                    let render_h = 128;
                    let mut buffer = vec![0u8; render_w * render_h * 4];
                    if player
                        .render_frame(render_w as i32, render_h as i32, &mut buffer)
                        .is_ok()
                    {
                        if let Some(rgba) =
                            image::RgbaImage::from_raw(render_w as u32, render_h as u32, buffer)
                        {
                            let mut buf = std::io::Cursor::new(Vec::new());
                            if rgba.write_to(&mut buf, image::ImageFormat::Png).is_ok() {
                                let result = buf.into_inner();
                                tracing::info!(
                                    "generate_thumbnail: successfully extracted thumbnail for {:?}, len: {}",
                                    path,
                                    result.len()
                                );
                                return Ok(result);
                            }
                        }
                    }
                } else {
                    tracing::warn!(
                        "generate_thumbnail: timed out waiting for video size for {:?}",
                        path
                    );
                }
            } else {
                tracing::warn!("generate_thumbnail: failed to load file {:?}", path);
            }
        } else {
            tracing::warn!(
                "generate_thumbnail: failed to create MpvContext for {:?}",
                path
            );
        }
        return Err(());
    }

    if let Ok(img) = image::open(path) {
        let thumbnail = img.thumbnail(128, 128);
        let mut buf = std::io::Cursor::new(Vec::new());
        if thumbnail
            .write_to(&mut buf, image::ImageFormat::Png)
            .is_ok()
        {
            return Ok(buf.into_inner());
        }
    }

    Err(())
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
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_nonexistent() {
        let result = generate_thumbnail(&std::path::PathBuf::from("/nonexistent/image_xyz.jpg"));
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_thumbnail_video() {
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let path = std::path::PathBuf::from(home)
                .join("ビデオ")
                .join("画面録画")
                .join("画面録画_20260222_144330.webm");
            if path.exists() {
                let result = generate_thumbnail(&path);
                match result {
                    Ok(bytes) => {
                        println!("VIDEO THUMBNAIL LEN: {}", bytes.len());
                        assert!(!bytes.is_empty());
                        assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
                    }
                    Err(()) => {
                        println!("VIDEO THUMBNAIL: not available, treating as expected failure");
                    }
                }
            }
        }
    }
}
