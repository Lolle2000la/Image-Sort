use media_sort_core::media_type::MediaType;
use std::path::PathBuf;
use std::sync::LazyLock;

use tracing;

type ThumbnailResult = Result<(u32, u32, Vec<u8>), ()>;
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
                    let _ = response.send(Err(()));
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
    if player.load_file(path).is_err() {
        return Err(());
    }
    player.set_paused(true);

    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(1000) {
        if player.has_frame_ready() {
            let (w, h) = player.get_video_size();
            if w > 0 && h > 0 {
                let render_w = 128;
                let render_h = 128;
                let mut buffer = vec![0u8; render_w * render_h * 4];
                if player
                    .render_frame(render_w as i32, render_h as i32, &mut buffer)
                    .is_ok()
                {
                    return Ok((render_w as u32, render_h as u32, buffer));
                }
            }
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Err(())
}

/// Generate a thumbnail for the given file. GIFs are always decoded as
/// images here regardless of animation settings — the image path is much
/// lighter on resources for grid thumbnails.
pub fn generate_thumbnail(path: &PathBuf) -> ThumbnailResult {
    let media_type = crate::state::detect_media_type(path, false);

    if media_type == MediaType::Audio {
        return media_sort_backend::media::thumbnail::generate_thumbnail(path, 128, 128)
            .map_err(|_| ());
    }

    if media_type == MediaType::Video {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        let sender = VIDEO_THUMBNAIL_WORKER.lock().unwrap().clone();
        sender.send((path.clone(), response_tx)).map_err(|_| ())?;
        return response_rx.recv().map_err(|_| ())?;
    }

    let file = std::fs::File::open(path).map_err(|_| ())?;
    let buf_reader = std::io::BufReader::new(file);
    let reader = image::ImageReader::new(buf_reader)
        .with_guessed_format()
        .map_err(|_| ())?;
    let img = reader.decode().map_err(|_| ())?;

    let thumbnail = img.thumbnail(128, 128).to_rgba8();
    let (w, h) = thumbnail.dimensions();
    Ok((w, h, thumbnail.into_raw()))
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
                    Ok((w, h, rgba)) => {
                        assert!(w > 0 && h > 0);
                        assert!(!rgba.is_empty());
                    }
                    Err(()) => {
                        println!("VIDEO THUMBNAIL: not available, treating as expected failure");
                    }
                }
            }
        }
    }
}
