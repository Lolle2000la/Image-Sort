use crate::state::{PlayerMessage, VideoState};
use crate::subscription::video_player_subscription_with;
use iced::Subscription;
use mpv_utils::PlayerConfig;
use std::path::PathBuf;

#[cfg(feature = "ui")]
use crate::action::VideoAction;
#[cfg(feature = "ui")]
use iced::Element;

/// A self-contained video player: state, subscription and view in one type.
///
/// Subscribe with [`VideoPlayer::subscription`] / `subscription_with`, feed
/// messages into [`VideoPlayer::update`], and render with
/// [`VideoPlayer::view`] (which maps each [`VideoAction`] into your message
/// type via a closure). If you prefer to own the pieces yourself, embed a
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

    /// Loads `path` and starts playback (see [`VideoState::load`]).
    pub fn load(&mut self, path: impl Into<PathBuf>) {
        self.state.load(path.into());
    }

    /// The subscription that spawns the video worker with the default
    /// [`PlayerConfig`]. Map it into your app's message type, or use
    /// [`VideoPlayer::subscription_with`].
    pub fn subscription() -> Subscription<PlayerMessage> {
        Self::subscription_with_config(PlayerConfig::default())
    }

    /// Like [`VideoPlayer::subscription`], with a custom [`PlayerConfig`].
    pub fn subscription_with_config(config: PlayerConfig) -> Subscription<PlayerMessage> {
        video_player_subscription_with(config, PlayerMessage::Ready, PlayerMessage::Event)
    }

    /// Like [`VideoPlayer::subscription`], but maps each [`PlayerMessage`]
    /// into your own message type inside the subscription.
    pub fn subscription_with<Message, F>(map: F) -> Subscription<Message>
    where
        Message: Send + 'static,
        F: Fn(PlayerMessage) -> Message + Send + Sync + 'static + Clone,
    {
        Self::subscription_with_config_and_map(PlayerConfig::default(), map)
    }

    /// Like [`VideoPlayer::subscription_with`], with a custom
    /// [`PlayerConfig`].
    pub fn subscription_with_config_and_map<Message, F>(
        config: PlayerConfig,
        map: F,
    ) -> Subscription<Message>
    where
        Message: Send + 'static,
        F: Fn(PlayerMessage) -> Message + Send + Sync + 'static + Clone,
    {
        let on_ready = map.clone();
        video_player_subscription_with(
            config,
            move |handle| on_ready(PlayerMessage::Ready(handle)),
            move |event| map(PlayerMessage::Event(event)),
        )
    }

    /// Feeds one [`PlayerMessage`] into the state machine; returns a
    /// [`crate::VideoEvent`] when the app needs to react.
    pub fn update(&mut self, message: PlayerMessage) -> Option<crate::VideoEvent> {
        self.state.update(message)
    }

    /// Renders the video frame (or thumbnail / "Loading..." placeholder) and
    /// the transport controls, mapping each user action via `on_action`.
    #[cfg(feature = "ui")]
    pub fn view<'a, Message, F>(&'a self, on_action: F) -> Element<'a, Message>
    where
        Message: 'a,
        F: Fn(VideoAction) -> Message + 'a + Clone,
    {
        crate::widget::player::video_player_view(
            &self.state,
            self.thumb_handle.clone(),
            None,
            on_action,
        )
    }

    /// Like [`VideoPlayer::view`], with a custom element shown while neither
    /// a frame nor a thumbnail is available.
    #[cfg(feature = "ui")]
    pub fn view_with_placeholder<'a, Message, F>(
        &'a self,
        placeholder: Element<'a, Message>,
        on_action: F,
    ) -> Element<'a, Message>
    where
        Message: 'a,
        F: Fn(VideoAction) -> Message + 'a + Clone,
    {
        crate::widget::player::video_player_view(
            &self.state,
            self.thumb_handle.clone(),
            Some(placeholder),
            on_action,
        )
    }
}
