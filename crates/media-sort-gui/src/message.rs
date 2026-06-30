use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use media_sort_core::settings::store::SettingsStore;

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    #[allow(dead_code)]
    SettingsLoaded(Box<Result<SettingsStore, String>>),
    Quit,
    EventOccurred(iced::Event),
    OpenCredits,
    CloseCredits,

    KeyCaptured(String, bool, bool, bool),

    Settings(SettingsMessage),
    Folder(FolderMessage),
    Media(MediaMessage),
    Video(VideoMessage),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Open,
    Close,
    ToggleDarkMode,
    ToggleReopenFolder,
    ToggleCheckForUpdates,
    ToggleInstallPrerelease,
    #[allow(dead_code)]
    ToggleIntegrationWithWindows,
    ToggleAnimateGifs,
    ToggleAnimateThumbnails,
    ChangeLanguage(String),
    Save,
    RestoreDefaultKeyBindings,
    OpenKeybindings,
    CloseKeybindings,
    EditKeyBinding(usize),
    ToggleMetadataPanel,
    StartDragFolderDivider,
    StartDragMetadataDivider,
}

#[derive(Debug, Clone)]
pub enum FolderMessage {
    Open(PathBuf),
    Pick,
    PickResult(Option<PathBuf>),
    PickPin,
    PickPinResult(Option<PathBuf>),
    Selected(PathBuf),
    ToggleExpand(PathBuf),
    PinCurrent,
    PinSelected,
    UnpinCurrent(PathBuf),
    MovePinnedUp(PathBuf),
    MovePinnedDown(PathBuf),
    PinShortcut(u8),
    TriggerCreate,
    CreateInputChanged(String),
    SubmitCreate(PathBuf),
    CancelCreate,
}

#[derive(Debug, Clone)]
pub enum MediaMessage {
    SelectEntry(usize),
    SearchQueryChanged(String),
    SearchFocused,
    #[allow(dead_code)]
    SearchBlurred,
    TriggerRename,
    RenameInputChanged(String),
    SubmitRename,
    CancelRename,
    RenameEntry(PathBuf, String),
    MoveToFolder(PathBuf),
    DeleteEntry(PathBuf),
    Undo,
    Redo,
    GoLeft,
    GoRight,
    MoveActive,
    GridScrolled(iced::widget::scrollable::AbsoluteOffset, f32, f32),
    ThumbnailReady(PathBuf, Vec<u8>),
    ThumbnailFailed(PathBuf),
    ImageLoaded(PathBuf, Result<(u32, u32, Vec<u8>), String>),
    MetadataLoaded(Result<BTreeMap<String, BTreeMap<String, String>>, String>),
    OpenExternal(PathBuf),
    PlayAudio,
    PauseAudio,
    StopAudio,
}

#[derive(Debug, Clone)]
pub enum VideoMessage {
    PlayerReady(tokio::sync::mpsc::Sender<media_sort_backend::media::mpv_context::VideoCommand>),
    Event(media_sort_backend::media::mpv_context::VideoEvent),
    Seek(f64),
    Volume(f64),
    Mute,
    PlayPause,
    Stop,
    PlayExternally(PathBuf),
}
