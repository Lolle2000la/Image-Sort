//! `iced-mpv` — an [iced](https://iced.rs) (Elm Architecture) integration for
//! playing videos with [libmpv](https://mpv.io) rendered into CPU memory.
//!
//! The crate builds on [`mpv-utils`], which owns the libmpv wrapper, the
//! background worker thread and the frame/rotation utilities. This crate adds
//! the iced layer on top:
//!
//! - [`video_player_subscription`] / [`VideoPlayer::subscription`] — spawn the
//!   worker and stream its events as iced messages
//! - [`VideoState`] — the observable playback state your app embeds; all
//!   playback commands (`load`, `seek`, `volume`, ...) are methods on it
//! - [`PlayerMessage`] / [`PlayerHandle`] — the message protocol between the
//!   subscription and your update loop; the raw tokio channel is hidden behind
//!   the opaque [`PlayerHandle`]
//! - `ui` feature (default): ready-made widgets — [`video_player_view`],
//!   [`media_controls_view`] and the raw [`video_shader_view`] wgpu renderer
//!
//! # Minimal usage
//!
//! ```ignore
//! // subscription
//! let sub = iced_mpv::VideoPlayer::subscription()
//!     .map(Message::VideoPlayer);
//!
//! // update: feed the messages into your VideoState
//! VideoMessage::Player(msg) => {
//!     if let Some(event) = state.video.update(msg) { /* handle */ }
//! }
//!
//! // view: draw the current frame + controls
//! iced_mpv::video_player_view(&state.video, thumb, None, |action| Message::from(action))
//! ```

pub mod action;
#[cfg(feature = "ui")]
pub mod player;
pub mod state;
pub mod subscription;
pub mod widget;

pub use action::VideoAction;
pub use mpv_utils::{
    MpvContext, MpvError, Rotation, VideoCommand, VideoEvent, detect_video_rotation, rotate_rgba,
    start_video_worker,
};

#[cfg(feature = "ui")]
pub use player::VideoPlayer;

pub use state::{PlayerHandle, PlayerMessage, VideoPlayerEvent, VideoState};
pub use subscription::video_player_subscription;

#[cfg(feature = "ui")]
pub use widget::controls::{MediaControl, format_time, media_controls_view};
#[cfg(feature = "ui")]
pub use widget::player::video_player_view;

pub use widget::shader::{VideoPipeline, VideoPrimitive, VideoProgram, video_shader_view};
