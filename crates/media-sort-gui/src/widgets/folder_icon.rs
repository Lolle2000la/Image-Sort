use iced::widget::text;
use iced::Element;

pub fn folder_icon() -> Element<'static, super::super::message::Message> {
    text("\u{1F4C1}").size(18).into()
}

pub fn open_folder_icon() -> Element<'static, super::super::message::Message> {
    text("\u{1F4C2}").size(18).into()
}
