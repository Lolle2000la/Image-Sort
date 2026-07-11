use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use media_sort_core::settings::store::SettingsStore;

#[derive(Debug, Clone, serde::Deserialize)]
pub enum Message {
    #[serde(skip_deserializing)]
    Tick(Instant),
    #[serde(skip_deserializing)]
    SettingsLoaded(Box<Result<SettingsStore, String>>),
    #[serde(skip_deserializing)]
    #[allow(dead_code)]
    MediaScanCompleted(Result<Vec<media_sort_core::models::MediaEntry>, String>),
    Quit,
    #[serde(skip_deserializing)]
    EventOccurred(iced::Event),
    OpenCredits,
    CloseCredits,

    #[serde(skip_deserializing)]
    KeyCaptured(String, bool, bool, bool),

    Settings(SettingsMessage),
    Folder(FolderMessage),
    Media(MediaMessage),
    #[serde(skip_deserializing)]
    Video(VideoMessage),

    #[cfg(feature = "velopack")]
    #[serde(skip_deserializing)]
    Update(UpdateMessage),
    #[cfg(feature = "demo")]
    #[serde(skip_deserializing)]
    AutomationBounds(Option<iced::Rectangle>),
    #[cfg(feature = "demo")]
    #[serde(skip_deserializing)]
    AutomationVirtualTick(std::time::Duration),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum SettingsMessage {
    Open,
    Close,
    SetTheme(String),
    ToggleReopenFolder,
    #[cfg(feature = "velopack")]
    #[serde(skip_deserializing)]
    ToggleCheckForUpdates,
    #[cfg(feature = "velopack")]
    #[serde(skip_deserializing)]
    ToggleInstallPrerelease,
    #[cfg(target_os = "windows")]
    #[serde(skip_deserializing)]
    ToggleIntegrationWithWindows,
    ToggleAnimateGifs,
    ChangeLanguage(String),
    Save,
    RestoreDefaultKeyBindings,
    OpenKeybindings,
    EditKeyBinding(usize),
    ToggleMetadataPanel,
    StartDragFolderDivider,
    StartDragMetadataDivider,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum FolderMessage {
    Open(PathBuf),
    Pick,
    #[serde(skip_deserializing)]
    PickResult(Option<PathBuf>),
    PickPin,
    #[serde(skip_deserializing)]
    PickPinResult(Option<PathBuf>),
    Selected(PathBuf, usize),
    ToggleExpand(PathBuf),
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

#[derive(Debug, Clone, serde::Deserialize)]
pub enum MediaMessage {
    SelectEntry(usize),
    SearchQueryChanged(String),
    SearchFocused,
    SearchBlurred,
    TriggerRename,
    RenameInputChanged(String),
    SubmitRename,
    CancelRename,
    RenameEntry(PathBuf, String),
    MoveToFolder(PathBuf),
    CopyToFolder(PathBuf),
    DeleteEntry(PathBuf),
    Undo,
    Redo,
    GoLeft,
    GoRight,
    MoveActive,
    CopyActive,
    #[serde(skip_deserializing)]
    GridScrolled(iced::widget::scrollable::AbsoluteOffset, f32, f32),
    #[serde(skip_deserializing)]
    ThumbnailReady(PathBuf, u32, u32, Vec<u8>),
    #[serde(skip_deserializing)]
    ThumbnailFailed(PathBuf),
    #[serde(skip_deserializing)]
    ThumbnailCancelled(PathBuf),
    #[serde(skip_deserializing)]
    ImageLoaded(PathBuf, Result<(u32, u32, Vec<u8>), String>),
    #[serde(skip_deserializing)]
    MetadataLoaded(Result<BTreeMap<String, BTreeMap<String, String>>, String>),
    OpenExternal(PathBuf),
    StopAudio,
    AudioSeek(f64),
    AudioSetVolume(f64),
    AudioToggleMute,
    AudioPlayPause,
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

#[cfg(feature = "velopack")]
#[derive(Debug, Clone)]
pub enum UpdateMessage {
    CheckForUpdates,
    UpdateAvailable(Box<velopack::UpdateInfo>),
    NoUpdateFound,
    UserConfirmedUpdate(Box<velopack::UpdateInfo>),
    UpdateFailed(String),
    DismissUpdatePrompt,
}
