use iced::widget::{button, column, container, row, scrollable, text, Text};
use iced::{Color, Element, Length};

use crate::message::{Message, SettingsMessage};
use crate::state::AppState;
use crate::subscriptions::keyboard::{format_keybinding, keybinding_list};

#[allow(dead_code)]
pub fn hotkey_popup_view(state: &AppState) -> Element<'_, Message> {
    let bindings = keybinding_list(state);
    let count = bindings.len();

    let descriptions: Vec<String> = bindings
        .iter()
        .map(|(n, _)| hotkey_description(state, n))
        .collect();
    let shortcut_texts: Vec<String> = bindings.iter().map(|(_, b)| format_keybinding(b)).collect();

    let header = text(state.l10n.tr("settings-tab-keybindings")).size(18);

    let mut rows = Vec::with_capacity(count);

    for i in 0..count {
        let r = row![
            Text::new(descriptions[i].clone()).size(13),
            iced::widget::Space::new().width(Length::Fill),
            Text::new(shortcut_texts[i].clone()).size(13),
        ]
        .spacing(8)
        .width(Length::Fill);
        rows.push(r.into());
    }

    let rows_column = column(rows).spacing(4);

    let close_btn = button(text(state.l10n.tr("ui-close")))
        .on_press(Message::Settings(SettingsMessage::CloseKeybindings))
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
    .width(Length::Fixed(400.0))
    .height(Length::Fixed(400.0))
    .into()
}

#[allow(dead_code)]
fn hotkey_description(state: &AppState, name: &str) -> String {
    match name {
        "move_to_folder" => state.l10n.tr("keybindings-move"),
        "delete" => state.l10n.tr("keybindings-delete"),
        "rename" => state.l10n.tr("keybindings-rename"),
        "undo" => state.l10n.tr("keybindings-undo"),
        "redo" => state.l10n.tr("keybindings-redo"),
        "open_folder" => state.l10n.tr("keybindings-open-folder"),
        "search_images" => state.l10n.tr("keybindings-search-images"),
        "toggle_metadata_panel" => state.l10n.tr("keybindings-toggle-metadata"),
        "pin" => state.l10n.tr("keybindings-pin"),
        "unpin" => state.l10n.tr("keybindings-unpin"),
        _ => name.to_string(),
    }
}
