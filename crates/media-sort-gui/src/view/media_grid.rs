use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn media_grid_view(state: &AppState) -> Element<'_, Message> {
    let filtered = state.filtered_media_entries();

    if filtered.is_empty() {
        return container(
            text(if state.search_query.is_empty() {
                "No media files in this folder."
            } else {
                "No matching files."
            })
            .size(14),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    // Left navigation button
    let prev_btn = button(
        text(char::from(lucide_icons::Icon::ChevronLeft))
            .font(iced::Font::with_name("lucide"))
            .size(16)
    )
    .on_press(Message::GoLeft)
    .style(iced::widget::button::secondary);

    // Right navigation button
    let next_btn = button(
        text(char::from(lucide_icons::Icon::ChevronRight))
            .font(iced::Font::with_name("lucide"))
            .size(16)
    )
    .on_press(Message::GoRight)
    .style(iced::widget::button::secondary);

    let mut entries_row = row![].spacing(8);

    for (i, entry) in filtered.iter().enumerate() {
        let is_selected = state.selected_index == Some(i);

        let thumbnail_content: Element<'_, Message> = if let Some(bytes) = state.thumbnail_cache.peek(&entry.path) {
            let handle = iced::widget::image::Handle::from_bytes(bytes.clone());
            iced::widget::image(handle).width(Length::Fill).height(Length::Fill).into()
        } else {
            text("[IMG]").size(12).into()
        };

        let thumbnail = container(thumbnail_content)
            .center_x(60)
            .center_y(50)
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(50.0))
            .style(move |theme: &iced::Theme| {
                let palette = theme.palette();
                let bg = if is_selected {
                    palette.primary
                } else {
                    palette.background
                };
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: if is_selected { 2.0 } else { 0.0 },
                        color: if is_selected {
                            palette.text
                        } else {
                            Color::TRANSPARENT
                        },
                    },
                    ..iced::widget::container::Style::default()
                }
            });

        let file_name = text(&entry.file_name).size(10);
        let card = column![thumbnail, file_name]
            .align_x(Alignment::Center)
            .spacing(2)
            .width(Length::Fixed(60.0));

        let idx = i;
        let entry_button = button(card)
            .on_press(Message::SelectEntry(idx))
            .style(iced::widget::button::text);

        entries_row = entries_row.push(entry_button);
    }

    let scrollable_row = scrollable(entries_row)
        .direction(iced::widget::scrollable::Direction::Horizontal(iced::widget::scrollable::Scrollbar::default()));

    container(
        row![
            prev_btn,
            container(scrollable_row).width(Length::Fill),
            next_btn
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .into()
}
