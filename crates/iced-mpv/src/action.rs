use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum VideoAction {
    PlayPause,
    Stop,
    Seek(f64),
    SetVolume(f64),
    ToggleMute,
    PlayExternally(PathBuf),
}
