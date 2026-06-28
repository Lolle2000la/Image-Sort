use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Color};

use crate::message::Message;
use crate::state::AppState;

pub fn control_panel_view(state: &AppState) -> Element<'_, Message> {
    // 1. Folder section
    let folder_header = text("Folder").size(15);
    let open_folder_btn = button(text("Open folder").size(12))
        .on_press(Message::PickFolder)
        .style(iced::widget::button::secondary)
        .width(Length::Fill);

    let open_sel_btn = if let Some(ref selected) = state.selected_folder {
        button(text("Open selected folder").size(12))
            .on_press(Message::OpenFolder(selected.clone()))
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    } else {
        button(text("Open selected folder").size(12))
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    };

    let folder_box = container(
        column![folder_header, open_folder_btn, open_sel_btn].spacing(8)
    )
    .padding(10)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        let border_color = Color { a: 0.2, ..palette.text };
        iced::widget::container::Style {
            background: Some(iced::Background::Color(palette.background)),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: border_color,
            },
            ..iced::widget::container::Style::default()
        }
    });

    // 2. History section
    let history_header = text("History").size(15);
    let undo_btn = if state.history.can_undo() {
        button(text("Undo").size(12))
            .on_press(Message::Undo)
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    } else {
        button(text("Undo").size(12))
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    };

    let redo_btn = if state.history.can_redo() {
        button(text("Redo").size(12))
            .on_press(Message::Redo)
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    } else {
        button(text("Redo").size(12))
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    };

    let history_box = container(
        column![
            history_header,
            row![undo_btn, redo_btn].spacing(6)
        ].spacing(8)
    )
    .padding(10)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        let border_color = Color { a: 0.2, ..palette.text };
        iced::widget::container::Style {
            background: Some(iced::Background::Color(palette.background)),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: border_color,
            },
            ..iced::widget::container::Style::default()
        }
    });

    // 3. Settings section
    let settings_header = text("Settings").size(15);
    let open_settings_btn = button(text("Open").size(12))
        .on_press(Message::OpenSettings)
        .style(iced::widget::button::secondary)
        .width(Length::Fill);

    let keybindings_btn = button(text("Key bindings").size(12))
        .on_press(Message::OpenKeybindings)
        .style(iced::widget::button::secondary)
        .width(Length::Fill);

    let credits_btn = button(text("Credits").size(12))
        .on_press(Message::OpenCredits)
        .style(iced::widget::button::secondary)
        .width(Length::Fill);

    let settings_box = container(
        column![
            settings_header,
            open_settings_btn,
            keybindings_btn,
            credits_btn,
        ].spacing(8)
    )
    .padding(10)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        let border_color = Color { a: 0.2, ..palette.text };
        iced::widget::container::Style {
            background: Some(iced::Background::Color(palette.background)),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: border_color,
            },
            ..iced::widget::container::Style::default()
        }
    });

    container(
        column![
            folder_box,
            history_box,
            settings_box,
        ]
        .spacing(12)
    )
    .padding(6)
    .width(Length::Fixed(220.0))
    .height(Length::Fill)
    .into()
}
