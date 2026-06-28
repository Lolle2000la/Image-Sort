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
    PickFolder,
    PickFolderResult(Option<PathBuf>),
    FolderSelected(PathBuf),
    ToggleFolderExpand(PathBuf),

    SelectEntry(usize),
    SearchQueryChanged(String),
    TriggerRename,

    MoveToFolder(PathBuf),
    DeleteEntry(PathBuf),
    RenameEntry(PathBuf, String),

    RenameInputChanged(String),
    SubmitRename,
    CancelRename,
    CreateFolderInputChanged(String),
    SubmitCreateFolder,
    CancelCreateFolder,

    Undo,
    Redo,

    PinCurrentFolder,
    PinSelectedFolder,
    UnpinCurrentFolder(PathBuf),
    TriggerCreateFolder,

    ToggleMetadataPanel,
    StartDragFolderDivider,
    MetadataLoaded(Result<BTreeMap<String, BTreeMap<String, String>>, String>),

    EditKeyBinding(usize),
    KeyCaptured(String, bool, bool, bool),

    OpenSettings,
    CloseSettings,
    ToggleDarkMode,
    ToggleReopenFolder,
    ToggleCheckForUpdates,
    ToggleInstallPrerelease,
    ToggleIntegrationWithWindows,
    ToggleAnimateGifs,
    ToggleAnimateThumbnails,
    ChangeLanguage(String),
    SaveSettings,
    RestoreDefaultKeyBindings,
    OpenCredits,
    CloseCredits,
    EventOccurred(iced::Event),
    OpenKeybindings,
    CloseKeybindings,

    PlayAudio,
    PauseAudio,
    StopAudio,

    ThumbnailReady(PathBuf, Vec<u8>),
    ImageLoaded(PathBuf, Result<(u32, u32, Vec<u8>), String>),
    GoLeft,
    GoRight,
    MoveMedia,
    PinFolderShortcut(u8),
    SearchFocused,
    SearchBlurred,
}
