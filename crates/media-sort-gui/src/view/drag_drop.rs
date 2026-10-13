use iced::widget::{column, container, row, text};
use iced::{Alignment, Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::state::drag_drop::{DenyReason, DragDropMode, DragZone};

pub fn drag_drop_overlay_view(state: &AppState) -> Element<'_, Message> {
    let mode = &state.drag_drop.mode;
    let target_zone = state.drag_drop.target_zone;

    let content: Element<'_, Message> = match mode {
        DragDropMode::Denied(reason) => render_denied_card(state, reason),
        DragDropMode::Files { .. } => render_dual_cards(
            state,
            DualCardsOptions {
                active_zone: target_zone,
                left: DualCardOption {
                    title_key: "drag-drop-copy-title",
                    desc_key: "drag-drop-copy-desc",
                    icon: lucide_icons::Icon::Copy,
                    target_zone: DragZone::Copy,
                },
                right: DualCardOption {
                    title_key: "drag-drop-move-title",
                    desc_key: "drag-drop-move-desc",
                    icon: lucide_icons::Icon::ArrowRight,
                    target_zone: DragZone::Move,
                },
            },
        ),
        DragDropMode::SingleFolder => render_dual_cards(
            state,
            DualCardsOptions {
                active_zone: target_zone,
                left: DualCardOption {
                    title_key: "drag-drop-open-title",
                    desc_key: "drag-drop-open-desc",
                    icon: lucide_icons::Icon::FolderOpen,
                    target_zone: DragZone::Open,
                },
                right: DualCardOption {
                    title_key: "drag-drop-pin-title",
                    desc_key: "drag-drop-pin-desc",
                    icon: lucide_icons::Icon::Pin,
                    target_zone: DragZone::Pin,
                },
            },
        ),
        DragDropMode::None => container(column![]).into(),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color {
                r: 0.05,
                g: 0.07,
                b: 0.12,
                a: 0.85,
            })),
            ..Default::default()
        })
        .into()
}

fn render_denied_card<'a>(state: &'a AppState, reason: &DenyReason) -> Element<'a, Message> {
    let title = state.l10n.tr("drag-drop-denied-title");
    let desc_key = match reason {
        DenyReason::NoOpenFolder => "drag-drop-denied-no-folder",
        DenyReason::MixedItems => "drag-drop-denied-mixed",
        DenyReason::MultipleFolders => "drag-drop-denied-multiple-folders",
    };
    let description = state.l10n.tr(desc_key);

    let icon_element = text(char::from(lucide_icons::Icon::CircleX))
        .font(iced::Font::with_name("lucide"))
        .size(48)
        .style(|_theme: &iced::Theme| text::Style {
            color: Some(Color::from_rgb(0.95, 0.3, 0.3)),
        });

    let title_element = text(title)
        .size(22)
        .style(|_theme: &iced::Theme| text::Style {
            color: Some(Color::from_rgb(0.98, 0.4, 0.4)),
        });

    let desc_element = text(description)
        .size(14)
        .style(|_theme: &iced::Theme| text::Style {
            color: Some(Color::from_rgb(0.85, 0.85, 0.85)),
        });

    let card = column![icon_element, title_element, desc_element]
        .spacing(12)
        .align_x(Alignment::Center);

    container(card)
        .padding(32)
        .max_width(460)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color {
                r: 0.2,
                g: 0.05,
                b: 0.05,
                a: 0.9,
            })),
            border: iced::Border {
                color: Color::from_rgb(0.9, 0.25, 0.25),
                width: 2.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn wrap_drop_zone<'a>(
    card: impl Into<Element<'a, Message>>,
    zone: DragZone,
) -> Element<'a, Message> {
    iced::widget::mouse_area(card)
        .on_enter(Message::DragDrop(
            crate::message::DragDropMessage::ZoneHovered(zone),
        ))
        .on_move(move |_| Message::DragDrop(crate::message::DragDropMessage::ZoneHovered(zone)))
        .into()
}

struct DualCardOption<'a> {
    title_key: &'a str,
    desc_key: &'a str,
    icon: lucide_icons::Icon,
    target_zone: DragZone,
}

struct DualCardsOptions<'a> {
    active_zone: DragZone,
    left: DualCardOption<'a>,
    right: DualCardOption<'a>,
}

fn render_dual_cards<'a>(
    state: &'a AppState,
    options: DualCardsOptions<'a>,
) -> Element<'a, Message> {
    let left_active = options.active_zone == options.left.target_zone;
    let right_active = options.active_zone == options.right.target_zone;

    let left_card = render_single_target_card(
        state.l10n.tr(options.left.title_key),
        state.l10n.tr(options.left.desc_key),
        options.left.icon,
        left_active,
        Color::from_rgb(0.2, 0.5, 0.95), // Blue accent for left card
    );

    let right_card = render_single_target_card(
        state.l10n.tr(options.right.title_key),
        state.l10n.tr(options.right.desc_key),
        options.right.icon,
        right_active,
        Color::from_rgb(0.65, 0.35, 0.95), // Purple accent for right card
    );

    let left_card_area = wrap_drop_zone(left_card, options.left.target_zone);
    let right_card_area = wrap_drop_zone(right_card, options.right.target_zone);

    row![left_card_area, right_card_area]
        .spacing(24)
        .padding(32)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_single_target_card<'a>(
    title: String,
    description: String,
    icon: lucide_icons::Icon,
    is_active: bool,
    accent_color: Color,
) -> Element<'a, Message> {
    let icon_size = if is_active { 56 } else { 44 };

    let icon_element = text(char::from(icon))
        .font(iced::Font::with_name("lucide"))
        .size(icon_size)
        .style(move |_theme: &iced::Theme| text::Style {
            color: Some(if is_active {
                accent_color
            } else {
                Color::from_rgb(0.7, 0.7, 0.75)
            }),
        });

    let title_element =
        text(title)
            .size(if is_active { 24 } else { 20 })
            .style(move |_theme: &iced::Theme| text::Style {
                color: Some(if is_active {
                    Color::WHITE
                } else {
                    Color::from_rgb(0.85, 0.85, 0.9)
                }),
            });

    let desc_element = text(description)
        .size(14)
        .style(move |_theme: &iced::Theme| text::Style {
            color: Some(if is_active {
                Color::from_rgb(0.9, 0.95, 1.0)
            } else {
                Color::from_rgb(0.65, 0.65, 0.7)
            }),
        });

    let card_content = column![icon_element, title_element, desc_element]
        .spacing(16)
        .align_x(Alignment::Center);

    let bg_color = if is_active {
        Color {
            r: accent_color.r * 0.25,
            g: accent_color.g * 0.25,
            b: accent_color.b * 0.25,
            a: 0.85,
        }
    } else {
        Color {
            r: 0.1,
            g: 0.12,
            b: 0.18,
            a: 0.5,
        }
    };

    let border_color = if is_active {
        accent_color
    } else {
        Color {
            r: 0.25,
            g: 0.3,
            b: 0.4,
            a: 0.4,
        }
    };

    let border_width = if is_active { 3.0 } else { 1.5 };

    container(card_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(24)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: iced::Border {
                color: border_color,
                width: border_width,
                radius: 16.0.into(),
            },
            ..Default::default()
        })
        .into()
}
