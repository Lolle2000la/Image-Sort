use iced::window;
use iced::{Element, Subscription, Task};

use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::rename_action::RenameAction;
use media_sort_core::actions::reversible::ReversibleAction;
use media_sort_core::media_type::MediaType;

use crate::message::Message;
use crate::state::AppState;
use crate::subscriptions::keyboard;
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
            let filtered_len = state.filtered_media_entries().len();
            if index < filtered_len {
                state.selected_index = Some(index);
                state.current_metadata = None;
                return load_metadata(state, index);
            }
            Task::none()
        }
        Message::SearchQueryChanged(query) => {
            state.search_query = query;
            state.selected_index = None;
            state.current_metadata = None;
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
                                state.current_metadata = None;
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
                        state.current_metadata = None;
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
                state.current_metadata = None;
            }
            Task::none()
        }
        Message::Redo => {
            if let Err(e) = state.history.redo() {
                log::error!("Redo failed: {e}");
            } else {
                state.scan_media();
                state.selected_index = None;
                state.current_metadata = None;
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

        Message::ToggleMetadataPanel => {
            state.metadata_panel_expanded = !state.metadata_panel_expanded;
            state.settings.metadata_panel.is_expanded = state.metadata_panel_expanded;
            let _ = state.settings.save();
            Task::none()
        }

        Message::MetadataLoaded(result) => match result {
            Ok(metadata) => {
                state.current_metadata = Some(metadata);
                Task::none()
            }
            Err(err) => {
                log::error!("Metadata load failed: {err}");
                state.current_metadata = None;
                Task::none()
            }
        },

        Message::EditKeyBinding(index) => {
            state.editing_keybinding = Some(index);
            state.waiting_for_key = true;
            Task::none()
        }
        Message::KeyCaptured(key, ctrl, shift, alt) => {
            if state.waiting_for_key {
                if let Some(idx) = state.editing_keybinding {
                    let bindings = keyboard::keybinding_list(state);
                    if idx < bindings.len() {
                        let (name, _) = &bindings[idx];
                        keyboard::update_keybinding(
                            &mut state.settings.keybindings,
                            name,
                            &key,
                            ctrl,
                            shift,
                            alt,
                        );
                        let _ = state.settings.save();
                    }
                }
                state.waiting_for_key = false;
                state.editing_keybinding = None;
                return Task::none();
            }

            let bindings = keyboard::keybinding_list(state);
            for (name, binding) in &bindings {
                if binding.key == key
                    && binding.ctrl == ctrl
                    && binding.shift == shift
                    && binding.alt == alt
                {
                    match name.as_str() {
                        "undo" if state.history.can_undo() => {
                            return Task::done(Message::Undo);
                        }
                        "redo" if state.history.can_redo() => {
                            return Task::done(Message::Redo);
                        }
                        "open_folder" => {
                            if let Ok(p) = std::env::current_dir() {
                                return Task::done(Message::OpenFolder(p));
                            }
                        }
                        "toggle_metadata_panel" => {
                            return Task::done(Message::ToggleMetadataPanel);
                        }
                        "pin" => {
                            return Task::done(Message::PinCurrentFolder);
                        }
                        "unpin" => {
                            if let Some(ref c) = state.current_folder {
                                return Task::done(Message::UnpinCurrentFolder(c.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Task::none()
        }

        Message::OpenSettings => {
            state.show_settings = true;
            Task::none()
        }
        Message::CloseSettings => {
            state.show_settings = false;
            state.editing_keybinding = None;
            state.waiting_for_key = false;
            Task::none()
        }
        Message::ToggleDarkMode => {
            state.settings.general.dark_mode = !state.settings.general.dark_mode;
            Task::none()
        }
        Message::ToggleAnimateGifs => {
            state.settings.general.animate_gifs = !state.settings.general.animate_gifs;
            Task::none()
        }
        Message::ToggleAnimateThumbnails => {
            state.settings.general.animate_gif_thumbnails =
                !state.settings.general.animate_gif_thumbnails;
            Task::none()
        }
        Message::SaveSettings => {
            let _ = state.settings.save();
            state.show_settings = false;
            Task::none()
        }

        Message::PlayAudio => {
            if let Some(ref player) = state.audio_player {
                if let Some(index) = state.selected_index {
                    let entries = state.filtered_media_entries();
                    if let Some(entry) = entries.get(index) {
                        if let Err(e) = player.play(&entry.path) {
                            log::error!("Audio play failed: {e}");
                        }
                    }
                }
            }
            Task::none()
        }
        Message::PauseAudio => {
            if let Some(ref player) = state.audio_player {
                player.pause();
            }
            Task::none()
        }
        Message::StopAudio => {
            if let Some(ref player) = state.audio_player {
                player.stop();
            }
            Task::none()
        }

        Message::ThumbnailReady(path, data) => {
            if !data.is_empty() {
                state.thumbnail_cache.push(path, data);
            }
            Task::none()
        }
    }
}

fn load_metadata(state: &AppState, index: usize) -> Task<Message> {
    let entries = state.filtered_media_entries();
    let Some(entry) = entries.get(index) else {
        return Task::none();
    };

    let path = entry.path.clone();
    let media_type = entry.media_type;

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || match media_type {
                MediaType::Image => {
                    media_sort_backend::metadata::image_meta::extract_image_metadata(&path)
                        .map_err(|e| e.to_string())
                }
                MediaType::Audio => {
                    media_sort_backend::metadata::audio_meta::extract_audio_metadata(&path)
                        .map_err(|e| e.to_string())
                }
                MediaType::Video => {
                    media_sort_backend::metadata::video_meta::extract_video_metadata(&path)
                        .map_err(|e| e.to_string())
                }
            })
            .await
            .unwrap_or_else(|e| Err(format!("Join error: {e}")))
        },
        Message::MetadataLoaded,
    )
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    view::main_layout::main_layout_view(state)
}

pub fn theme(state: &AppState) -> iced::Theme {
    if state.settings.general.dark_mode {
        iced::Theme::Dark
    } else {
        iced::Theme::Light
    }
}

pub fn subscription(_state: &AppState) -> Subscription<Message> {
    let tick_sub = iced::time::every(std::time::Duration::from_millis(16)).map(Message::Tick);

    let keyboard_sub = crate::subscriptions::keyboard::keyboard_subscription();

    Subscription::batch(vec![tick_sub, keyboard_sub])
}
