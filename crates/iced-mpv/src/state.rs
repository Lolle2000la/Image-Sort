use mpv_utils::{Rotation, VideoCommand, VideoEvent};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// An opaque handle to the video worker's command channel.
///
/// The subscription hands this to your app via
/// [`PlayerMessage::Ready`]; [`VideoState`] stores it so playback commands can
/// be sent. The underlying tokio channel is not exposed — to send commands,
/// call the [`VideoState`] methods (e.g. `state.video.seek(10.0)`).
#[derive(Debug, Clone)]
pub struct PlayerHandle {
    sender: tokio::sync::mpsc::Sender<VideoCommand>,
}

impl PlayerHandle {
    /// Wraps a raw command channel sender.
    ///
    /// In normal use the subscription constructs this for you. This
    /// constructor exists for tests that want to drive [`VideoState`] without
    /// a running worker.
    pub fn new(sender: tokio::sync::mpsc::Sender<VideoCommand>) -> Self {
        Self { sender }
    }

    pub(crate) fn sender(&self) -> &tokio::sync::mpsc::Sender<VideoCommand> {
        &self.sender
    }
}

/// Messages produced by the [`video_player_subscription`](crate::video_player_subscription)
/// and consumed by [`VideoState::update`].
#[derive(Debug, Clone)]
pub enum PlayerMessage {
    /// The worker is up; carry the opaque command [`PlayerHandle`].
    Ready(PlayerHandle),
    /// A raw worker event (rendered frame, progress, ...).
    Event(VideoEvent),
    /// An action originating from a widget (see [`VideoAction`](crate::VideoAction)).
    Action(crate::VideoAction),
}

/// Domain events that need application-level handling, produced by
/// [`VideoState::update`] (as opposed to [`VideoEvent`], which is the raw
/// worker event stream).
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPlayerEvent {
    /// Loading a file failed. `error` is a human-readable description.
    LoadFailed { path: PathBuf, error: String },
    /// The user asked to open the video in an external player.
    PlayExternally(PathBuf),
}

/// The observable video playback state and command interface.
///
/// Embed this in your application state (e.g. `AppState { video: VideoState }`)
/// and feed it every [`PlayerMessage`] from the subscription via
/// [`VideoState::update`]. All playback commands are fire-and-forget methods
/// that send [`VideoCommand`]s to the worker; when no worker is connected
/// (yet), they are logged and dropped — the state machine stays consistent.
///
/// `VideoState` sends a `Deactivate` command when dropped, so playback stops
/// automatically on teardown.
#[derive(Debug, Clone)]
pub struct VideoState {
    sender: Option<PlayerHandle>,
    rgba: Option<Arc<Vec<u8>>>,
    width: u32,
    height: u32,
    rotation: Rotation,
    position: f64,
    duration: f64,
    volume: f64,
    muted: bool,
    paused: bool,
    ready: bool,
    seek_position: Option<f64>,
    last_seek_time: Option<Instant>,
    selected_path: Option<PathBuf>,
}

impl Default for VideoState {
    fn default() -> Self {
        Self {
            sender: None,
            rgba: None,
            width: 0,
            height: 0,
            rotation: Rotation::R0,
            position: 0.0,
            duration: 0.0,
            volume: 100.0,
            muted: false,
            paused: false,
            ready: false,
            seek_position: None,
            last_seek_time: None,
            selected_path: None,
        }
    }
}

