use iced::widget::{button, column, container, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::message::FolderMessage;
use crate::state::AppState;

pub fn create_folder_modal_view<'a>(
    state: &'a AppState,
    parent: &'a std::path::Path,
) -> Element<'a, FolderMessage> {
    let parent_name = parent
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    let title = text(state.l10n.tr("ui-create-folder-title")).size(18);
    let parent_label = text(state.l10n.get("ui-parent", &[("name", &parent_name)]))
        .size(12)
        .color(Color::from_rgb(0.6, 0.6, 0.6))
        .shaping(iced::widget::text::Shaping::Advanced);

    let input = text_input(
        &state.create_folder.create_folder_placeholder,
        &state.create_folder.create_folder_input,
    )
    .on_input(FolderMessage::CreateInputChanged)
    .on_submit(FolderMessage::SubmitCreate(parent.to_path_buf()))
    .padding(8)
    .size(14);

    let submit_btn = button(text(state.l10n.tr("ui-create")).size(14))
        .on_press(FolderMessage::SubmitCreate(parent.to_path_buf()))
        .style(iced::widget::button::primary)
        .padding(8);

    let cancel_btn = button(text(state.l10n.tr("ui-cancel")).size(14))
        .on_press(FolderMessage::CancelCreate)
        .style(iced::widget::button::secondary)
        .padding(8);

    let buttons = iced::widget::row![submit_btn, cancel_btn].spacing(8);

    container(
        column![title, parent_label, input, buttons]
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
