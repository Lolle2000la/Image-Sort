use mpv_utils::{Rotation, VideoCommand, VideoEvent as WorkerEvent};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// An opaque handle to the video worker's command channel.
///
/// The subscription hands this to your app via
/// [`PlayerMessage::Ready`]; [`VideoState`] stores it so playback commands can
/// be sent. The underlying tokio channel is not exposed — to send commands,
/// call the [`VideoState`] methods (e.g. `state.video.seek(10.0)`).
///
/// The `new` constructor is only available with the `test-utils` feature (or
/// in this crate's own tests) — production code never constructs a handle
/// itself.
#[derive(Debug, Clone)]
pub struct PlayerHandle {
    sender: tokio::sync::mpsc::Sender<VideoCommand>,
}

impl PlayerHandle {
    /// Wraps a raw command channel sender.
    ///
    /// In normal use the subscription constructs this for you. This
    /// constructor exists for tests that want to drive [`VideoState`] without
    /// a running worker — prefer [`crate::testing::ready`].
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new(sender: tokio::sync::mpsc::Sender<VideoCommand>) -> Self {
        Self { sender }
    }

    pub(crate) fn from_sender(sender: tokio::sync::mpsc::Sender<VideoCommand>) -> Self {
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
    Event(WorkerEvent),
    /// An action originating from a widget (see [`VideoAction`](crate::VideoAction)).
    Action(crate::VideoAction),
}

/// Domain events that need application-level handling, produced by
/// [`VideoState::update`] (as opposed to [`WorkerEvent`], the raw worker
/// event stream).
#[derive(Debug, Clone, PartialEq)]
pub enum VideoEvent {
    /// Loading a file failed. `error` is a human-readable description.
    LoadFailed { path: PathBuf, error: String },
    /// The user asked to open the video in an external player.
    PlayExternally(PathBuf),
    /// The first frame of the current selection was committed, i.e. playback
    /// actually started for the selected file. Fired at most once per
    /// [`VideoState::load`] / [`VideoState::select`] cycle.
    FrameReady { path: PathBuf },
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
    /// The composite [`crate::video_player_view`] hides the transport controls
    /// until this is true.
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
    ///
    /// Note that frame pixels are always unrotated — the shader applies this
    /// rotation at draw time. Consumers that want raw pixels can ignore it.
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    /// The playback position in seconds.
    pub fn position(&self) -> f64 {
        self.position
    }

