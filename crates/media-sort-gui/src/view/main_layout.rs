use std::path::PathBuf;

use iced::widget::{column, container, horizontal_rule, row, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::view::{
    folder_panel, history_bar, keybinding_editor, media_grid, media_preview, metadata_panel,
    search_bar,
};

pub fn main_layout_view(state: &AppState) -> Element<'_, Message> {
    let folder_panel = folder_panel::folder_panel_view(state);

    let search_bar = search_bar::search_bar_view(&state.search_query);

    let grid = media_grid::media_grid_view(state);
    let preview = media_preview::media_preview_view(state);

    let metadata = metadata_panel::metadata_panel_view(state);
    let hist = history_bar::history_bar_view(state);

    let top_bar = row![
        text("Media Sort v3.0.0").size(18),
        {
            iced::widget::button(text("Open Folder"))
                .on_press({
                    let path = std::env::current_dir().ok();
                    Message::OpenFolder(path.unwrap_or_else(|| PathBuf::from(".")))
                })
                .style(iced::widget::button::primary)
        },
        iced::widget::horizontal_space(),
        {
            iced::widget::button(text("\u{2699}"))
                .on_press(Message::OpenSettings)
                .style(iced::widget::button::secondary)
        },
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let center_area = column![
        search_bar,
        horizontal_rule(1),
        row![
            column![grid]
                .width(iced::Length::FillPortion(1))
                .height(iced::Length::Fill),
            column![preview]
                .width(iced::Length::FillPortion(1))
                .height(iced::Length::Fill),
            metadata,
        ]
        .height(iced::Length::Fill)
        .spacing(4),
        horizontal_rule(1),
        hist,
    ]
    .spacing(4);

    let main_content = row![
        folder_panel,
        container(center_area)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(4)
    .height(Length::Fill);

    let app_content = column![top_bar, horizontal_rule(1), main_content,].spacing(4);

    let result = container(app_content).padding(8);

    if state.show_settings {
        let overlay = if state.editing_keybinding.is_some() {
            keybinding_editor::keybinding_editor_view(state)
        } else {
            crate::view::settings_dialog::settings_dialog_view(state)
        };

        return iced::widget::stack![
            result,
            container(overlay)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .into();
    }

    result.into()
}
