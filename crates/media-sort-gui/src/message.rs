use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use media_sort_core::settings::store::SettingsStore;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    Tick(Instant),
    SettingsLoaded(Box<Result<SettingsStore, String>>),
    Quit,

    OpenFolder(PathBuf),
    FolderSelected(PathBuf),
    ToggleFolderExpand(PathBuf),

    SelectEntry(usize),
    SearchQueryChanged(String),

    MoveToFolder(PathBuf),
    DeleteEntry(PathBuf),
    RenameEntry(PathBuf, String),

    Undo,
    Redo,

    PinCurrentFolder,
    UnpinCurrentFolder(PathBuf),

    ToggleMetadataPanel,
    MetadataLoaded(Result<BTreeMap<String, BTreeMap<String, String>>, String>),

    EditKeyBinding(usize),
    KeyCaptured(String, bool, bool, bool),

    OpenSettings,
    CloseSettings,
    ToggleDarkMode,
    ToggleAnimateGifs,
    ToggleAnimateThumbnails,
    SaveSettings,

    PlayAudio,
    PauseAudio,
    StopAudio,

    ThumbnailReady(PathBuf, Vec<u8>),
}
