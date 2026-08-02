//! Test helpers for driving [`crate::VideoState`] without a running worker.
//!
//! Available in this crate's unit tests and behind the `test-utils` feature.
//! Enable the feature from consuming crates' `[dev-dependencies]` so
//! integration tests can synthesize [`crate::PlayerMessage::Ready`] and
//! assert on the commands the state sends — without ever naming the tokio
//! sender type.

use crate::state::{PlayerHandle, PlayerMessage};
use mpv_utils::VideoCommand;
use tokio::sync::mpsc;

/// Creates a [`PlayerMessage::Ready`] wired to a fresh command channel,
/// plus the receiving end so tests can assert which commands were sent.
///
/// ```ignore
/// let (msg, mut rx) = iced_mpv::testing::ready(8);
/// state.video.update(msg);
/// state.video.set_volume(50.0);
/// assert!(matches!(rx.try_recv(), Ok(VideoCommand::SetVolume(50.0))));
/// ```
pub fn ready(capacity: usize) -> (PlayerMessage, mpsc::Receiver<VideoCommand>) {
    let (tx, rx) = mpsc::channel(capacity);
    (PlayerMessage::Ready(PlayerHandle::new(tx)), rx)
}
