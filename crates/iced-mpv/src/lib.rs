pub mod action;
pub mod engine;
#[cfg(feature = "ui")]
pub mod player;
pub mod state;
pub mod subscription;
pub mod widget;

pub use action::VideoAction;
pub use engine::mpv_context::MpvContext;
pub use engine::rotation::detect_video_rotation;
pub use engine::worker::{VideoCommand, VideoEvent, rotate_rgba, start_video_worker};
pub use libmpv_sys;

#[cfg(feature = "ui")]
pub use player::{PlayerMessage, VideoPlayer, VideoPlayerEvent};

pub use state::VideoState;
pub use subscription::video_player_subscription;

#[cfg(feature = "ui")]
pub use widget::controls::{MediaControlMessage, format_time, media_controls_view};
#[cfg(feature = "ui")]
pub use widget::player::video_player_view;

pub use widget::shader::{VideoPipeline, VideoPrimitive, VideoProgram, video_shader_view};

/// Raw video frame renderer element without any control UI or overlay buttons.
pub use widget::shader::video_shader_view as raw_video_view;
