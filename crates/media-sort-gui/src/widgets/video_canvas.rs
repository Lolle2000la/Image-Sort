use std::path::PathBuf;

use iced::widget::{column, container, text};
use iced::{Color, Element, Font, Length};

use crate::message::{MediaMessage, Message};
use crate::state::AppState;
use crate::widgets::media_controls;

pub fn audio_controls_view(
    _path: PathBuf,
    state: &AppState,
    thumb: Option<iced::widget::image::Handle>,
) -> Element<'_, Message> {
    let playing = state.audio_playing && !state.audio_player.as_ref().is_none_or(|p| p.is_paused());

    let audio_content: Element<'_, Message> = if let Some(handle) = thumb {
        iced::widget::image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        container(
            text(char::from(lucide_icons::Icon::Music))
                .font(Font::with_name("lucide"))
                .size(48)
                .color(Color::from_rgb(0.4, 0.4, 0.45)),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    };

    let controls = media_controls::media_controls_view(
        state.audio_position,
        state.audio_duration,
        state.audio_volume,
        state.audio_muted,
        playing,
    )
    .map(|msg| match msg {
        media_controls::MediaControlMessage::PlayPause => {
            Message::Media(MediaMessage::AudioPlayPause)
        }
        media_controls::MediaControlMessage::Stop => Message::Media(MediaMessage::StopAudio),
        media_controls::MediaControlMessage::Seek(v) => Message::Media(MediaMessage::AudioSeek(v)),
        media_controls::MediaControlMessage::SetVolume(v) => {
            Message::Media(MediaMessage::AudioSetVolume(v))
        }
        media_controls::MediaControlMessage::ToggleMute => {
            Message::Media(MediaMessage::AudioToggleMute)
        }
    });

    let content = column![
        container(audio_content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
        controls
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            iced::widget::container::Style {
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color {
                        a: 0.2,
                        ..palette.text
                    },
                },
                text_color: Some(palette.text),
                ..iced::widget::container::Style::default()
            }
        })
        .into()
}