    /// A pending seek target in seconds, if the user is currently dragging
    /// the seekbar (throttled seeks are not yet applied to mpv). Cleared once
    /// the worker confirms playback at the target position.
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
    /// machine. Returns a [`VideoEvent`] when the app needs to react (load
    /// failure, play-externally request, first committed frame). The result is
    /// `#[must_use]` — silently dropping these events loses application
    /// behavior.
    #[must_use = "returned VideoEvent (LoadFailed / PlayExternally / FrameReady) requires application handling"]
    pub fn update(&mut self, message: PlayerMessage) -> Option<VideoEvent> {
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
                crate::VideoAction::PlayExternally(path) => Some(VideoEvent::PlayExternally(path)),
            },
        }
    }

    fn handle_event(&mut self, event: &WorkerEvent) -> Option<VideoEvent> {
        match event {
            WorkerEvent::FrameReady {
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
                    let first_frame = self.rgba.is_none();
                    self.rgba = Some(rgba.clone());
                    self.width = *width;
                    self.height = *height;
                    self.rotation = *rotation;
                    if first_frame {
                        Some(VideoEvent::FrameReady { path: path.clone() })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            WorkerEvent::PlaybackProgress { position, duration } => {
                self.position = *position;
                self.duration = *duration;
                self.ready = true;
                // A pending seek counts as applied once playback reaches it
                // (within a second's slack for keyframe snapping), so the
                // seekbar stops pinning the drag target.
                if let Some(target) = self.seek_position
                    && (*position - target).abs() <= 1.0
                {
                    self.seek_position = None;
                }
                None
            }
            WorkerEvent::Muted(muted) => {
                self.muted = *muted;
                None
            }
            WorkerEvent::Volume(vol) => {
                self.volume = *vol;
                None
            }
            WorkerEvent::Paused(paused) => {
                self.paused = *paused;
                None
            }
            WorkerEvent::LoadFailed { path, error } => Some(VideoEvent::LoadFailed {
                path: path.clone(),
                error: error.clone(),
            }),
        }
    }

    /// Selects what the player should do after a selection change:
    /// `Some(path)` loads a video and starts playback, `None` (e.g. an image
    /// or audio file was selected, or nothing is selected) stops playback and
    /// resets all observable state via [`VideoState::reset`].
    pub fn select(&mut self, path: Option<PathBuf>) {
        match path {
            Some(path) => self.load(path),
            None => self.reset(),
        }
    }

    /// Loads `path` and starts playback.
    ///
    /// Also marks `path` as the selected video so that only frames belonging
    /// to it are committed ([`WorkerEvent::FrameReady`] matching). The next
    /// committed frame reports a [`VideoEvent::FrameReady`] domain event.
    ///
    /// The transport state is reset so the control bar never shows the
    /// previous video's time or pause state until the first
    /// [`WorkerEvent::PlaybackProgress`] arrives.
    pub fn load(&mut self, path: PathBuf) {
        self.selected_path = Some(path.clone());
        self.rgba = None;
        self.ready = false;
        self.position = 0.0;
        self.duration = 0.0;
        self.seek_position = None;
        self.paused = false;
        self.send(VideoCommand::Load(path));
    }

    /// Resumes playback.
    pub fn play(&self) {
        self.send(VideoCommand::Play);
    }

    /// Pauses playback.
    pub fn pause(&self) {
        self.send(VideoCommand::Pause);
    }

    /// Toggles between playing and paused.
    pub fn toggle_pause(&self) {
        self.send(VideoCommand::TogglePause);
    }

    /// Seeks to `pos` seconds. Repeated calls within 333 ms are coalesced to
    /// avoid flooding mpv while the user drags a seekbar; the pending position
    /// is reflected immediately in the UI via `seek_position` and cleared once
    /// playback reaches it.
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

    /// Sets the volume in percent, clamped to 0–100.
    pub fn set_volume(&self, vol: f64) {
        self.send(VideoCommand::SetVolume(vol.clamp(0.0, 100.0)));
    }

    /// Toggles mute.
    pub fn toggle_mute(&self) {
        self.send(VideoCommand::SetMute(!self.muted));
    }

    /// Stops playback.
    pub fn stop(&self) {
        self.send(VideoCommand::Stop);
    }

    /// Stops playback and releases the current file handle on the worker.
    ///
    /// Also called automatically when the state is dropped. To additionally
    /// clear the visual state (frame, position, selection), use
    /// [`VideoState::reset`].
    pub fn deactivate(&self) {
        self.send(VideoCommand::Deactivate);
    }

    /// Stops playback and resets all observable state (frame, size, position,
    /// selection). Use when navigating away from a video, e.g. selecting an
    /// image or opening a different folder — this prevents a late
    /// [`WorkerEvent::FrameReady`] from the previous video from repopulating
    /// the frame. Prefer [`VideoState::select`] over calling this directly.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use std::path::Path;

    fn connected_state() -> VideoState {
        let (msg, _rx) = testing::ready(8);
        let mut state = VideoState::new();
        let _ = state.update(msg);
        state
    }

    fn progress_event(position: f64, duration: f64) -> WorkerEvent {
        WorkerEvent::PlaybackProgress { position, duration }
    }

    fn frame_event(path: &str, rotation: Rotation) -> WorkerEvent {
        WorkerEvent::FrameReady {
            path: PathBuf::from(path),
            width: 640,
            height: 360,
            rotation,
            rgba: Arc::new(vec![0u8; 640 * 360 * 4]),
        }
    }

    #[test]
    fn test_select_some_loads() {
        let mut state = connected_state();
        state.select(Some(PathBuf::from("/videos/one.mp4")));
        assert_eq!(
            state.selected_path().map(|p| p.as_path()),
            Some(Path::new("/videos/one.mp4"))
        );
        assert!(!state.ready());
    }

    #[test]
    fn test_select_none_resets() {
        let mut state = connected_state();
        state.load(PathBuf::from("/videos/one.mp4"));
        let _ = state.update(PlayerMessage::Event(progress_event(5.0, 10.0)));
        assert!(state.ready());

        state.select(None);
        assert_eq!(state.selected_path(), None);
        assert!(!state.ready());
        assert_eq!(state.position(), 0.0);
        assert_eq!(state.duration(), 0.0);
    }

    #[test]
    fn test_load_resets_transport_state() {
        let mut state = connected_state();
        state.load(PathBuf::from("/videos/one.mp4"));
        let _ = state.update(PlayerMessage::Event(progress_event(30.0, 60.0)));
        let _ = state.update(PlayerMessage::Event(WorkerEvent::Paused(true)));
        state.seek(40.0);
        assert_eq!(state.position(), 30.0);
        assert!(state.paused());
        assert_eq!(state.seek_position(), Some(40.0));

        // Loading a different video must not show the previous one's
        // position/duration, pending seek or paused state until the worker
        // reports fresh values.
        state.load(PathBuf::from("/videos/two.mp4"));
        assert_eq!(
            state.selected_path().map(|p| p.as_path()),
            Some(Path::new("/videos/two.mp4"))
        );
        assert_eq!(state.position(), 0.0);
        assert_eq!(state.duration(), 0.0);
        assert_eq!(state.seek_position(), None);
        assert!(!state.paused());
        assert!(!state.ready());
    }

    #[test]
    fn test_reset_clears_all_state() {
        let mut state = connected_state();
        state.load(PathBuf::from("/videos/one.mp4"));
        let _ = state.update(PlayerMessage::Event(progress_event(5.0, 10.0)));
        let _ = state.update(PlayerMessage::Event(frame_event(
            "/videos/one.mp4",
            Rotation::R90,
        )));
        assert!(state.ready());
        assert!(state.rgba().is_some());
        assert_eq!(state.width(), 640);

        state.reset();
        assert_eq!(state.selected_path(), None);
        assert!(!state.ready());
        assert!(state.rgba().is_none());
        assert_eq!(state.width(), 0);
        assert_eq!(state.rotation(), Rotation::R0);
        assert_eq!(state.seek_position(), None);
    }

    #[test]
    fn test_stale_frame_ignored_after_reset() {
        let mut state = connected_state();
        state.load(PathBuf::from("/videos/one.mp4"));
        let _ = state.update(PlayerMessage::Event(progress_event(0.0, 10.0)));

        // A frame for a *different* path (or after selection changed) is dropped.
        state.select(None);
        let _ = state.update(PlayerMessage::Event(frame_event(
            "/videos/one.mp4",
            Rotation::R0,
        )));
        assert!(state.rgba().is_none());
    }

    #[test]
    fn test_first_frame_emits_domain_event_once() {
        let mut state = connected_state();
        state.load(PathBuf::from("/videos/one.mp4"));
        let _ = state.update(PlayerMessage::Event(progress_event(0.0, 10.0)));

        let first = state.update(PlayerMessage::Event(frame_event(
            "/videos/one.mp4",
            Rotation::R0,
        )));
        assert_eq!(
            first,
            Some(VideoEvent::FrameReady {
                path: PathBuf::from("/videos/one.mp4")
            })
        );

        let second = state.update(PlayerMessage::Event(frame_event(
            "/videos/one.mp4",
            Rotation::R0,
        )));
        assert_eq!(second, None);

        // Reloading the same file counts as a new load cycle.
        state.load(PathBuf::from("/videos/one.mp4"));
        let _ = state.update(PlayerMessage::Event(progress_event(0.0, 10.0)));
        let reloaded = state.update(PlayerMessage::Event(frame_event(
            "/videos/one.mp4",
            Rotation::R0,
        )));
        assert_eq!(
            reloaded,
            Some(VideoEvent::FrameReady {
                path: PathBuf::from("/videos/one.mp4")
            })
        );
    }

    #[test]
    fn test_seek_position_cleared_when_reached() {
        let mut state = connected_state();
        state.load(PathBuf::from("/videos/one.mp4"));

        state.seek(42.0);
        assert_eq!(state.seek_position(), Some(42.0));

        // Progress while still far from the target keeps the pending seek.
        let _ = state.update(PlayerMessage::Event(progress_event(10.0, 120.0)));
        assert_eq!(state.seek_position(), Some(42.0));

        // Progress at the target clears it.
        let _ = state.update(PlayerMessage::Event(progress_event(42.0, 120.0)));
        assert_eq!(state.seek_position(), None);
    }

    #[test]
    fn test_set_volume_clamps() {
        let (msg, mut rx) = testing::ready(8);
        let mut state = VideoState::new();
        let _ = state.update(msg);

        state.set_volume(150.0);
        match rx.try_recv() {
            Ok(VideoCommand::SetVolume(v)) => assert_eq!(v, 100.0),
            other => panic!("expected SetVolume(100.0), got {other:?}"),
        }
        state.set_volume(-20.0);
        match rx.try_recv() {
            Ok(VideoCommand::SetVolume(v)) => assert_eq!(v, 0.0),
            other => panic!("expected SetVolume(0.0), got {other:?}"),
        }
    }

    #[test]
    fn test_ready_sets_connected() {
        let mut state = VideoState::new();
        assert!(!state.is_connected());

        let (msg, _rx) = testing::ready(8);
        let _ = state.update(msg);
        assert!(state.is_connected());
    }

    #[test]
    fn test_action_sends_command() {
        let (msg, mut rx) = testing::ready(8);
        let mut state = VideoState::new();
        let _ = state.update(msg);
        assert!(state.is_connected());

        let _ = state.update(PlayerMessage::Action(crate::VideoAction::SetVolume(50.0)));
        match rx.try_recv() {
            Ok(VideoCommand::SetVolume(v)) => assert_eq!(v, 50.0),
            other => panic!("expected SetVolume(50.0), got {other:?}"),
        }
    }

    #[test]
    fn test_action_play_pause_without_sender() {
        let mut state = VideoState::new();
        assert!(!state.is_connected());
        let _ = state.update(PlayerMessage::Action(crate::VideoAction::PlayPause));
        assert!(!state.is_connected());
    }

    #[test]
    fn test_action_stop_without_sender() {
        let mut state = VideoState::new();
        assert!(!state.is_connected());
        let _ = state.update(PlayerMessage::Action(crate::VideoAction::Stop));
        assert!(!state.is_connected());
    }

    #[test]
    fn test_progress_event_updates_transport() {
        let mut state = VideoState::new();
        let _ = state.update(PlayerMessage::Event(progress_event(10.0, 120.0)));
        assert_eq!(state.position(), 10.0);
        assert_eq!(state.duration(), 120.0);
        assert!(state.ready());
    }

    #[test]
    fn test_muted_event_sets_muted() {
        let mut state = VideoState::new();
        assert!(!state.muted());
        let _ = state.update(PlayerMessage::Event(WorkerEvent::Muted(true)));
        assert!(state.muted());
    }

    #[test]
    fn test_volume_event_sets_volume() {
        let mut state = VideoState::new();
        let _ = state.update(PlayerMessage::Event(WorkerEvent::Volume(75.0)));
        assert_eq!(state.volume(), 75.0);
    }

    #[test]
    fn test_paused_event_sets_paused() {
        let mut state = VideoState::new();
        assert!(!state.paused());
        let _ = state.update(PlayerMessage::Event(WorkerEvent::Paused(true)));
        assert!(state.paused());
    }

    #[test]
    fn test_action_seek_stores_position() {
        let mut state = connected_state();
        let _ = state.update(PlayerMessage::Action(crate::VideoAction::Seek(42.0)));
        assert_eq!(state.seek_position(), Some(42.0));
    }

    #[test]
    fn test_action_seek_without_sender() {
        let mut state = VideoState::new();
        assert!(!state.is_connected());
        let _ = state.update(PlayerMessage::Action(crate::VideoAction::Seek(10.0)));
        assert_eq!(state.seek_position(), Some(10.0));
    }
}
