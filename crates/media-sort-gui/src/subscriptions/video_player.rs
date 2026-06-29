use iced::futures::SinkExt;
use iced::Subscription;
use tokio::sync::mpsc;
use media_sort_backend::media::mpv_context::start_video_worker;
use crate::message::Message;

pub fn video_player_subscription() -> Subscription<Message> {
    Subscription::run(video_stream)
}

fn video_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(128, |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (event_tx, mut event_rx) = mpsc::channel(128);

        start_video_worker(cmd_rx, event_tx);

        if output.send(Message::VideoPlayerReady(cmd_tx)).await.is_err() {
            return;
        }

        while let Some(event) = event_rx.recv().await {
            if output.send(Message::VideoEvent(event)).await.is_err() {
                break;
            }
        }
    })
}
