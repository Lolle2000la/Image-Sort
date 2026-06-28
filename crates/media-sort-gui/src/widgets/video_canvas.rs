use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

use crate::message::Message;

#[allow(dead_code)]
pub fn video_canvas_view() -> Element<'static, Message> {
    container(
        column![
            text("Video playback coming soon").size(16),
            text("MPV/wgpu rendering not yet implemented").size(12),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fixed(200.0))
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.08))),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.2, 0.2, 0.25),
        },
        ..iced::widget::container::Style::default()
    })
    .into()
}

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
                .on_press(Message::PlayAudio),
                button(
                    row![
                        text(char::from(lucide_icons::Icon::Pause))
                            .font(iced::Font::with_name("lucide"))
                            .size(12),
                        text(" Pause")
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::PauseAudio),
                button(
                    row![
                        text(char::from(lucide_icons::Icon::Square))
                            .font(iced::Font::with_name("lucide"))
                            .size(12),
                        text(" Stop")
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::StopAudio),
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
