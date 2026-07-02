use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

use media_sort_core::models::FolderNode;

use crate::message::{FolderMessage, Message};
use crate::widgets::folder_icon;

const INDENT_WIDTH: f32 = 20.0;

pub fn folder_tree_view<'a>(
    tree: &'a [FolderNode],
    selected_folder: Option<&'a std::path::Path>,
) -> Element<'a, Message> {
    column(
        tree.iter()
            .enumerate()
            .map(|(i, node)| render_node(node, 0, i, selected_folder))
            .collect::<Vec<_>>(),
    )
    .spacing(0)
    .into()
}

#[allow(clippy::only_used_in_recursion)]
fn render_node<'a>(
    node: &'a FolderNode,
    depth: u16,
    root_index: usize,
    selected_folder: Option<&'a std::path::Path>,
) -> Element<'a, Message> {
    let icon = if node.is_parent_nav {
        folder_icon::arrow_up_icon()
    } else if node.is_expanded && !node.children.is_empty() {
        folder_icon::open_folder_icon()
    } else {
        folder_icon::folder_icon()
    };

    let node_path = node.path.clone();

    // Arrow button specifically for expand/collapse toggle
    let arrow_content: Element<'static, Message> = if node.children.is_empty() {
        container(text("")).width(Length::Fixed(16.0)).into()
    } else {
        button(
            text(char::from(if node.is_expanded {
                lucide_icons::Icon::ChevronDown
            } else {
                lucide_icons::Icon::ChevronRight
            }))
            .font(iced::Font::with_name("lucide"))
            .size(12)
            .width(Length::Fixed(12.0)),
        )
        .on_press(Message::Folder(FolderMessage::ToggleExpand(
            node_path.clone(),
        )))
        .style(iced::widget::button::text)
        .padding(iced::Padding::new(2.0))
        .into()
    };

    // Pinned status indicators (only on root folders other than the current folder)
    let pin_indicator = if depth == 0 && root_index > 0 {
        Some(
            text(char::from(lucide_icons::Icon::Pin))
                .font(iced::Font::with_name("lucide"))
                .size(11)
                .color(Color::from_rgb(0.9, 0.45, 0.45)),
        )
    } else {
        None
    };

    let shortcut_badge = if depth == 0 && root_index > 0 && root_index <= 9 {
        Some(
            container(
                text(format!("Alt+{}", root_index))
                    .size(9)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
            )
            .padding([2, 4])
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color {
                        a: 0.1,
                        ..palette.text
                    })),
                    border: iced::Border {
                        radius: 3.0.into(),
                        width: 1.0,
                        color: Color {
                            a: 0.15,
                            ..palette.text
                        },
                    },
                    ..Default::default()
                }
            }),
        )
    } else {
        None
    };

    let mut row_content = row![icon].spacing(4).align_y(iced::Alignment::Center);

    if let Some(pin) = pin_indicator {
        row_content = row_content.push(pin);
    }

    row_content = row_content.push(
        text(&node.name)
            .size(14)
            .wrapping(iced::widget::text::Wrapping::None)
            .shaping(iced::widget::text::Shaping::Advanced),
    );

    if let Some(badge) = shortcut_badge {
        row_content = row_content.push(badge);
    }

    // Folder selection button
    let folder_action = if node.is_parent_nav {
        FolderMessage::Open(node_path.clone())
    } else {
        FolderMessage::Selected(node_path.clone())
    };
    let select_button = button(row_content)
        .on_press(Message::Folder(folder_action))
        .style(move |theme: &iced::Theme, _status| {
            let palette = theme.palette();
            let base = iced::widget::button::Style::default();
            let is_selected = selected_folder.is_some_and(|p| p == node.path);
            if node.is_current {
                let border = if is_selected {
                    iced::Border {
                        color: Color::WHITE,
                        width: 2.0,
                        radius: 4.0.into(),
                    }
                } else {
                    iced::Border::default()
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(palette.primary)),
                    text_color: Color::WHITE,
                    border,
                    ..base
                }
            } else if is_selected {
                let selected_bg = Color {
                    a: 0.4,
                    ..palette.primary
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(selected_bg)),
                    text_color: palette.text,
                    ..base
                }
            } else {
                iced::widget::button::Style {
                    text_color: palette.text,
                    ..base
                }
            }
        })
        .width(Length::Shrink);

    // Combined item layout: Chevron on the left, folder button on the right
    let item_layout = row![arrow_content, select_button]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .width(Length::Shrink);

    let children: Vec<Element<'a, Message>> = if node.is_expanded && !node.children.is_empty() {
        vec![
            item_layout.into(),
            container(
                column(
                    node.children
                        .iter()
                        .filter(|child| !child.path.as_os_str().is_empty())
                        .map(|child| render_node(child, depth + 1, root_index, selected_folder))
                        .collect::<Vec<_>>(),
                )
                .spacing(0),
            )
            .padding(iced::Padding::new(0.).left(INDENT_WIDTH))
            .into(),
        ]
    } else {
        vec![item_layout.into()]
    };

    column(children).spacing(0).into()
}
