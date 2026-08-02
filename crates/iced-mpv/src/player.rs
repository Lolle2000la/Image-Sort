use crate::state::{PlayerMessage, VideoState};
use crate::subscription::video_player_subscription;
use crate::widget::player::video_player_view;
use iced::{Element, Subscription};
use std::path::PathBuf;

/// A self-contained video player: state, subscription and view in one type.
///
/// This is the most ergonomic entry point when you want the crate's own
/// widgets: subscribe with [`VideoPlayer::subscription`], feed messages into
/// [`VideoPlayer::update`], and render with [`VideoPlayer::view`]. The app
/// message type must be [`PlayerMessage`] for `view` to wire the built-in
/// controls — if your app has its own message enum, embed a
/// [`VideoState`] and use the free [`crate::video_player_view`] instead.
#[derive(Debug)]
pub struct VideoPlayer {
    pub state: VideoState,
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
        self.state.load(path.into());
    }

    /// The subscription that spawns the video worker. Map it into your app's
    /// message type, or use [`VideoPlayer::subscription_with`].
    pub fn subscription() -> Subscription<PlayerMessage> {
        video_player_subscription(PlayerMessage::Ready, PlayerMessage::Event)
    }

    /// Like [`VideoPlayer::subscription`], but maps each [`PlayerMessage`]
    /// into your own message type inside the subscription.
    pub fn subscription_with<Message, F>(map: F) -> Subscription<Message>
    where
        Message: Send + 'static,
        F: Fn(PlayerMessage) -> Message + Send + Sync + 'static + Clone,
    {
        let on_ready = map.clone();
        video_player_subscription(
            move |handle| on_ready(PlayerMessage::Ready(handle)),
            move |event| map(PlayerMessage::Event(event)),
        )
    }

    pub fn update(&mut self, message: PlayerMessage) -> Option<crate::VideoPlayerEvent> {
        self.state.update(message)
    }

    pub fn view(&self) -> Element<'_, PlayerMessage> {
        video_player_view(
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
        video_player_view(
            &self.state,
            self.thumb_handle.clone(),
            Some(placeholder),
            PlayerMessage::Action,
        )
    }
}
