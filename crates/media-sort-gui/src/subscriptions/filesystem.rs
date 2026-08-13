//! Filesystem watcher subscription: keeps the media grid and the folder
//! tree in sync with external changes (adds, renames, moves, removes made
//! outside the app).
//!
//! The watched set is every directory whose direct children are currently
//! displayed — the current folder (media grid) plus each expanded folder
//! tree node. The subscription identity is that set itself: iced restarts
//! the stream whenever it changes (folder switch, expand/collapse), so the
//! OS watches always mirror what is on screen. Excluded from the headless
//! demo subscription so parallel demo renders stay deterministic.

use iced::{Subscription, stream};

use crate::message::Message;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct WatchTargets(Vec<std::path::PathBuf>);

pub fn filesystem_subscription(state: &crate::state::AppState) -> Subscription<Message> {
    let dirs = state.watched_directories();
    if dirs.is_empty() {
        return Subscription::none();
    }
    let targets = WatchTargets(dirs);
    Subscription::run_with(targets, run)
}

fn run(targets: &WatchTargets) -> iced::futures::stream::BoxStream<'static, Message> {
    use iced::futures::{SinkExt, StreamExt};

    let dirs = targets.0.clone();
    stream::channel(
        256,
        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            // The handle keeps the OS watcher (and its debounce thread) alive
            // for the lifetime of the stream; dropping it on subscription
            // restart/replacement stops both.
            let (_handle, mut rx) =
                media_sort_backend::filesystem::watcher::watch_directories(&dirs);
            while let Some(event) = rx.recv().await {
                if output
                    .send(Message::FileSystemChanged(vec![event]))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        },
    )
    .boxed()
}
