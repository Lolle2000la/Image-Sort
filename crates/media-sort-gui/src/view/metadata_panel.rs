use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn metadata_panel_view(state: &AppState) -> Element<'_, Message> {
    if !state.metadata_panel_expanded {
        let label = state.l10n.tr("ui-metadata");
        let vertical_text: String = label.chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let toggle_btn = button(
            column![
                text("<").size(10).align_x(iced::Alignment::Center),
                text(vertical_text).size(10).line_height(1.1).align_x(iced::Alignment::Center),
            ]
            .align_x(iced::Alignment::Center)
            .spacing(4)
        )
        .on_press(Message::ToggleMetadataPanel)
        .style(iced::widget::button::secondary)
        .padding(6);

        return container(toggle_btn)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .padding(4)
            .into();
    }

    let header_row = row![
        text(state.l10n.tr("ui-metadata")).size(16),
        iced::widget::Space::new().width(Length::Fill),
        button(text("X"))
            .on_press(Message::ToggleMetadataPanel)
            .style(iced::widget::button::text),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let content: Element<'_, Message> = match &state.current_metadata {
        Some(metadata) => {
            let mut section_list = column![].spacing(8);

            for (section_name, fields) in metadata {
                let display_section = if section_name == "File" {
                    state.l10n.tr("metadata-section-file")
                } else {
                    section_name.clone()
                };

                let section_header =
                    text(format!("[{}]", display_section))
                        .size(12)
                        .style(move |theme: &iced::Theme| text::Style {
                            color: Some(theme.palette().primary),
                        });

                let mut field_list = column![].spacing(2);
                for (tag_name, value) in fields {
                    let display_tag = if section_name == "File" {
                        match tag_name.as_str() {
                            "Name" => state.l10n.tr("metadata-field-name"),
                            "Size" => state.l10n.tr("metadata-field-size"),
                            "Modified" => state.l10n.tr("metadata-field-modified"),
                            "Dimensions" => state.l10n.tr("metadata-field-dimensions"),
                            _ => tag_name.clone(),
                        }
                    } else {
                        tag_name.clone()
                    };

                    let line = row![
                        text(format!("{}:", display_tag)).size(11),
                        text(value).size(11).shaping(iced::widget::text::Shaping::Advanced),
                    ]
                    .spacing(4);
                    field_list = field_list.push(line);
                }

                section_list = section_list.push(column![section_header, field_list].spacing(4));
            }

            scrollable(section_list).height(Length::Fill).into()
        }
        None => {
            if state.selected_index.is_some() {
                container(text(state.l10n.tr("ui-loading-metadata")).size(12))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            } else {
                container(text(state.l10n.tr("ui-select-file-metadata")).size(12))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
        }
    };

    container(column![header_row, content].spacing(8).height(Length::Fill))
        .padding(8)
        .width(Length::Fixed(f32::from(
            state.settings.metadata_panel.panel_width,
        )))
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            iced::widget::container::Style {
                background: Some(iced::Background::Color(palette.background)),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 1.0,
                    color: Color { a: 0.15, ..palette.text },
                },
                ..iced::widget::container::Style::default()
            }
        })
        .into()
}
