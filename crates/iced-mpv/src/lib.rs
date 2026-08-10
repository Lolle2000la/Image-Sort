//! `iced-mpv` — an [iced](https://iced.rs) (Elm Architecture) integration for
//! playing videos with [libmpv](https://mpv.io) rendered into CPU memory.
//!
//! The crate builds on [`mpv-utils`], which owns the libmpv wrapper, the
//! background worker thread and the frame/rotation utilities. This crate adds
//! the iced layer on top:
//!
//! - [`VideoPlayer::subscription`] / [`video_player_subscription`] — spawn the
//!   worker and stream its events as iced messages
//! - [`VideoState`] — the observable playback state your app embeds; all
//!   playback commands (`select`, `load`, `seek`, `volume`, ...) are methods
//!   on it
//! - [`PlayerMessage`] / [`PlayerHandle`] — the message protocol between the
//!   subscription and your update loop; the raw tokio channel is hidden behind
//!   the opaque [`PlayerHandle`]
//! - [`VideoEvent`] — domain events (`LoadFailed`, `PlayExternally`, first
//!   `FrameReady`) that need app-level handling, returned by
//!   [`VideoState::update`]
//! - `ui` feature (default): ready-made widgets — [`video_player_view`] and
//!   [`media_controls_view`]; `wgpu` feature (default): the raw
//!   [`video_shader_view`] wgpu renderer
//!
//! # Minimal usage
//!
//! ```ignore
//! // 1. subscription: spawn the worker and map its messages into yours
//! let sub = iced_mpv::VideoPlayer::subscription_with(Message::Video);
//!
//! // 2. update: feed messages into the state machine; handle domain events
//! //    (load failures, play-externally requests, first-frame arrival)
//! Message::Video(player_msg) => {
//!     if let Some(event) = state.video.update(player_msg) {
//!         match event {
//!             iced_mpv::VideoEvent::LoadFailed { path, .. } => { /* show error */ }
//!             iced_mpv::VideoEvent::PlayExternally(path) => { /* open player */ }
//!             iced_mpv::VideoEvent::FrameReady { path } => { /* playback started */ }
//!         }
//!     }
//! }
//!
//! // 3. selection: load-or-reset in one call
//! state.video.select(Some(path)); // `None` when navigating away
//!
//! // 4. view: frame + built-in controls, mapped into your message type
//! iced_mpv::video_player_view(&state.video, thumb, None, |action| {
//!     Message::Video(iced_mpv::PlayerMessage::Action(action))
//! })
//! ```

pub mod action;
pub mod player;
pub mod state;
pub mod subscription;
pub mod widget;

pub use action::VideoAction;
/// The raw worker event stream ([`mpv_utils::VideoEvent`]), re-exported under
/// a distinct name so it cannot be confused with the crate's own
/// [`VideoEvent`] domain events.
pub use mpv_utils::VideoEvent as WorkerEvent;
pub use mpv_utils::{
    MpvContext, MpvError, PlayerConfig, Rotation, VideoCommand, detect_video_rotation, rotate_rgba,
    start_video_worker, start_video_worker_with,
};

pub use player::VideoPlayer;
pub use state::{PlayerHandle, PlayerMessage, VideoEvent, VideoState};
pub use subscription::video_player_subscription;

#[cfg(feature = "ui")]
pub use widget::controls::{MediaControl, MediaControlsState, format_time, media_controls_view};
#[cfg(feature = "ui")]
pub use widget::player::video_player_view;

#[cfg(feature = "wgpu")]
pub mod gl_import;

#[cfg(feature = "wgpu")]
pub use widget::shader::{
    MpvShaderPipeline, VideoPipeline, VideoPrimitive, VideoProgram, video_shader_view,
    video_zero_copy_shader_view,
};

#[cfg(any(test, feature = "test-utils"))]
pub mod testing;
