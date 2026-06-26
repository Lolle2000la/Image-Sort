use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

use crate::message::Message;

pub fn search_bar_view<'a>(query: &'a str) -> Element<'a, Message> {
    let search_input = text_input("Search images... (Shortcuts: 'I' to focus, 'Tab' to leave)", query)
        .on_input(Message::SearchQueryChanged)
        .padding(6)
        .width(Length::Fill);

    let clear_btn = if !query.is_empty() {
        button(text("X").size(12))
            .on_press(Message::SearchQueryChanged(String::new()))
            .style(iced::widget::button::text)
    } else {
        button(text("").size(12)).style(iced::widget::button::text)
    };

    container(
        row![search_input, clear_btn]
            .spacing(4)
            .align_y(iced::Alignment::Center),
    )
    .padding(4)
    .width(Length::Fill)
    .into()
}
