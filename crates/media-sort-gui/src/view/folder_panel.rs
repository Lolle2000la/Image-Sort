use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::view::folder_tree;

pub fn folder_panel_view(state: &AppState) -> Element<'_, Message> {
    let current_path = state
        .current_folder
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "No folder selected".to_string());

    let folder_label = text("Folders").size(18);

    let path_display = text(format!("Path: {}", current_path)).size(12);

    let tree_content = folder_tree::folder_tree_view(&state.folder_tree);
    let scrollable_tree = scrollable(tree_content).height(Length::Fill);

    let mut pin_buttons = row![].spacing(4);
    if let Some(current) = &state.current_folder {
        let is_pinned = state.pinned_folders.iter().any(|p| p.path == *current);

        if is_pinned {
            pin_buttons = pin_buttons.push(
                button(text("Unpin"))
                    .on_press(Message::UnpinCurrentFolder(current.clone()))
                    .style(iced::widget::button::secondary),
            );
        } else {
            pin_buttons = pin_buttons.push(
                button(text("Pin"))
                    .on_press(Message::PinCurrentFolder)
                    .style(iced::widget::button::secondary),
            );
        }
    }

    let pinned_label = text("Pinned folders:").size(14);
    let mut pinned_column = column![].spacing(2);
    for pinned in &state.pinned_folders {
        let path = pinned.path.clone();
        pinned_column = pinned_column.push(
            button(text(&pinned.name).size(13))
                .on_press(Message::OpenFolder(path.clone()))
                .style(iced::widget::button::text)
                .width(Length::Fill),
        );
    }

    container(
        column![
            folder_label,
            path_display,
            pin_buttons,
            text("").height(8),
            scrollable_tree,
            text("").height(8),
            pinned_label,
            pinned_column,
        ]
        .spacing(4),
    )
    .padding(8)
    .width(Length::Fixed(220.0))
    .height(Length::Fill)
    .into()
}
