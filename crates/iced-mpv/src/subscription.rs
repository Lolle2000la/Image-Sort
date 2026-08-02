use crate::engine::worker::{VideoCommand, VideoEvent, start_video_worker};
use iced::Subscription;
use iced::futures::SinkExt;
use tokio::sync::mpsc;

#[derive(Clone, Copy)]
struct Handlers<FReady, FEvent> {
    on_ready: FReady,
    on_event: FEvent,
}

impl<FReady: 'static, FEvent: 'static> std::hash::Hash for Handlers<FReady, FEvent> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }
}

pub fn video_player_subscription<Message, FReady, FEvent>(
    on_ready: FReady,
    on_event: FEvent,
) -> Subscription<Message>
where
    Message: Send + 'static,
    FReady: Fn(mpsc::Sender<VideoCommand>) -> Message + Send + Sync + 'static + Copy,
    FEvent: Fn(VideoEvent) -> Message + Send + Sync + 'static + Copy,
{
    Subscription::run_with(Handlers { on_ready, on_event }, |handlers| {
        video_stream(handlers.on_ready, handlers.on_event)
    })
}

fn video_stream<Message, FReady, FEvent>(
    on_ready: FReady,
    on_event: FEvent,
) -> impl iced::futures::Stream<Item = Message>
where
    Message: Send + 'static,
    FReady: Fn(mpsc::Sender<VideoCommand>) -> Message + Send + Sync + 'static,
    FEvent: Fn(VideoEvent) -> Message + Send + Sync + 'static,
{
    iced::stream::channel(
        32,
        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            let (cmd_tx, cmd_rx) = mpsc::channel(64);
            let (event_tx, mut event_rx) = mpsc::channel(32);

            start_video_worker(cmd_rx, event_tx);

            if output.send(on_ready(cmd_tx)).await.is_err() {
                return;
            }

            while let Some(event) = event_rx.recv().await {
                if output.send(on_event(event)).await.is_err() {
                    break;
                }
            }
        },
    )
}
