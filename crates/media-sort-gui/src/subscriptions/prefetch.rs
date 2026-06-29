use std::path::PathBuf;

use crate::message::Message;

#[allow(dead_code)]
pub fn generate_thumbnail(path: &PathBuf) -> Vec<u8> {
    let is_video = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        ext_lower == "mp4" || ext_lower == "mkv" || ext_lower == "avi" || ext_lower == "mov" || ext_lower == "webm" || ext_lower == "wmv" || ext_lower == "flv"
    } else {
        false
    };

    if is_video {
        if let Ok(mut player) = media_sort_backend::media::mpv_context::MpvContext::new() {
            if player.load_file(path).is_ok() {
                player.set_paused(true);
                let start_time = std::time::Instant::now();
                let mut loaded = false;
                while start_time.elapsed() < std::time::Duration::from_millis(500) {
                    let (w, h) = player.get_video_size();
                    if w > 0 && h > 0 {
                        loaded = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                if loaded {
                    let render_w = 128;
                    let render_h = 128;
                    let mut buffer = vec![0u8; render_w * render_h * 4];
                    if player.render_frame(render_w as i32, render_h as i32, &mut buffer).is_ok() {
                        if let Some(rgba) = image::RgbaImage::from_raw(render_w as u32, render_h as u32, buffer) {
                            let mut buf = std::io::Cursor::new(Vec::new());
                            if rgba.write_to(&mut buf, image::ImageFormat::Png).is_ok() {
                                return buf.into_inner();
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(img) = image::open(path) {
        let thumbnail = img.thumbnail(128, 128);
        let mut buf = std::io::Cursor::new(Vec::new());
        if thumbnail
            .write_to(&mut buf, image::ImageFormat::Png)
            .is_ok()
        {
            return buf.into_inner();
        }
    }
    Vec::new()
}

fn prefetch_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(64, |_output| async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    })
}

#[allow(dead_code)]
pub fn prefetch_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run(prefetch_stream)
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
        assert!(!result.is_empty());
        assert_eq!(&result[0..4], &[0x89, 0x50, 0x4E, 0x47]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_thumbnail_nonexistent() {
        let result = generate_thumbnail(&std::path::PathBuf::from("/nonexistent/image_xyz.jpg"));
        assert!(result.is_empty());
    }
}
