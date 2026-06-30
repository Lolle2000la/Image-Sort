use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn metadata_panel_view(state: &AppState) -> Element<'_, Message> {
    if !state.metadata_panel_expanded {
        return iced::widget::Space::new().width(Length::Fixed(0.0)).height(Length::Fixed(0.0)).into();
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
                } else if section_name == "Container Metadata" {
                    state.l10n.tr("metadata-section-container")
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
                    let display_tag = localize_tag_name(section_name, tag_name, &state.l10n);

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

fn localize_tag_name(section: &str, key: &str, l10n: &media_sort_core::l10n::Localization) -> String {
    let key_upper = key.to_ascii_uppercase();
    if section == "File" {
        match key_upper.as_str() {
            "NAME" => l10n.tr("metadata-field-name"),
            "SIZE" => l10n.tr("metadata-field-size"),
            "MODIFIED" => l10n.tr("metadata-field-modified"),
            "DIMENSIONS" => l10n.tr("metadata-field-dimensions"),
            _ => key.to_string(),
        }
    } else {
        match key_upper.as_str() {
            "DURATION" => l10n.tr("metadata-field-duration"),
            "ENCODER" => l10n.tr("metadata-field-encoder"),
            "TITLE" | "TRACKTITLE" => l10n.tr("metadata-field-title"),
            "ARTIST" | "ALBUMARTIST" => l10n.tr("metadata-field-artist"),
            "ALBUM" => l10n.tr("metadata-field-album"),
            "GENRE" => l10n.tr("metadata-field-genre"),
            "DATE" | "YEAR" => l10n.tr("metadata-field-date"),
            "TRACKNUMBER" | "TRACK" => l10n.tr("metadata-field-track"),
            "COMMENT" => l10n.tr("metadata-field-comment"),
            "DESCRIPTION" => l10n.tr("metadata-field-description"),
            "COPYRIGHT" => l10n.tr("metadata-field-copyright"),
            "LANGUAGE" => l10n.tr("metadata-field-language"),
            _ => key.to_string(),
        }
    }
}
