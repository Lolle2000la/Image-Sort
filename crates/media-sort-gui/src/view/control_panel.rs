use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};
use crate::state::AppState;

pub fn control_panel_view(state: &AppState) -> Element<'_, Message> {
    // 1. Folder section
    let folder_header = text(state.l10n.tr("ui-folder")).size(15);
    let open_folder_btn = button(text(state.l10n.tr("ui-open-folder")).size(12))
        .on_press(Message::Folder(FolderMessage::Pick))
        .width(Length::Fill);

    let open_sel_btn = if let Some(ref selected) = state.selected_folder {
        button(text(state.l10n.tr("ui-open-selected-folder")).size(12))
            .on_press(Message::Folder(FolderMessage::Open(selected.clone())))
            .width(Length::Fill)
    } else {
        button(text(state.l10n.tr("ui-open-selected-folder")).size(12)).width(Length::Fill)
    };

    let folder_box = container(column![folder_header, open_folder_btn, open_sel_btn].spacing(8))
        .padding(10)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            let border_color = Color {
                a: 0.2,
                ..palette.text
            };
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
    let history_header = text(state.l10n.tr("ui-history")).size(15);
    let undo_btn = if state.history.can_undo() {
        button(text(state.l10n.tr("ui-undo")).size(12))
            .on_press(Message::Media(MediaMessage::Undo))
            .width(Length::Fill)
    } else {
        button(text(state.l10n.tr("ui-undo")).size(12)).width(Length::Fill)
    };

    let redo_btn = if state.history.can_redo() {
        button(text(state.l10n.tr("ui-redo")).size(12))
            .on_press(Message::Media(MediaMessage::Redo))
            .width(Length::Fill)
    } else {
        button(text(state.l10n.tr("ui-redo")).size(12)).width(Length::Fill)
    };

    let history_box =
        container(column![history_header, row![undo_btn, redo_btn].spacing(6)].spacing(8))
            .padding(10)
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                let border_color = Color {
                    a: 0.2,
                    ..palette.text
                };
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
    let settings_header = text(state.l10n.tr("ui-settings")).size(15);
    let open_settings_btn = container(
        button(text(state.l10n.tr("ui-open")).size(12))
            .on_press(Message::Settings(SettingsMessage::Open))
            .width(Length::Fill),
    )
    .id(iced::widget::Id::new("settings_btn"))
    .width(Length::Fill);

    let keybindings_btn = button(text(state.l10n.tr("ui-key-bindings")).size(12))
        .on_press(Message::Settings(SettingsMessage::OpenKeybindings))
        .width(Length::Fill);

    let credits_btn = button(text(state.l10n.tr("ui-credits")).size(12))
        .on_press(Message::OpenCredits)
        .width(Length::Fill);

    let settings_box = container(
        column![
            settings_header,
            open_settings_btn,
            keybindings_btn,
            credits_btn,
        ]
        .spacing(8),
    )
    .padding(10)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        let border_color = Color {
            a: 0.2,
            ..palette.text
        };
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

    container(column![folder_box, history_box, settings_box,].spacing(12))
        .padding(6)
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .into()
}
