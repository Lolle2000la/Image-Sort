use crate::action::VideoAction;
use crate::state::VideoState;
use crate::widget::controls::{MediaControlMessage, media_controls_view};
use crate::widget::shader::video_shader_view;
use iced::widget::{column, container};
use iced::{Element, Length};
use std::path::PathBuf;

pub fn video_player_view<'a, Message: 'a, F>(
    _path: PathBuf,
    state: &VideoState,
    thumb_handle: Option<iced::widget::image::Handle>,
    placeholder: Option<Element<'a, Message>>,
    on_action: F,
) -> Element<'a, Message>
where
    F: Fn(VideoAction) -> Message + 'a + Clone,
{
    let video_content: Element<'_, Message> = if state.rgba.is_some() {
        video_shader_view(
            state.width,
            state.height,
            state.rotation,
            state.rgba.clone(),
        )
    } else if let Some(handle) = thumb_handle {
        iced::widget::image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else if let Some(ph) = placeholder {
        ph
    } else {
        container(iced::widget::text("Loading video..."))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    let display_position = state.seek_position.unwrap_or(state.position);

    let on_action_clone = on_action.clone();
    let controls_row = media_controls_view(
        display_position,
        state.duration,
        state.volume,
        state.muted,
        !state.paused,
    )
    .map(move |msg| match msg {
        MediaControlMessage::PlayPause => on_action_clone(VideoAction::PlayPause),
        MediaControlMessage::Stop => on_action_clone(VideoAction::Stop),
        MediaControlMessage::Seek(v) => on_action_clone(VideoAction::Seek(v)),
        MediaControlMessage::SetVolume(v) => on_action_clone(VideoAction::SetVolume(v)),
        MediaControlMessage::ToggleMute => on_action_clone(VideoAction::ToggleMute),
    });

    column![
        container(video_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        controls_row
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
