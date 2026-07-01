use iced::widget::{button, column, container, text};
use iced::{Alignment, Color, Element, Length};

use crate::message::{MediaMessage, Message};
use crate::state::AppState;
use crate::widgets::video_player::video_player;

pub fn media_preview_view(state: &AppState) -> Element<'_, Message> {
    let Some(index) = state.selected_index else {
        return container(text(state.l10n.tr("ui-select-file")).size(14))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let filtered = state.filtered_media_entries();
    let Some(entry) = filtered.get(index) else {
        return container(text(state.l10n.tr("ui-file-not-found")).size(14))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    if state.unsupported_files.contains(&entry.path) {
        return container(
            column![
                text(state.l10n.tr("ui-file-not-supported")).size(14),
                button(text(state.l10n.tr("ui-open-externally"))).on_press(Message::Media(
                    MediaMessage::OpenExternal(entry.path.clone())
                )),
            ]
            .spacing(12)
            .align_x(Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    let preview_element: Element<'_, Message> = match entry.media_type {
        media_sort_core::media_type::MediaType::Image => {
            if let Some((ref path, ref handle)) = state.selected_image {
                if path == &entry.path {
                    iced::widget::image(handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    container(text(state.l10n.tr("ui-loading-image")).size(14))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
            } else {
                container(text(state.l10n.tr("ui-loading-image")).size(14))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
        media_sort_core::media_type::MediaType::Video => {
            let thumb = state.thumbnail_cache.peek(&entry.path).cloned();
            video_player(entry.path.clone(), state, thumb)
        }
        media_sort_core::media_type::MediaType::Audio => {
            crate::widgets::video_canvas::audio_controls_view()
        }
    };

    container(preview_element)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(4)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            let border_color = Color {
                a: 0.2,
                ..palette.text
            };
            iced::widget::container::Style {
                background: Some(iced::Background::Color(palette.background)),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: border_color,
                },
                ..iced::widget::container::Style::default()
            }
        })
        .into()
}
