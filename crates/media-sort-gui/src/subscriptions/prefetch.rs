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
