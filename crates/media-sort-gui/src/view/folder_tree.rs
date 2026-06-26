use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

use media_sort_core::models::FolderNode;

use crate::message::Message;
use crate::widgets::folder_icon;

const INDENT_WIDTH: u16 = 20;

pub fn folder_tree_view<'a>(tree: &'a [FolderNode]) -> Element<'a, Message> {
    column(
        tree.iter()
            .map(|node| render_node(node, 0))
            .collect::<Vec<_>>(),
    )
    .spacing(0)
    .into()
}

#[allow(clippy::only_used_in_recursion)]
fn render_node<'a>(node: &'a FolderNode, depth: u16) -> Element<'a, Message> {
    let icon = if node.is_expanded {
        folder_icon::open_folder_icon()
    } else {
        folder_icon::folder_icon()
    };

    let arrow = if node.children.is_empty() {
        text("  ").size(14)
    } else if node.is_expanded {
        text("\u{25BC}").size(12)
    } else {
        text("\u{25B6}").size(12)
    };

    let node_path = node.path.clone();

    let row_content = row![arrow, icon, text(&node.name).size(14),]
        .spacing(4)
        .align_y(iced::Alignment::Center);

    let row_button = button(row_content)
        .on_press(Message::FolderSelected(node_path.clone()))
        .style(move |_theme, _status| {
            let base = iced::widget::button::Style::default();
            if node.is_current {
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.5, 0.9))),
                    text_color: Color::WHITE,
                    ..base
                }
            } else {
                base
            }
        })
        .width(Length::Fill);

    let expand_path = node.path.clone();
    let row_with_expand = if node.children.is_empty() {
        row_button
    } else {
        button(row_button)
            .on_press(Message::ToggleFolderExpand(expand_path))
            .style(iced::widget::button::text)
            .width(Length::Fill)
    };

    let children: Vec<Element<'a, Message>> = if node.is_expanded && !node.children.is_empty() {
        vec![
            row_with_expand.into(),
            container(
                column(
                    node.children
                        .iter()
                        .map(|child| render_node(child, depth + 1))
                        .collect::<Vec<_>>(),
                )
                .spacing(0),
            )
            .padding(iced::Padding::new(0.).left(INDENT_WIDTH))
            .into(),
        ]
    } else {
        vec![row_with_expand.into()]
    };

    column(children).spacing(0).into()
}
