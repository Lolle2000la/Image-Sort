use iced::widget::{button, checkbox, column, container, row, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn settings_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = text("Settings").size(20);

    let dark_mode_cb = checkbox("Dark Mode", state.settings.general.dark_mode)
        .on_toggle(|_| Message::ToggleDarkMode)
        .size(16);

    let animate_gifs_cb = checkbox("Animate GIFs", state.settings.general.animate_gifs)
        .on_toggle(|_| Message::ToggleAnimateGifs)
        .size(16);

    let animate_thumbs_cb = checkbox(
        "Animate GIF Thumbnails",
        state.settings.general.animate_gif_thumbnails,
    )
    .on_toggle(|_| Message::ToggleAnimateThumbnails)
    .size(16);

    let btn_row = row![
        button(text("Cancel"))
            .on_press(Message::CloseSettings)
            .style(iced::widget::button::secondary),
        button(text("Save"))
            .on_press(Message::SaveSettings)
            .style(iced::widget::button::primary),
    ]
    .spacing(8);

    container(
        column![
            title,
            dark_mode_cb,
            animate_gifs_cb,
            animate_thumbs_cb,
            btn_row,
        ]
        .spacing(16)
        .align_x(iced::Alignment::Start),
    )
    .padding(24)
    .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.3, 0.3, 0.35),
        },
        ..iced::widget::container::Style::default()
    })
    .width(Length::Fixed(350.0))
    .height(Length::Fixed(280.0))
    .into()
}
