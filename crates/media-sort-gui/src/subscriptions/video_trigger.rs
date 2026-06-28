use iced::futures::SinkExt;
use iced::Subscription;

use crate::message::Message;

fn video_trigger_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(16, |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
        loop {
            let duration = std::time::Duration::from_millis(33);
            tokio::time::sleep(duration).await;
            let instant = std::time::Instant::now();
            if output.send(Message::Tick(instant)).await.is_err() {
                break;
            }
        }
    })
}

#[allow(dead_code)]
pub fn video_trigger_subscription() -> Subscription<Message> {
    Subscription::run(video_trigger_stream)
}
