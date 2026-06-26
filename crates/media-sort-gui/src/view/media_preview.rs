use iced::widget::{column, container, row, text};
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

    let type_label = match entry.media_type {
        media_sort_core::media_type::MediaType::Image => "Image",
        media_sort_core::media_type::MediaType::Video => "Video",
        media_sort_core::media_type::MediaType::Audio => "Audio",
    };

    let preview_placeholder = container(column![
        text("Image preview").size(16),
        text(format!("Type: {}", type_label)).size(12),
    ])
    .center_x(300)
    .center_y(200)
    .width(Length::Fixed(300.0))
    .height(Length::Fixed(200.0))
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.3, 0.3, 0.3),
        },
        ..iced::widget::container::Style::default()
    });

    let info = column![
        text(&entry.file_name).size(18),
        text(format!("Type: {}", type_label)).size(12),
        text(format!("Path: {}", entry.path.display())).size(10),
    ]
    .spacing(4);

    let delete_btn = iced::widget::button(text("Delete"))
        .on_press(Message::DeleteEntry(entry.path.clone()))
        .style(iced::widget::button::danger);

    let action_row = row![delete_btn].spacing(8);

    container(column![info, preview_placeholder, action_row].spacing(12))
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
