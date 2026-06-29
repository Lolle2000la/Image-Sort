use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::view::folder_tree;

pub fn folder_panel_view(state: &AppState) -> Element<'_, Message> {
    let pin_btn = if state.current_folder.is_some() {
        button(text(state.l10n.tr("ui-pin")).size(12))
            .on_press(Message::PinCurrentFolder)
    } else {
        button(text(state.l10n.tr("ui-pin")).size(12))
    };

    let pin_sel_btn = if state.selected_folder.is_some() {
        button(text(state.l10n.tr("ui-pin-selected")).size(12))
            .on_press(Message::PinSelectedFolder)
    } else {
        button(text(state.l10n.tr("ui-pin-selected")).size(12))
    };

    let unpin_btn = if let Some(ref current) = state.current_folder {
        button(text(state.l10n.tr("ui-unpin")).size(12))
            .on_press(Message::UnpinCurrentFolder(current.clone()))
    } else {
        button(text(state.l10n.tr("ui-unpin")).size(12))
    };

    let has_parent = state.selected_folder.is_some() || state.current_folder.is_some();
    let create_folder_btn = if has_parent {
        button(text(state.l10n.tr("ui-create-folder")).size(12))
            .on_press(Message::TriggerCreateFolder)
    } else {
        button(text(state.l10n.tr("ui-create-folder")).size(12))
    };

    let buttons_row = row![pin_btn, pin_sel_btn, unpin_btn, create_folder_btn]
        .spacing(4)
        .wrap();

    let tree_content = folder_tree::folder_tree_view(&state.folder_tree, state.selected_folder.as_deref());
    let scrollable_tree = scrollable(tree_content)
        .direction(iced::widget::scrollable::Direction::Both {
            vertical: iced::widget::scrollable::Scrollbar::default(),
            horizontal: iced::widget::scrollable::Scrollbar::default(),
        })
        .height(Length::Fill);

    container(
        column![
            buttons_row,
            scrollable_tree,
        ]
        .spacing(6),
    )
    .padding(6)
    .width(Length::Fixed(f32::from(state.settings.general.folder_tree_width)))
    .height(Length::Fill)
    .into()
}
