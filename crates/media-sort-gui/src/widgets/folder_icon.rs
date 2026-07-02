use iced::Element;
use iced::widget::text;

pub fn folder_icon() -> Element<'static, super::super::message::Message> {
    text(char::from(lucide_icons::Icon::Folder))
        .font(iced::Font::with_name("lucide"))
        .size(16)
        .into()
}

pub fn open_folder_icon() -> Element<'static, super::super::message::Message> {
    text(char::from(lucide_icons::Icon::FolderOpen))
        .font(iced::Font::with_name("lucide"))
        .size(16)
        .into()
}

pub fn arrow_up_icon() -> Element<'static, super::super::message::Message> {
    text(char::from(lucide_icons::Icon::ArrowUp))
        .font(iced::Font::with_name("lucide"))
        .size(16)
        .into()
}
