use iced::widget::{column, container, mouse_area, row, text};
use iced::{Color, Element, Length};

use crate::message::{MediaMessage, Message, SettingsMessage};
use crate::state::AppState;
use crate::view::{
    control_panel, folder_panel, media_grid, media_preview, metadata_panel, search_bar,
};

pub fn main_layout_view(state: &AppState) -> Element<'_, Message> {
    let folder_panel = folder_panel::folder_panel_view(state);
    let control_panel = control_panel::control_panel_view(state);

    let search_bar = search_bar::search_bar_view(&state.search_query, &state.search_placeholder);
    let grid = media_grid::media_grid_view(state);
    let preview = media_preview::media_preview_view(state);
    let metadata = metadata_panel::metadata_panel_view(state);

    let can_move_or_copy = state.selected_index.is_some()
        && state.selected_folder.is_some()
        && state
            .selected_folder
            .as_ref()
            .zip(state.current_folder.as_ref())
            .is_none_or(|(sf, cf)| !media_sort_core::path_utils::paths_equal(sf, cf));

    let move_btn = {
        let btn_content = row![
            text(state.l10n.tr("ui-move")),
            text(char::from(lucide_icons::Icon::ArrowUp))
                .font(iced::Font::with_name("lucide"))
                .size(14),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);
        let inner = iced::widget::button(btn_content).width(Length::Fill);
        let inner = if can_move_or_copy {
            inner.on_press(Message::Media(MediaMessage::MoveActive))
        } else {
            inner
        };
        container(inner)
            .id(iced::widget::Id::new("move_btn"))
            .width(Length::Fill)
    };

    let copy_btn = {
        let btn_content = row![
            text(state.l10n.tr("ui-copy")),
            text(char::from(lucide_icons::Icon::Copy))
                .font(iced::Font::with_name("lucide"))
                .size(14),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);
        let btn = iced::widget::button(btn_content).width(Length::Fill);
        let inner = if can_move_or_copy {
            btn.on_press(Message::Media(MediaMessage::CopyActive))
        } else {
            btn
        };
        container(inner)
            .id(iced::widget::Id::new("copy_btn"))
            .width(Length::Fill)
    };

    // Rename (R) button next to search bar:
    let rename_btn = {
        let btn = iced::widget::button(text(state.l10n.tr("ui-rename")).size(13));
        if state.selected_index.is_some() {
            btn.on_press(Message::Media(MediaMessage::TriggerRename))
        } else {
            btn
        }
    };

    // Delete (Down Arrow) button at the bottom:
    let delete_btn = {
        let btn_content = row![
            text(state.l10n.tr("ui-delete")),
            text(char::from(lucide_icons::Icon::ArrowDown))
                .font(iced::Font::with_name("lucide"))
                .size(14),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);
        let btn = iced::widget::button(btn_content)
            .width(Length::Fill)
            .style(iced::widget::button::danger);
        let inner = if let Some(index) = state.selected_index {
            let filtered = state.filtered_media_entries();
            if let Some(entry) = filtered.get(index) {
                btn.on_press(Message::Media(MediaMessage::DeleteEntry(
                    entry.path.clone(),
                )))
            } else {
                btn
            }
        } else {
            btn
        };
        container(inner)
            .id(iced::widget::Id::new("delete_btn"))
            .width(Length::Fill)
    };

    // Metadata button:
    let metadata_btn = iced::widget::button(text(state.l10n.tr("ui-metadata")).size(13))
        .on_press(Message::Settings(SettingsMessage::ToggleMetadataPanel));

    // Row containing the search bar, Rename button, and Metadata button:
    let search_rename_row = row![search_bar, rename_btn, metadata_btn,]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    // Center area (the preview area and bottom action area):
    let media_column = column![
        row![move_btn, copy_btn].spacing(4),
        container(preview).height(Length::Fill).width(Length::Fill),
        search_rename_row,
        grid,
        delete_btn,
    ]
    .spacing(8)
    .height(Length::Fill);

    let divider = divider_view(Message::Settings(SettingsMessage::StartDragFolderDivider));

    let media_metadata_row = if state.metadata_panel_expanded {
        row![
            container(media_column)
                .width(Length::Fill)
                .height(Length::Fill),
            divider_view(Message::Settings(SettingsMessage::StartDragMetadataDivider)),
            metadata,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        row![
            container(media_column)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
    };

    let main_content = row![
        folder_panel,
        divider,
        control_panel,
        iced::widget::Space::new().width(Length::Fixed(8.0)),
        media_metadata_row,
    ]
    .spacing(0)
    .height(Length::Fill);

    let result = container(main_content).padding(8);

    let mut overlays = Vec::new();

    if state.show_settings {
        overlays.push(crate::view::settings_dialog::settings_dialog_view(state));
    }

    if state.show_credits {
        overlays.push(crate::view::credits_dialog::credits_dialog_view(state));
    }

    #[cfg(feature = "velopack")]
    if state.show_update_prompt {
        overlays.push(crate::view::update_prompt::update_prompt_view(state));
    }

    if let Some(ref path) = state.renaming_path {
        overlays
            .push(crate::widgets::rename_modal::rename_modal_view(state, path).map(Message::Media));
    }

    if let Some(ref parent) = state.creating_folder_parent {
        overlays.push(
            crate::widgets::create_folder_modal::create_folder_modal_view(state, parent)
                .map(Message::Folder),
        );
    }

    if !overlays.is_empty() {
        let mut stack = iced::widget::stack![result];
        for overlay in overlays {
            stack = stack.push(
                container(overlay)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.6,
                        ))),
                        ..iced::widget::container::Style::default()
                    }),
            );
        }
        return stack.into();
    }

    result.into()
}

fn divider_view<'a>(on_press: Message) -> Element<'a, Message> {
    mouse_area(
        container(
            container(
                iced::widget::Space::new()
                    .width(Length::Fixed(2.0))
                    .height(Length::Fill),
            )
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color {
                        a: 0.15,
                        ..palette.text
                    })),
                    ..Default::default()
                }
            }),
        )
        .width(Length::Fixed(8.0))
        .height(Length::Fill)
        .center_x(8.0),
    )
    .on_press(on_press)
    .interaction(iced::mouse::Interaction::ResizingHorizontally)
    .into()
}
