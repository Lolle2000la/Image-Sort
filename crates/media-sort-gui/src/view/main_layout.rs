use iced::widget::{column, container, row, text, Text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::view::{
    control_panel, folder_panel, media_grid, media_preview, metadata_panel,
    search_bar,
};

pub fn main_layout_view(state: &AppState) -> Element<'_, Message> {
    let folder_panel = folder_panel::folder_panel_view(state);
    let control_panel = control_panel::control_panel_view(state);

    let search_bar = search_bar::search_bar_view(&state.search_query);
    let grid = media_grid::media_grid_view(state);
    let preview = media_preview::media_preview_view(state);
    let metadata = metadata_panel::metadata_panel_view(state);

    let move_btn = {
        let btn_content = row![
            text("Move "),
            text(char::from(lucide_icons::Icon::ArrowUp))
                .font(iced::Font::with_name("lucide"))
                .size(14),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);
        let btn = iced::widget::button(btn_content)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);
        if state.selected_index.is_some() && state.selected_folder.is_some() {
            btn.on_press(Message::MoveMedia)
        } else {
            btn
        }
    };

    // Rename (R) button next to search bar:
    let rename_btn = {
        let btn = iced::widget::button(text("Rename").size(13))
            .style(iced::widget::button::secondary);
        if state.selected_index.is_some() {
            btn.on_press(Message::TriggerRename)
        } else {
            btn
        }
    };

    // Delete (Down Arrow) button at the bottom:
    let delete_btn = {
        let btn_content = row![
            text("Delete "),
            text(char::from(lucide_icons::Icon::ArrowDown))
                .font(iced::Font::with_name("lucide"))
                .size(14),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);
        let btn = iced::widget::button(btn_content)
            .width(Length::Fill)
            .style(iced::widget::button::danger);
        if let Some(index) = state.selected_index {
            let filtered = state.filtered_media_entries();
            if let Some(entry) = filtered.get(index) {
                btn.on_press(Message::DeleteEntry(entry.path.clone()))
            } else {
                btn
            }
        } else {
            btn
        }
    };

    // Row containing the search bar and Rename button:
    let search_rename_row = row![
        search_bar,
        rename_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Center area (the preview area and bottom action area):
    let media_column = column![
        move_btn,
        container(preview).height(Length::Fill).width(Length::Fill),
        search_rename_row,
        grid,
        delete_btn,
    ]
    .spacing(8)
    .height(Length::Fill);

    let main_content = row![
        folder_panel,
        control_panel,
        row![
            container(media_column)
                .width(Length::Fill)
                .height(Length::Fill),
            metadata,
        ]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .spacing(8)
    .height(Length::Fill);

    let result = container(main_content).padding(8);

    let mut overlays = Vec::new();

    if state.show_settings {
        overlays.push(crate::view::settings_dialog::settings_dialog_view(state));
    }

    if state.show_credits {
        overlays.push(crate::view::credits_dialog::credits_dialog_view(state));
    }

    if let Some(ref path) = state.renaming_path {
        overlays.push(rename_modal_view(state, path));
    }

    if let Some(ref parent) = state.creating_folder_parent {
        overlays.push(create_folder_modal_view(state, parent));
    }

    if !overlays.is_empty() {
        let mut stack = iced::widget::stack![result];
        for overlay in overlays {
            stack = stack.push(
                container(overlay)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
                        ..iced::widget::container::Style::default()
                    })
            );
        }
        return stack.into();
    }

    result.into()
}

fn rename_modal_view<'a>(state: &'a AppState, path: &'a std::path::Path) -> Element<'a, Message> {
    let old_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    
    let title = Text::new("Rename File").size(18);
    let old_name_label = Text::new(format!("Original: {}", old_name))
        .size(12)
        .color(Color::from_rgb(0.6, 0.6, 0.6));

    let input = iced::widget::text_input("Enter new name...", &state.rename_input_value)
        .on_input(Message::RenameInputChanged)
        .on_submit(Message::SubmitRename)
        .padding(8)
        .size(14);

    let submit_btn = iced::widget::button(text("Rename").size(14))
        .on_press(Message::SubmitRename)
        .style(iced::widget::button::primary)
        .padding(8);

    let cancel_btn = iced::widget::button(text("Cancel").size(14))
        .on_press(Message::CancelRename)
        .style(iced::widget::button::secondary)
        .padding(8);

    let buttons = row![submit_btn, cancel_btn].spacing(8);

    container(
        column![title, old_name_label, input, buttons]
            .spacing(12)
            .align_x(iced::Alignment::Start)
    )
    .padding(20)
    .width(Length::Fixed(400.0))
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

fn create_folder_modal_view<'a>(state: &'a AppState, parent: &'a std::path::Path) -> Element<'a, Message> {
    let parent_name = parent.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "/".to_string());
    
    let title = Text::new("Create Folder").size(18);
    let parent_label = Text::new(format!("Parent: {}", parent_name))
        .size(12)
        .color(Color::from_rgb(0.6, 0.6, 0.6));

    let input = iced::widget::text_input("Folder name...", &state.create_folder_input)
        .on_input(Message::CreateFolderInputChanged)
        .on_submit(Message::SubmitCreateFolder)
        .padding(8)
        .size(14);

    let submit_btn = iced::widget::button(text("Create").size(14))
        .on_press(Message::SubmitCreateFolder)
        .style(iced::widget::button::primary)
        .padding(8);

    let cancel_btn = iced::widget::button(text("Cancel").size(14))
        .on_press(Message::CancelCreateFolder)
        .style(iced::widget::button::secondary)
        .padding(8);

    let buttons = row![submit_btn, cancel_btn].spacing(8);

    container(
        column![title, parent_label, input, buttons]
            .spacing(12)
            .align_x(iced::Alignment::Start)
    )
    .padding(20)
    .width(Length::Fixed(400.0))
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
