use iced::Element;
use iced::widget::{button, column, container, text};

use crate::message::{Message, UpdateMessage};
use crate::state::AppState;

pub fn update_prompt_view(state: &AppState) -> Element<'_, Message> {
    let Some(ref update) = state.pending_update else {
        return text("").into();
    };

    let version = &update.TargetFullRelease.Version;

    let title = text(state.l10n.tr("update-available-title")).size(20);

    let body = text(
        state
            .l10n
            .tr("update-available-body")
            .replace("{version}", version),
    )
    .size(14)
    .line_height(1.5);

    let update_btn = button(text(state.l10n.tr("update-confirm")).size(14)).on_press(
        Message::Update(UpdateMessage::UserConfirmedUpdate(update.clone())),
    );

    let dismiss_btn = button(text(state.l10n.tr("update-dismiss")).size(14))
        .on_press(Message::Update(UpdateMessage::DismissUpdatePrompt));

    let button_row = iced::widget::row![update_btn, dismiss_btn].spacing(12);

    let inner = container(
        column![title, body, button_row]
            .spacing(16)
            .max_width(380.0),
    )
    .padding(24)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(iced::Background::Color(palette.background.base.color)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    inner.into()
}
