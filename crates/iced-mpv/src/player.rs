use crate::action::VideoAction;
use crate::engine::worker::{VideoCommand, VideoEvent};
use crate::state::VideoState;
use crate::subscription::video_player_subscription;
use crate::widget::player::video_player_view;
use iced::{Element, Subscription};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    Ready(mpsc::Sender<VideoCommand>),
    Event(VideoEvent),
    Action(VideoAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoPlayerEvent {
    LoadFailed { path: PathBuf, error: String },
    PlayExternally(PathBuf),
}

pub struct VideoPlayer {
    pub state: VideoState,
    pub current_path: Option<PathBuf>,
    pub thumb_handle: Option<iced::widget::image::Handle>,
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            state: VideoState::new(),
            current_path: None,
            thumb_handle: None,
        }
    }

    pub fn with_thumbnail(mut self, handle: Option<iced::widget::image::Handle>) -> Self {
        self.thumb_handle = handle;
        self
    }

    pub fn set_thumbnail(&mut self, handle: Option<iced::widget::image::Handle>) {
        self.thumb_handle = handle;
    }

    pub fn load(&mut self, path: impl Into<PathBuf>) {
        let p = path.into();
        self.current_path = Some(p.clone());
        self.state.load(p);
    }

    pub fn subscription() -> Subscription<PlayerMessage> {
        video_player_subscription(PlayerMessage::Ready, PlayerMessage::Event)
    }

    pub fn update(&mut self, message: PlayerMessage) -> Option<VideoPlayerEvent> {
        match message {
            PlayerMessage::Ready(sender) => {
                self.state.set_sender(sender);
                if let Some(path) = &self.current_path {
                    self.state.load(path.clone());
                }
                None
            }
            PlayerMessage::Event(event) => {
                if let Some((path, err)) = self.state.handle_event(&event) {
                    Some(VideoPlayerEvent::LoadFailed { path, error: err })
                } else {
                    None
                }
            }
            PlayerMessage::Action(action) => match action {
                VideoAction::PlayPause => {
                    self.state.toggle_pause();
                    None
                }
                VideoAction::Stop => {
                    self.state.stop();
                    None
                }
                VideoAction::Seek(pos) => {
                    self.state.seek(pos);
                    None
                }
                VideoAction::SetVolume(vol) => {
                    self.state.set_volume(vol);
                    None
                }
                VideoAction::ToggleMute => {
                    self.state.toggle_mute();
                    None
                }
                VideoAction::PlayExternally(path) => Some(VideoPlayerEvent::PlayExternally(path)),
            },
        }
    }

    pub fn view(&self) -> Element<'_, PlayerMessage> {
        let path = self.current_path.clone().unwrap_or_default();
        video_player_view(
            path,
            &self.state,
            self.thumb_handle.clone(),
            None,
            PlayerMessage::Action,
        )
    }

    pub fn view_with_placeholder<'a>(
        &'a self,
        placeholder: Element<'a, PlayerMessage>,
    ) -> Element<'a, PlayerMessage> {
        let path = self.current_path.clone().unwrap_or_default();
        video_player_view(
            path,
            &self.state,
            self.thumb_handle.clone(),
            Some(placeholder),
            PlayerMessage::Action,
        )
    }
}
