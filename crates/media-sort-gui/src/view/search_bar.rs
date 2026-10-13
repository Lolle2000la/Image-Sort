use crate::message::{MediaMessage, Message};
use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

pub static SEARCH_INPUT_ID: iced::widget::Id = iced::widget::Id::new("search_bar");

pub fn search_bar_view<'a>(query: &'a str, placeholder: &'a str) -> Element<'a, Message> {
    let search_input = text_input(placeholder, query)
        .id(SEARCH_INPUT_ID.clone())
        .on_input(|s| Message::Media(MediaMessage::SearchQueryChanged(s)))
        .padding(6)
        .width(Length::Fill);

    let clear_btn = if !query.is_empty() {
        button(text("X").size(12))
            .on_press(Message::Media(MediaMessage::SearchQueryChanged(
                String::new(),
            )))
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
