use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn history_bar_view(state: &AppState) -> Element<'_, Message> {
    let can_undo = state.history.can_undo();
    let can_redo = state.history.can_redo();

    let undo_btn = if can_undo {
        let label = format!("Undo ({})", state.history.last_done_name().unwrap_or(""));
        button(text(label)).on_press(Message::Undo)
    } else {
        button(text("Undo"))
    };

    let status_text = text(format!(
        "Done: {}  |  Undone: {}",
        state.history.done_len(),
        state.history.undone_len()
    ))
    .size(12);

    let redo_btn = if can_redo {
        let label = format!("Redo ({})", state.history.last_undone_name().unwrap_or(""));
        button(text(label)).on_press(Message::Redo)
    } else {
        button(text("Redo"))
    };

    container(
        row![undo_btn, status_text, redo_btn]
            .spacing(12)
            .align_y(iced::Alignment::Center),
    )
    .padding(4)
    .width(Length::Fill)
    .into()
}
