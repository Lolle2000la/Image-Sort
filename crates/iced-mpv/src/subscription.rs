use crate::start_video_worker;
use crate::state::PlayerHandle;
use iced::Subscription;
use iced::futures::SinkExt;
use mpv_utils::VideoEvent;

#[derive(Clone)]
struct Handlers<FReady, FEvent> {
    on_ready: FReady,
    on_event: FEvent,
}

// iced identifies subscriptions by the `Hash` of their state. Closures are not
// hashable, so the identity is derived from the closure *types* via TypeId —
// this keeps the stream alive as long as the same closure types are passed
// across frames (the common case: function items or non-capturing closures).
impl<FReady: 'static, FEvent: 'static> std::hash::Hash for Handlers<FReady, FEvent> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }
}

/// Spawns the video worker as an iced subscription and maps its messages into
/// `Message` via the two handler closures.
///
/// - `on_ready` receives the opaque [`PlayerHandle`] once, when the worker is
///   up (deliver it to your [`crate::VideoState`] via
///   [`crate::PlayerMessage::Ready`]).
/// - `on_event` receives every raw [`VideoEvent`].
///
/// Prefer [`crate::VideoPlayer::subscription`] / `subscription_with` unless
/// you need the raw closures.
pub fn video_player_subscription<Message, FReady, FEvent>(
    on_ready: FReady,
    on_event: FEvent,
) -> Subscription<Message>
where
    Message: Send + 'static,
    FReady: Fn(PlayerHandle) -> Message + Send + Sync + 'static + Clone,
    FEvent: Fn(VideoEvent) -> Message + Send + Sync + 'static + Clone,
{
    Subscription::run_with(Handlers { on_ready, on_event }, |handlers| {
        video_stream(handlers.on_ready.clone(), handlers.on_event.clone())
    })
}

fn video_stream<Message, FReady, FEvent>(
    on_ready: FReady,
    on_event: FEvent,
) -> impl iced::futures::Stream<Item = Message>
where
    Message: Send + 'static,
    FReady: Fn(PlayerHandle) -> Message + Send + Sync + 'static,
    FEvent: Fn(VideoEvent) -> Message + Send + Sync + 'static,
{
    iced::stream::channel(
        32,
        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(64);
            let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(32);

            start_video_worker(cmd_rx, event_tx);

            if output
                .send(on_ready(PlayerHandle::new(cmd_tx)))
                .await
                .is_err()
            {
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
