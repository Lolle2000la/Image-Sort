use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use media_sort_core::settings::store::SettingsStore;

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    SettingsLoaded(Box<Result<SettingsStore, String>>),
    #[allow(dead_code)]
    MediaScanCompleted(Result<Vec<media_sort_core::models::MediaEntry>, String>),
    Quit,
    EventOccurred(iced::Event),
    OpenCredits,
    CloseCredits,

    KeyCaptured(String, bool, bool, bool),

    Settings(SettingsMessage),
    Folder(FolderMessage),
    Media(MediaMessage),
    Video(VideoMessage),

    #[cfg(feature = "velopack")]
    Update(UpdateMessage),
    #[cfg(feature = "demo")]
    AutomationBounds(Option<iced::Rectangle>),
    #[cfg(feature = "demo")]
    #[allow(dead_code)]
    AutomationVirtualTick(std::time::Duration),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Open,
    Close,
    SetTheme(String),
    ToggleReopenFolder,
    #[cfg(feature = "velopack")]
    ToggleCheckForUpdates,
    #[cfg(feature = "velopack")]
    ToggleInstallPrerelease,
    #[cfg(target_os = "windows")]
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

#[derive(Debug, Clone)]
pub enum FolderMessage {
    Open(PathBuf),
    Pick,
    PickResult(Option<PathBuf>),
    PickPin,
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

#[derive(Debug, Clone)]
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
    GridScrolled(iced::widget::scrollable::AbsoluteOffset, f32, f32),
    ThumbnailReady(PathBuf, u32, u32, Vec<u8>),
    ThumbnailFailed(PathBuf),
    ThumbnailCancelled(PathBuf),
    ImageLoaded(PathBuf, Result<(u32, u32, Vec<u8>), String>),
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
