use std::path::PathBuf;

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Font, Length};

use crate::message::{MediaMessage, Message};
use crate::state::AppState;

pub fn video_player<'a>(
    path: PathBuf,
    state: &AppState,
    thumb_handle: Option<iced::widget::image::Handle>,
) -> Element<'a, Message> {
    iced_mpv::video_player_view(
        &state.video,
        thumb_handle,
        Some(placeholder(path, &state.l10n)),
        |action| Message::Video(iced_mpv::PlayerMessage::Action(action)),
    )
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
            .on_press(Message::Media(MediaMessage::OpenExternal(path))),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style::default())
    .into()
}
