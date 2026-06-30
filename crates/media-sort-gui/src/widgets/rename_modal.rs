use iced::widget::{button, column, container, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::message::MediaMessage;
use crate::state::AppState;

pub fn rename_modal_view<'a>(
    state: &'a AppState,
    path: &'a std::path::Path,
) -> Element<'a, MediaMessage> {
    let old_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let title = text(state.l10n.tr("ui-rename-file")).size(18);
    let old_name_label = text(state.l10n.get("ui-original", &[("name", &old_name)]))
        .size(12)
        .color(Color::from_rgb(0.6, 0.6, 0.6))
        .shaping(iced::widget::text::Shaping::Advanced);

    let input = text_input(&state.rename_placeholder, &state.rename_input_value)
        .on_input(MediaMessage::RenameInputChanged)
        .on_submit(MediaMessage::SubmitRename)
        .padding(8)
        .size(14);

    let submit_btn = button(text(state.l10n.tr("ui-rename")).size(14))
        .on_press(MediaMessage::SubmitRename)
        .style(iced::widget::button::primary)
        .padding(8);

    let cancel_btn = button(text(state.l10n.tr("ui-cancel")).size(14))
        .on_press(MediaMessage::CancelRename)
        .style(iced::widget::button::secondary)
        .padding(8);

    let buttons = iced::widget::row![submit_btn, cancel_btn].spacing(8);

    container(
        column![title, old_name_label, input, buttons]
            .spacing(12)
            .align_x(Alignment::Start),
    )
    .padding(20)
    .width(Length::Fixed(400.0))
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        let border_color = Color {
            a: 0.2,
            ..palette.text
        };
        container::Style {
            background: Some(Background::Color(palette.background)),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: border_color,
            },
            ..container::Style::default()
        }
    })
    .into()
}
