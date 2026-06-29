use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;
use crate::view::folder_tree;

pub fn folder_panel_view(state: &AppState) -> Element<'_, Message> {
    let pin_btn = if state.current_folder.is_some() {
        button(text(state.l10n.tr("ui-pin")).size(12))
            .on_press(Message::PinCurrentFolder)
    } else {
        button(text(state.l10n.tr("ui-pin")).size(12))
    };

    let pin_sel_btn = if state.selected_folder.is_some() {
        button(text(state.l10n.tr("ui-pin-selected")).size(12))
            .on_press(Message::PinSelectedFolder)
    } else {
        button(text(state.l10n.tr("ui-pin-selected")).size(12))
    };

    let unpin_btn = if let Some(ref current) = state.current_folder {
        button(text(state.l10n.tr("ui-unpin")).size(12))
            .on_press(Message::UnpinCurrentFolder(current.clone()))
    } else {
        button(text(state.l10n.tr("ui-unpin")).size(12))
    };

    let has_parent = state.selected_folder.is_some() || state.current_folder.is_some();
    let create_folder_btn = if has_parent {
        button(text(state.l10n.tr("ui-create-folder")).size(12))
            .on_press(Message::TriggerCreateFolder)
    } else {
        button(text(state.l10n.tr("ui-create-folder")).size(12))
    };

    let buttons_row = row![pin_btn, pin_sel_btn, unpin_btn, create_folder_btn]
        .spacing(4)
        .wrap();

    // Pinned folders section
    let pinned_section = if !state.pinned_folders.is_empty() {
        let mut pinned_col = column![
            text(state.l10n.tr("ui-pinned-folders"))
                .size(13)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                }),
        ]
        .spacing(6);

        for (i, pinned) in state.pinned_folders.iter().enumerate() {
            let shortcut_str = if i < 9 {
                format!("Alt+{}", i + 1)
            } else {
                String::new()
            };

            // Reorder up/down buttons
            let up_btn = if i > 0 {
                button(
                    text(char::from(lucide_icons::Icon::ChevronUp))
                        .font(iced::Font::with_name("lucide"))
                        .size(11)
                )
                .style(iced::widget::button::text)
                .on_press(Message::MovePinnedFolderUp(pinned.path.clone()))
            } else {
                button(
                    text(char::from(lucide_icons::Icon::ChevronUp))
                        .font(iced::Font::with_name("lucide"))
                        .size(11)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.15))
                )
                .style(iced::widget::button::text)
            };

            let down_btn = if i < state.pinned_folders.len() - 1 {
                button(
                    text(char::from(lucide_icons::Icon::ChevronDown))
                        .font(iced::Font::with_name("lucide"))
                        .size(11)
                )
                .style(iced::widget::button::text)
                .on_press(Message::MovePinnedFolderDown(pinned.path.clone()))
            } else {
                button(
                    text(char::from(lucide_icons::Icon::ChevronDown))
                        .font(iced::Font::with_name("lucide"))
                        .size(11)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.15))
                )
                .style(iced::widget::button::text)
            };

            // Folder button (Click to open)
            let folder_btn = button(
                row![
                    text(char::from(lucide_icons::Icon::Folder))
                        .font(iced::Font::with_name("lucide"))
                        .size(12)
                        .color(Color::from_rgb(0.9, 0.75, 0.3)),
                    text(&pinned.name).size(12),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center)
            )
            .style(iced::widget::button::secondary)
            .on_press(Message::OpenFolder(pinned.path.clone()))
            .width(Length::Fill);

            // Shortcut badge
            let shortcut_badge = if !shortcut_str.is_empty() {
                container(
                    text(shortcut_str)
                        .size(9)
                        .color(Color::from_rgb(0.7, 0.7, 0.7))
                )
                .padding([2, 4])
                .style(|theme: &iced::Theme| {
                    let palette = theme.palette();
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color { a: 0.1, ..palette.text })),
                        border: iced::Border {
                            radius: 3.0.into(),
                            width: 1.0,
                            color: Color { a: 0.15, ..palette.text },
                        },
                        ..Default::default()
                    }
                })
            } else {
                container(iced::widget::Space::new())
            };

            // Unpin button
            let unpin_item_btn = button(
                text(char::from(lucide_icons::Icon::Pin))
                    .font(iced::Font::with_name("lucide"))
                    .size(12)
            )
            .style(iced::widget::button::text)
            .on_press(Message::UnpinCurrentFolder(pinned.path.clone()));

            let row_content = row![
                up_btn,
                down_btn,
                folder_btn,
                shortcut_badge,
                unpin_item_btn,
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center);

            pinned_col = pinned_col.push(row_content);
        }

        // Add a line separator after pinned folders section
        let separator = container(
            container(iced::widget::Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                .style(|theme: &iced::Theme| {
                    let palette = theme.palette();
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color { a: 0.1, ..palette.text })),
                        ..Default::default()
                    }
                })
        )
        .padding([4, 0]);

        column![pinned_col, separator].spacing(6)
    } else {
        column![]
    };

    let tree_content = folder_tree::folder_tree_view(&state.folder_tree, state.selected_folder.as_deref());
    let scrollable_tree = scrollable(tree_content)
        .direction(iced::widget::scrollable::Direction::Both {
            vertical: iced::widget::scrollable::Scrollbar::default(),
            horizontal: iced::widget::scrollable::Scrollbar::default(),
        })
        .height(Length::Fill);

    container(
        column![
            buttons_row,
            pinned_section,
            scrollable_tree,
        ]
        .spacing(6),
    )
    .padding(6)
    .width(Length::Fixed(f32::from(state.settings.general.folder_tree_width)))
    .height(Length::Fill)
    .into()
}
