use std::path::PathBuf;

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Font, Length};

use crate::message::{Message, VideoMessage};
use crate::state::AppState;
use crate::widgets::media_controls;

pub fn video_player<'a>(
    path: PathBuf,
    state: &AppState,
    thumb_handle: Option<iced::widget::image::Handle>,
) -> Element<'a, Message> {
    let video_content: Element<'_, Message> = if state.video_rgba.is_some() {
        crate::widgets::video_shader::video_shader_view(
            state.video_width,
            state.video_height,
            state.video_rgba.clone(),
        )
    } else if let Some(handle) = thumb_handle {
        iced::widget::image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        placeholder(path, &state.l10n)
    };

    let controls_row = media_controls::media_controls_view(
        state.video_position,
        state.video_duration,
        state.video_volume,
        state.video_muted,
        !state.video_paused,
    )
    .map(|msg| match msg {
        media_controls::MediaControlMessage::PlayPause => Message::Video(VideoMessage::PlayPause),
        media_controls::MediaControlMessage::Stop => Message::Video(VideoMessage::Stop),
        media_controls::MediaControlMessage::Seek(v) => Message::Video(VideoMessage::Seek(v)),
        media_controls::MediaControlMessage::SetVolume(v) => {
            Message::Video(VideoMessage::Volume(v))
        }
        media_controls::MediaControlMessage::ToggleMute => Message::Video(VideoMessage::Mute),
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

fn placeholder(
    path: PathBuf,
    l10n: &media_sort_core::l10n::Localization,
) -> Element<'static, Message> {
    container(
        column![
            text(l10n.tr("ui-video-playback-soon")).size(16),
            text(l10n.tr("ui-rendering-not-implemented")).size(12),
            button(
                row![
                    text(char::from(lucide_icons::Icon::ExternalLink))
                        .font(Font::with_name("lucide"))
                        .size(12),
                    text(format!(" {}", l10n.tr("ui-play-in-system-player")))
                ]
                .align_y(Alignment::Center)
            )
            .padding([8, 16])
            .on_press(Message::Video(VideoMessage::PlayExternally(path))),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        container::Style {
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color {
                    a: 0.2,
                    ..palette.text
                },
            },
            text_color: Some(palette.text),
            ..container::Style::default()
        }
    })
    .into()
}
