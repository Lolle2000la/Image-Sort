use std::path::PathBuf;

/// User actions emitted by the crate's widgets (see
/// [`video_player_view`](crate::video_player_view)).
///
/// This is the *intent* layer: the view produces these, and either
/// [`crate::PlayerMessage::Action`] or your own message enum carries them into
/// your update loop.
#[derive(Debug, Clone, PartialEq)]
pub enum VideoAction {
    PlayPause,
    Stop,
    Seek(f64),
    SetVolume(f64),
    ToggleMute,
    /// Open the given file in an external player.
    PlayExternally(PathBuf),
}
