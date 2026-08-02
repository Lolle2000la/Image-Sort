use crate::engine::worker::{VideoCommand, VideoEvent};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct VideoState {
    pub sender: Option<tokio::sync::mpsc::Sender<VideoCommand>>,
    pub frame: Option<iced::widget::image::Handle>,
    pub rgba: Option<Arc<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
    pub rotation: i64,
    pub position: f64,
    pub duration: f64,
    pub volume: f64,
    pub muted: bool,
    pub paused: bool,
    pub ready: bool,
    pub seek_position: Option<f64>,
    pub last_seek_time: Option<Instant>,
    pub selected_path: Option<PathBuf>,
}

impl Default for VideoState {
    fn default() -> Self {
        Self {
            sender: None,
            frame: None,
            rgba: None,
            width: 0,
            height: 0,
            rotation: 0,
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

    pub fn set_sender(&mut self, sender: tokio::sync::mpsc::Sender<VideoCommand>) {
        self.sender = Some(sender);
    }

    pub fn update(
        &mut self,
        message: crate::player::PlayerMessage,
    ) -> Option<crate::player::VideoPlayerEvent> {
        use crate::action::VideoAction;
        use crate::player::PlayerMessage;
        use crate::player::VideoPlayerEvent;

        match message {
            PlayerMessage::Ready(sender) => {
                self.set_sender(sender);
                if let Some(path) = &self.selected_path {
                    self.load(path.clone());
                }
                None
            }
            PlayerMessage::Event(event) => {
                if let Some((path, err)) = self.handle_event(&event) {
                    Some(VideoPlayerEvent::LoadFailed { path, error: err })
                } else {
                    None
                }
            }
            PlayerMessage::Action(action) => match action {
                VideoAction::PlayPause => {
                    self.toggle_pause();
                    None
                }
                VideoAction::Stop => {
                    self.stop();
                    None
                }
                VideoAction::Seek(pos) => {
                    self.seek(pos);
                    None
                }
                VideoAction::SetVolume(vol) => {
                    self.set_volume(vol);
                    None
                }
                VideoAction::ToggleMute => {
                    self.toggle_mute();
                    None
                }
                VideoAction::PlayExternally(path) => Some(VideoPlayerEvent::PlayExternally(path)),
            },
        }
    }

    pub fn handle_event(&mut self, event: &VideoEvent) -> Option<(PathBuf, String)> {
        match event {
            VideoEvent::FrameReady {
                path,
                width,
                height,
                rotation,
                rgba,
            } => {
                if self.selected_path.as_deref() == Some(path.as_path()) && self.ready {
                    self.rgba = Some(rgba.clone());
                    self.width = *width;
                    self.height = *height;
                    self.rotation = *rotation;
                    self.frame = Some(iced::widget::image::Handle::from_rgba(1, 1, vec![0]));
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
            VideoEvent::LoadFailed { path, error } => Some((path.clone(), error.clone())),
        }
    }

    pub fn load(&mut self, path: PathBuf) {
        self.selected_path = Some(path.clone());
        self.rgba = None;
        self.ready = false;
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::Load(path));
        }
    }

    pub fn play(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::Play);
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::Pause);
        }
    }

    pub fn toggle_pause(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::TogglePause);
        }
    }

    pub fn seek(&mut self, pos: f64) {
        self.seek_position = Some(pos);
        let should_seek = self
            .last_seek_time
            .is_none_or(|t| t.elapsed() >= Duration::from_millis(333));
        if should_seek {
            if let Some(ref sender) = self.sender {
                let _ = sender.try_send(VideoCommand::SeekAbsolute(pos));
            }
            self.last_seek_time = Some(Instant::now());
        }
    }

    pub fn set_volume(&mut self, vol: f64) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::SetVolume(vol));
        }
    }

    pub fn toggle_mute(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::SetMute(!self.muted));
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::Stop);
        }
    }

    pub fn deactivate(&mut self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.try_send(VideoCommand::Deactivate);
        }
    }
}
