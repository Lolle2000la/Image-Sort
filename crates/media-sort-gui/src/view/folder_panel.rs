use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::{FolderMessage, Message};
use crate::state::AppState;
use crate::view::folder_tree;

pub static FOLDER_TREE_SCROLLABLE_ID: iced::widget::Id =
    iced::widget::Id::new("folder_tree_scrollable");

pub fn folder_panel_view(state: &AppState) -> Element<'_, Message> {
    let pin_btn = button(text(state.l10n.tr("ui-pin")).size(12))
        .on_press(Message::Folder(FolderMessage::PickPin));

    let pin_sel_btn = {
        let path_to_pin = state
            .selected_folder
            .clone()
            .or(state.current_folder.clone());
        if path_to_pin.is_some() {
            button(text(state.l10n.tr("ui-pin-selected")).size(12))
                .on_press(Message::Folder(FolderMessage::PinSelected))
        } else {
            button(text(state.l10n.tr("ui-pin-selected")).size(12))
        }
    };

    let unpin_btn = {
        let path_to_unpin = state
            .selected_folder
            .clone()
            .or(state.current_folder.clone());
        if let Some(path) = path_to_unpin {
            button(text(state.l10n.tr("ui-unpin")).size(12))
                .on_press(Message::Folder(FolderMessage::UnpinCurrent(path)))
        } else {
            button(text(state.l10n.tr("ui-unpin")).size(12))
        }
    };

    let has_parent = state.selected_folder.is_some() || state.current_folder.is_some();
    let create_folder_btn = if has_parent {
        button(text(state.l10n.tr("ui-create-folder")).size(12))
            .on_press(Message::Folder(FolderMessage::TriggerCreate))
    } else {
        button(text(state.l10n.tr("ui-create-folder")).size(12))
    };

    let buttons_row = row![pin_btn, pin_sel_btn, unpin_btn, create_folder_btn]
        .spacing(4)
        .wrap();

    let tree_content = folder_tree::folder_tree_view(&state.folder_tree, state.selected_folder_idx);
    let scrollable_tree = scrollable(tree_content)
        .id(FOLDER_TREE_SCROLLABLE_ID.clone())
        .direction(iced::widget::scrollable::Direction::Both {
            vertical: iced::widget::scrollable::Scrollbar::default(),
            horizontal: iced::widget::scrollable::Scrollbar::default(),
        })
        .width(Length::Fill)
        .height(Length::Fill);

    container(column![buttons_row, scrollable_tree,].spacing(6))
        .padding(6)
        .width(Length::Fixed(f32::from(
            state.settings.general.folder_tree_width,
        )))
        .height(Length::Fill)
        .into()
}
