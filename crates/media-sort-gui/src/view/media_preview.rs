use iced::widget::{container, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn media_preview_view(state: &AppState) -> Element<'_, Message> {
    let Some(index) = state.selected_index else {
        return container(text("Select a file to preview").size(14))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let filtered = state.filtered_media_entries();
    let Some(entry) = filtered.get(index) else {
        return container(text("File not found").size(14))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let preview_element: Element<'_, Message> = match entry.media_type {
        media_sort_core::media_type::MediaType::Image => {
            if let Some((ref path, ref bytes)) = state.selected_image_bytes {
                if path == &entry.path {
                    let handle = iced::widget::image::Handle::from_bytes(bytes.clone());
                    iced::widget::image(handle)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    container(text("Loading image...").size(14))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
            } else {
                container(text("Loading image...").size(14))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
        media_sort_core::media_type::MediaType::Video => {
            crate::widgets::video_canvas::video_canvas_view()
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
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            let border_color = Color { a: 0.2, ..palette.text };
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
