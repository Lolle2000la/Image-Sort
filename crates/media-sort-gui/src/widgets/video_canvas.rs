use iced::widget::{button, column, container, row, text};
use iced::{Color, Element};

use crate::message::{MediaMessage, Message};

#[allow(dead_code)]
pub fn audio_controls_view() -> Element<'static, Message> {
    container(
        column![
            text("Audio Controls").size(14),
            row![
                button(
                    row![
                        text(char::from(lucide_icons::Icon::Play))
                            .font(iced::Font::with_name("lucide"))
                            .size(12),
                        text(" Play")
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::Media(MediaMessage::PlayAudio)),
                button(
                    row![
                        text(char::from(lucide_icons::Icon::Pause))
                            .font(iced::Font::with_name("lucide"))
                            .size(12),
                        text(" Pause")
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::Media(MediaMessage::PauseAudio)),
                button(
                    row![
                        text(char::from(lucide_icons::Icon::Square))
                            .font(iced::Font::with_name("lucide"))
                            .size(12),
                        text(" Stop")
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::Media(MediaMessage::StopAudio)),
            ]
            .spacing(8),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .padding(8)
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
        border: iced::Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.25, 0.25, 0.3),
        },
        ..iced::widget::container::Style::default()
    })
    .into()
}
