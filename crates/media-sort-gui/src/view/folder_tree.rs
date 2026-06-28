use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

use media_sort_core::models::FolderNode;

use crate::message::Message;
use crate::widgets::folder_icon;

const INDENT_WIDTH: u16 = 20;

pub fn folder_tree_view<'a>(tree: &'a [FolderNode], selected_folder: Option<&'a std::path::Path>) -> Element<'a, Message> {
    column(
        tree.iter()
            .map(|node| render_node(node, 0, selected_folder))
            .collect::<Vec<_>>(),
    )
    .spacing(0)
    .into()
}

#[allow(clippy::only_used_in_recursion)]
fn render_node<'a>(node: &'a FolderNode, depth: u16, selected_folder: Option<&'a std::path::Path>) -> Element<'a, Message> {
    let icon = if node.is_expanded && !node.children.is_empty() {
        folder_icon::open_folder_icon()
    } else {
        folder_icon::folder_icon()
    };

    let node_path = node.path.clone();

    // Arrow button specifically for expand/collapse toggle
    let arrow_content: Element<'static, Message> = if node.children.is_empty() {
        text(" ").size(12).width(Length::Fixed(12.0)).into()
    } else {
        button(
            text(char::from(if node.is_expanded {
                lucide_icons::Icon::ChevronDown
            } else {
                lucide_icons::Icon::ChevronRight
            }))
            .font(iced::Font::with_name("lucide"))
            .size(12)
            .width(Length::Fixed(12.0))
        )
        .on_press(Message::ToggleFolderExpand(node_path.clone()))
        .style(iced::widget::button::text)
        .padding(iced::Padding::new(2.0))
        .into()
    };

    // Main row content with Folder icon and Folder name
    let row_content = row![icon, text(&node.name).size(14)]
        .spacing(4)
        .align_y(iced::Alignment::Center);

    // Folder selection button
    let select_button = button(row_content)
        .on_press(Message::FolderSelected(node_path.clone()))
        .style(move |theme: &iced::Theme, _status| {
            let palette = theme.palette();
            let base = iced::widget::button::Style::default();
            let is_selected = selected_folder.map_or(false, |p| p == node.path);
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
                let selected_bg = Color { a: 0.4, ..palette.primary };
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
        .width(Length::Fill);

    // Combined item layout: Chevron on the left, folder button on the right
    let item_layout = row![arrow_content, select_button]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

    let children: Vec<Element<'a, Message>> = if node.is_expanded && !node.children.is_empty() {
        vec![
            item_layout.into(),
            container(
                column(
                    node.children
                        .iter()
                        .filter(|child| !child.path.as_os_str().is_empty())
                        .map(|child| render_node(child, depth + 1, selected_folder))
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