impl VideoState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` once the subscription has delivered its
    /// [`PlayerMessage::Ready`] handle, i.e. commands will reach a worker.
    pub fn is_connected(&self) -> bool {
        self.sender.is_some()
    }

    /// The last rendered frame as raw RGBA, if any.
    pub fn rgba(&self) -> Option<&Arc<Vec<u8>>> {
        self.rgba.as_ref()
    }

    /// The width of the last rendered frame (unrotated).
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of the last rendered frame (unrotated).
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The rotation of the last rendered frame.
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    /// The playback position in seconds.
    pub fn position(&self) -> f64 {
        self.position
    }

    /// A pending seek target in seconds, if the user is currently dragging
    /// the seekbar (throttled seeks are not yet applied to mpv).
    pub fn seek_position(&self) -> Option<f64> {
        self.seek_position
    }

    /// The media duration in seconds (0 until known).
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// The volume in percent (0–100).
    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Whether the worker has produced at least one progress event for the
    /// currently selected file (frames are only committed after this).
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// The path of the currently selected (requested) video.
    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.selected_path.as_ref()
    }

    fn send(&self, cmd: VideoCommand) {
        match self.sender.as_ref() {
            Some(handle) => {
                if let Err(e) = handle.sender().try_send(cmd) {
                    tracing::debug!("video command dropped, worker unreachable: {e:?}");
                }
            }
            None => {
                tracing::debug!("video command dropped, no worker connected");
            }
        }
    }

    /// Feeds one [`PlayerMessage`] from the subscription into the state
    /// machine. Returns a [`VideoPlayerEvent`] when the app needs to react
    /// (load failure, play-externally request).
    pub fn update(&mut self, message: PlayerMessage) -> Option<VideoPlayerEvent> {
        match message {
            PlayerMessage::Ready(handle) => {
                self.sender = Some(handle);
                if let Some(path) = &self.selected_path {
                    self.load(path.clone());
                }
                None
            }
            PlayerMessage::Event(event) => self.handle_event(&event),
            PlayerMessage::Action(action) => match action {
                crate::VideoAction::PlayPause => {
                    self.toggle_pause();
                    None
                }
                crate::VideoAction::Stop => {
                    self.stop();
                    None
                }
                crate::VideoAction::Seek(pos) => {
                    self.seek(pos);
                    None
                }
                crate::VideoAction::SetVolume(vol) => {
                    self.set_volume(vol);
                    None
                }
                crate::VideoAction::ToggleMute => {
                    self.toggle_mute();
                    None
                }
                crate::VideoAction::PlayExternally(path) => {
                    Some(VideoPlayerEvent::PlayExternally(path))
                }
            },
        }
    }

    fn handle_event(&mut self, event: &VideoEvent) -> Option<VideoPlayerEvent> {
        match event {
            VideoEvent::FrameReady {
                path,
                width,
                height,
                rotation,
                rgba,
            } => {
                // Only commit frames that belong to the currently selected
                // video, and only after the worker confirmed readiness. This
                // guards against a stale frame from a previously selected
                // video repopulating the state after a fast selection change.
                if self.selected_path.as_deref() == Some(path.as_path()) && self.ready {
                    self.rgba = Some(rgba.clone());
                    self.width = *width;
                    self.height = *height;
                    self.rotation = *rotation;
                }
                None
            }
            VideoEvent::PlaybackProgress { position, duration } => {
                self.position = *position;
                self.duration = *duration;
                self.ready = true;
                None
            }
            VideoEvent::Muted(muted) => {
                self.muted = *muted;
                None
            }
            VideoEvent::Volume(vol) => {
                self.volume = *vol;
                None
            }
            VideoEvent::Paused(paused) => {
                self.paused = *paused;
                None
            }
            VideoEvent::LoadFailed { path, error } => Some(VideoPlayerEvent::LoadFailed {
                path: path.clone(),
                error: error.clone(),
            }),
        }
    }

    /// Loads `path` and starts playback.
    ///
    /// Also marks `path` as the selected video so that only frames belonging
    /// to it are committed ([`VideoEvent::FrameReady`] matching).
    pub fn load(&mut self, path: PathBuf) {
        self.selected_path = Some(path.clone());
        self.rgba = None;
        self.ready = false;
        self.send(VideoCommand::Load(path));
    }

    pub fn play(&mut self) {
        self.send(VideoCommand::Play);
    }

    pub fn pause(&mut self) {
        self.send(VideoCommand::Pause);
    }

    pub fn toggle_pause(&mut self) {
        self.send(VideoCommand::TogglePause);
    }

    /// Seeks to `pos` seconds. Repeated calls within 333 ms are coalesced to
    /// avoid flooding mpv while the user drags a seekbar; the pending position
    /// is reflected immediately in the UI via `seek_position`.
    pub fn seek(&mut self, pos: f64) {
        self.seek_position = Some(pos);
        let should_seek = self
            .last_seek_time
            .is_none_or(|t| t.elapsed() >= Duration::from_millis(333));
        if should_seek {
            self.send(VideoCommand::SeekAbsolute(pos));
            self.last_seek_time = Some(Instant::now());
        }
    }

    pub fn set_volume(&mut self, vol: f64) {
        self.send(VideoCommand::SetVolume(vol));
    }

    pub fn toggle_mute(&mut self) {
        self.send(VideoCommand::SetMute(!self.muted));
    }

    pub fn stop(&mut self) {
        self.send(VideoCommand::Stop);
    }

    /// Stops playback and releases the current file handle on the worker.
    ///
    /// Also called automatically when the state is dropped. To additionally
    /// clear the visual state (frame, position, selection), use
    /// [`VideoState::reset`].
    pub fn deactivate(&mut self) {
        self.send(VideoCommand::Deactivate);
    }

    /// Stops playback and resets all observable state (frame, size, position,
    /// selection). Use when navigating away from a video, e.g. selecting an
    /// image or opening a different folder — this prevents a late
    /// [`VideoEvent::FrameReady`] from the previous video from repopulating
    /// the frame.
    pub fn reset(&mut self) {
        self.deactivate();
        self.rgba = None;
        self.width = 0;
        self.height = 0;
        self.rotation = Rotation::R0;
        self.position = 0.0;
        self.duration = 0.0;
        self.ready = false;
        self.seek_position = None;
        self.last_seek_time = None;
        self.selected_path = None;
    }
}

impl Drop for VideoState {
    fn drop(&mut self) {
        self.deactivate();
    }
}
