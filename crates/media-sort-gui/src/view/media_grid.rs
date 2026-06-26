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

    let mut entries_column = column![].spacing(4);
    let mut current_row = row![].spacing(8);

    for (i, entry) in filtered.iter().enumerate() {
        let is_selected = state.selected_index == Some(i);

        let thumbnail = container(text("[IMG]").size(12))
            .center_x(60)
            .center_y(50)
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(50.0))
            .style(move |_theme| {
                let bg = if is_selected {
                    Color::from_rgb(0.3, 0.5, 0.9)
                } else {
                    Color::from_rgb(0.15, 0.15, 0.15)
                };
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: if is_selected { 2.0 } else { 0.0 },
                        color: if is_selected {
                            Color::WHITE
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

        current_row = current_row.push(entry_button);

        if (i + 1) % 4 == 0 || i == filtered.len() - 1 {
            entries_column = entries_column.push(current_row);
            current_row = row![].spacing(8);
        }
    }

    scrollable(entries_column).height(Length::Fill).into()
}
