use std::path::PathBuf;

use crate::message::Message;

#[allow(dead_code)]
pub fn generate_thumbnail(path: &PathBuf) -> Vec<u8> {
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

#[allow(dead_code)]
pub fn prefetch_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "prefetch",
        iced::stream::channel(64, |_output| async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }),
    )
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
