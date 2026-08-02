use std::path::PathBuf;

/// User actions emitted by the crate's widgets (see
/// [`video_player_view`](crate::video_player_view)).
///
/// This is the *intent* layer: the view produces these, and either
/// [`crate::PlayerMessage::Action`] or your own message enum carries them into
/// your update loop. Feeding them back via
/// `PlayerMessage::Action(action)` makes [`crate::VideoState::update`] apply
/// them to the worker.
#[derive(Debug, Clone, PartialEq)]
pub enum VideoAction {
    /// Toggle between playing and paused.
    PlayPause,
    /// Stop playback.
    Stop,
    /// Seek to the given position in seconds.
    Seek(f64),
    /// Set the volume in percent (0–100).
    SetVolume(f64),
    /// Toggle mute.
    ToggleMute,
    /// Open the given file in an external player.
    PlayExternally(PathBuf),
}
