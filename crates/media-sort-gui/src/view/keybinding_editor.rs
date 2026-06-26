use iced::widget::{button, column, container, row, scrollable, text, Text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::subscriptions::keyboard::{format_keybinding, keybinding_list};

pub fn keybinding_editor_view(state: &AppState) -> Element<'_, Message> {
    let bindings = keybinding_list(state);
    let is_editing = state.editing_keybinding;
    let count = bindings.len();

    let header = text("Key Bindings").size(18);

    let display_names: Vec<String> = bindings
        .iter()
        .map(|(n, _)| keybinding_display_name(n))
        .collect();
    let shortcut_texts: Vec<String> = bindings.iter().map(|(_, b)| format_keybinding(b)).collect();

    let mut rows = Vec::with_capacity(count);

    for i in 0..count {
        let edit_label: Text<'_> = if is_editing == Some(i) {
            Text::new("Press a key...").color(Color::from_rgb(1.0, 0.8, 0.0))
        } else {
            Text::new("Edit")
        };

        let r = row![
            Text::new(display_names[i].clone())
                .size(13)
                .width(Length::Fixed(180.0)),
            Text::new(shortcut_texts[i].clone()).size(13),
            button(edit_label)
                .on_press(Message::EditKeyBinding(i))
                .style(iced::widget::button::secondary),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        rows.push(r.into());
    }

    let rows_column = column(rows).spacing(6);

    let close_btn = button(text("Close"))
        .on_press(Message::CloseSettings)
        .style(iced::widget::button::primary);

    container(
        column![
            header,
            scrollable(rows_column).height(Length::Fill),
            close_btn
        ]
        .spacing(12),
    )
    .padding(16)
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.3, 0.3, 0.35),
        },
        ..iced::widget::container::Style::default()
    })
    .width(Length::Fixed(420.0))
    .height(Length::Fixed(480.0))
    .into()
}

fn keybinding_display_name(name: &str) -> String {
    match name {
        "move_to_folder" => "Move to Folder".into(),
        "delete" => "Delete".into(),
        "rename" => "Rename".into(),
        "undo" => "Undo".into(),
        "redo" => "Redo".into(),
        "open_folder" => "Open Folder".into(),
        "search_images" => "Search Images".into(),
        "toggle_metadata_panel" => "Toggle Metadata".into(),
        "pin" => "Pin Folder".into(),
        "unpin" => "Unpin Folder".into(),
        _ => name.to_string(),
    }
}
