use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::view::folder_tree;

pub fn folder_panel_view(state: &AppState) -> Element<'_, Message> {
    let pin_btn = if state.current_folder.is_some() {
        button(text("Pin").size(12))
            .on_press(Message::PinCurrentFolder)
            .style(iced::widget::button::secondary)
    } else {
        button(text("Pin").size(12))
            .style(iced::widget::button::secondary)
    };

    let pin_sel_btn = if state.selected_folder.is_some() {
        button(text("Pin selected").size(12))
            .on_press(Message::PinSelectedFolder)
            .style(iced::widget::button::secondary)
    } else {
        button(text("Pin selected").size(12))
            .style(iced::widget::button::secondary)
    };

    let unpin_btn = if let Some(ref current) = state.current_folder {
        button(text("Unpin").size(12))
            .on_press(Message::UnpinCurrentFolder(current.clone()))
            .style(iced::widget::button::secondary)
    } else {
        button(text("Unpin").size(12))
            .style(iced::widget::button::secondary)
    };

    let has_parent = state.selected_folder.is_some() || state.current_folder.is_some();
    let create_folder_btn = if has_parent {
        button(text("Create Folder").size(12))
            .on_press(Message::TriggerCreateFolder)
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    } else {
        button(text("Create Folder").size(12))
            .style(iced::widget::button::secondary)
            .width(Length::Fill)
    };

    let buttons_row = row![pin_btn, pin_sel_btn, unpin_btn].spacing(4);

    let tree_content = folder_tree::folder_tree_view(&state.folder_tree, state.selected_folder.as_deref());
    let scrollable_tree = scrollable(tree_content).height(Length::Fill);

    container(
        column![
            buttons_row,
            create_folder_btn,
            scrollable_tree,
        ]
        .spacing(6),
    )
    .padding(6)
    .width(Length::Fixed(240.0))
    .height(Length::Fill)
    .into()
}
