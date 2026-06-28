use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn metadata_panel_view(state: &AppState) -> Element<'_, Message> {
    if !state.metadata_panel_expanded {
        let toggle_btn = button(text(format!("> {}", state.l10n.tr("ui-metadata"))))
            .on_press(Message::ToggleMetadataPanel)
            .style(iced::widget::button::secondary);
        return container(toggle_btn).padding(4).into();
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
                let section_header =
                    text(format!("[{}]", section_name))
                        .size(12)
                        .style(move |_theme| text::Style {
                            color: Some(Color::from_rgb(0.6, 0.7, 0.9)),
                        });

                let mut field_list = column![].spacing(2);
                for (tag_name, value) in fields {
                    let line = row![
                        text(format!("{}:", tag_name)).size(11),
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
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
            border: iced::Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..iced::widget::container::Style::default()
        })
        .into()
}
