use std::path::PathBuf;

use iced::window;
use iced::{Element, Subscription, Task};

use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::rename_action::RenameAction;
use media_sort_core::actions::reversible::ReversibleAction;

use crate::message::Message;
use crate::state::AppState;
use crate::view;

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Tick(_instant) => {
            if state.should_exit {
                let _ = state.settings.save();
                return window::get_latest().and_then(window::close);
            }
            Task::none()
        }
        Message::SettingsLoaded(result) => match *result {
            Ok(settings) => {
                state.settings = settings;
                Task::none()
            }
            Err(err) => {
                log::error!("Failed to load settings: {err}");
                Task::none()
            }
        },
        Message::Quit => {
            let _ = state.settings.save();
            state.should_exit = true;
            Task::none()
        }

        Message::OpenFolder(path) => {
            state.open_folder(&path);
            Task::none()
        }
        Message::FolderSelected(path) => {
            state.open_folder(&path);
            Task::none()
        }
        Message::ToggleFolderExpand(path) => {
            state.toggle_folder_expand(&path);
            Task::none()
        }

        Message::SelectEntry(index) => {
            if index < state.filtered_media_entries().len() {
                state.selected_index = Some(index);
            }
            Task::none()
        }
        Message::SearchQueryChanged(query) => {
            state.search_query = query;
            state.selected_index = None;
            Task::none()
        }

        Message::MoveToFolder(target_folder) => {
            if let Some(index) = state.selected_index {
                let filtered = state.filtered_media_entries();
                if let Some(entry) = filtered.get(index) {
                    match MoveAction::new(&entry.path, &target_folder) {
                        Ok(mut action) => {
                            if let Err(e) = action.execute() {
                                log::error!("Move failed: {e}");
                            } else {
                                state.history.push_executed(Box::new(action));
                                state.scan_media();
                                state.selected_index = None;
                            }
                        }
                        Err(e) => {
                            log::error!("Cannot create move action: {e}");
                        }
                    }
                }
            }
            Task::none()
        }
        Message::DeleteEntry(path) => {
            match media_sort_backend::filesystem::trash_staging::TrashStaging::new() {
                Ok(staging) => match staging.stage_file(&path) {
                    Ok(handle) => {
                        let action = media_sort_core::actions::delete_action::DeleteAction::new(
                            &path, handle,
                        );
                        state.history.push_executed(Box::new(action));
                        state.scan_media();
                        state.selected_index = None;
                    }
                    Err(e) => {
                        log::error!("Cannot stage file for deletion: {e}");
                    }
                },
                Err(e) => {
                    log::error!("Cannot create trash staging: {e}");
                }
            }
            Task::none()
        }
        Message::RenameEntry(path, new_name) => {
            match RenameAction::new(&path, &new_name) {
                Ok(mut action) => {
                    if let Err(e) = action.execute() {
                        log::error!("Rename failed: {e}");
                    } else {
                        state.history.push_executed(Box::new(action));
                        state.scan_media();
                    }
                }
                Err(e) => {
                    log::error!("Cannot create rename action: {e}");
                }
            }
            Task::none()
        }

        Message::Undo => {
            if let Err(e) = state.history.undo() {
                log::error!("Undo failed: {e}");
            } else {
                state.scan_media();
                state.selected_index = None;
            }
            Task::none()
        }
        Message::Redo => {
            if let Err(e) = state.history.redo() {
                log::error!("Redo failed: {e}");
            } else {
                state.scan_media();
                state.selected_index = None;
            }
            Task::none()
        }

        Message::PinCurrentFolder => {
            state.pin_current_folder();
            let _ = state.settings.save();
            Task::none()
        }
        Message::UnpinCurrentFolder(path) => {
            state.unpin_folder(&path);
            let _ = state.settings.save();
            Task::none()
        }
    }
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    use iced::widget::{column, container, horizontal_rule, row, text};

    let folder_panel = view::folder_panel::folder_panel_view(state);

    let search_bar = view::search_bar::search_bar_view(&state.search_query);

    let media_grid = view::media_grid::media_grid_view(state);
    let media_preview = view::media_preview::media_preview_view(state);

    let mut history_row = row![].spacing(8);
    if state.history.can_undo() {
        history_row = history_row.push(
            iced::widget::button(text(format!(
                "Undo ({})",
                state.history.last_done_name().unwrap_or("")
            )))
            .on_press(Message::Undo)
            .style(iced::widget::button::secondary),
        );
    }
    if state.history.can_redo() {
        history_row = history_row.push(
            iced::widget::button(text(format!(
                "Redo ({})",
                state.history.last_undone_name().unwrap_or("")
            )))
            .on_press(Message::Redo)
            .style(iced::widget::button::secondary),
        );
    }

    let main_content = column![
        search_bar,
        horizontal_rule(1),
        row![
            column![media_grid]
                .width(iced::Length::FillPortion(1))
                .height(iced::Length::Fill),
            column![media_preview]
                .width(iced::Length::FillPortion(1))
                .height(iced::Length::Fill),
        ]
        .height(iced::Length::Fill)
        .spacing(8),
        horizontal_rule(1),
        history_row,
    ]
    .spacing(4);

    let top_row = row![text("Media Sort v3.0.0").size(18), {
        iced::widget::button(text("Open Folder"))
            .on_press({
                let path = std::env::current_dir().ok();
                Message::OpenFolder(path.unwrap_or_else(|| PathBuf::from(".")))
            })
            .style(iced::widget::button::primary)
    },]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let app_layout = column![
        top_row,
        horizontal_rule(1),
        row![folder_panel, main_content]
            .spacing(4)
            .height(iced::Length::Fill),
    ]
    .spacing(4)
    .padding(8);

    container(app_layout).into()
}

pub fn theme(state: &AppState) -> iced::Theme {
    if state.settings.general.dark_mode {
        iced::Theme::Dark
    } else {
        iced::Theme::Light
    }
}

pub fn subscription(_state: &AppState) -> Subscription<Message> {
    iced::time::every(std::time::Duration::from_millis(16)).map(Message::Tick)
}
