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

#[cfg(test)]
mod tests {
    use super::*;
    use media_sort_core::actions::rename_action::RenameAction;
    use media_sort_core::media_type::MediaType;
    use media_sort_core::models::MediaEntry;
    use media_sort_core::settings::store::SettingsStore;
    use std::path::PathBuf;

    #[test]
    fn test_select_entry_in_bounds() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![MediaEntry {
            path: PathBuf::from("/test/a.jpg"),
            media_type: MediaType::Image,
            file_name: "a.jpg".into(),
        }];
        state.search_query = String::new();
        let _task = update(&mut state, Message::SelectEntry(0));
        assert_eq!(state.selected_index, Some(0));
        assert!(state.current_metadata.is_none());
    }

    #[test]
    fn test_select_entry_out_of_bounds() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![];
        state.search_query = String::new();
        state.selected_index = None;
        let _task = update(&mut state, Message::SelectEntry(99));
        assert_eq!(state.selected_index, None);
    }

    #[test]
    fn test_select_entry_filtered_empty() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![MediaEntry {
            path: PathBuf::from("/test/a.jpg"),
            media_type: MediaType::Image,
            file_name: "a.jpg".into(),
        }];
        state.search_query = "nomatch".into();
        state.selected_index = None;
        let _task = update(&mut state, Message::SelectEntry(0));
        assert_eq!(state.selected_index, None);
    }

    fn setup_temp_rename_action(dir_prefix: &str) -> (std::path::PathBuf, RenameAction) {
        let dir = std::env::temp_dir().join(format!("{}_{}", dir_prefix, std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.txt");
        std::fs::write(&file, b"content").unwrap();
        let mut action = RenameAction::new(&file, "renamed").unwrap();
        action.execute().unwrap();
        (dir, action)
    }

    #[test]
    fn test_keycaptured_undo_when_history_has_actions() {
        let mut state = AppState::new(SettingsStore::default());
        let (dir, action) = setup_temp_rename_action("mediasort_undo");

        state.history.push_executed(Box::new(action));
        assert!(state.history.can_undo());

        let _ = update(
            &mut state,
            Message::KeyCaptured("Q".into(), false, false, false),
        );
        let _ = update(&mut state, Message::Undo);
        assert!(state.history.can_redo());
        assert!(!state.history.can_undo());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_keycaptured_undo_when_history_empty() {
        let mut state = AppState::new(SettingsStore::default());
        assert!(!state.history.can_undo());

        let _task = update(
            &mut state,
            Message::KeyCaptured("Q".into(), false, false, false),
        );
        assert!(!state.history.can_undo());
        assert!(!state.history.can_redo());
    }

    #[test]
    fn test_keycaptured_redo_when_history_has_undone() {
        let mut state = AppState::new(SettingsStore::default());
        let (dir, action) = setup_temp_rename_action("mediasort_redo");

        state.history.push_executed(Box::new(action));
        state.history.undo().unwrap();
        assert!(state.history.can_redo());
        assert!(!state.history.can_undo());

        let _ = update(
            &mut state,
            Message::KeyCaptured("E".into(), false, false, false),
        );
        let _ = update(&mut state, Message::Redo);
        assert!(!state.history.can_redo());
        assert!(state.history.can_undo());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_keycaptured_capture_mode_updates_binding() {
        let mut state = AppState::new(SettingsStore::default());
        state.waiting_for_key = true;
        state.editing_keybinding = Some(0);

        let _task = update(
            &mut state,
            Message::KeyCaptured("X".into(), true, false, false),
        );

        assert!(!state.waiting_for_key);
        assert_eq!(state.editing_keybinding, None);
        let kb = &state.settings.keybindings;
        assert_eq!(kb.move_to_folder.key, "X");
        assert!(kb.move_to_folder.ctrl);
        assert!(!kb.move_to_folder.shift);
        assert!(!kb.move_to_folder.alt);
    }

    #[test]
    fn test_keycaptured_capture_mode_clears_editing_state() {
        let mut state = AppState::new(SettingsStore::default());
        state.waiting_for_key = true;
        state.editing_keybinding = Some(3);

        let _task = update(
            &mut state,
            Message::KeyCaptured("Left".into(), false, false, false),
        );

        assert!(!state.waiting_for_key);
        assert_eq!(state.editing_keybinding, None);
    }

    #[test]
    fn test_keycaptured_toggle_metadata_panel() {
        let mut state = AppState::new(SettingsStore::default());
        assert!(!state.metadata_panel_expanded);

        let _ = update(
            &mut state,
            Message::KeyCaptured("M".into(), false, false, false),
        );
        let _ = update(&mut state, Message::ToggleMetadataPanel);
        assert!(state.metadata_panel_expanded);

        let _ = update(
            &mut state,
            Message::KeyCaptured("M".into(), false, false, false),
        );
        let _ = update(&mut state, Message::ToggleMetadataPanel);
        assert!(!state.metadata_panel_expanded);
    }

    #[test]
    fn test_keycaptured_pin_triggers_pin() {
        let mut state = AppState::new(SettingsStore::default());
        state.current_folder = Some(PathBuf::from("/test/folder"));
        assert!(state.pinned_folders.is_empty());

        let _ = update(
            &mut state,
            Message::KeyCaptured("P".into(), false, false, false),
        );
        let _ = update(&mut state, Message::PinCurrentFolder);
        assert_eq!(state.pinned_folders.len(), 1);
    }

    #[test]
    fn test_keycaptured_unpin_triggers_unpin() {
        let mut state = AppState::new(SettingsStore::default());
        let folder = PathBuf::from("/test/unpin_dir");
        state.current_folder = Some(folder.clone());
        state.pin_current_folder();
        assert_eq!(state.pinned_folders.len(), 1);

        let _ = update(
            &mut state,
            Message::KeyCaptured("U".into(), false, false, false),
        );
        let _ = update(&mut state, Message::UnpinCurrentFolder(folder.clone()));
        assert!(state.pinned_folders.is_empty());
    }

    #[test]
    fn test_keycaptured_pin_without_folder_is_noop() {
        let mut state = AppState::new(SettingsStore::default());
        state.current_folder = None;
        assert!(state.pinned_folders.is_empty());

        let _task = update(
            &mut state,
            Message::KeyCaptured("P".into(), false, false, false),
        );
        assert!(state.pinned_folders.is_empty());
    }

    #[test]
    fn test_keycaptured_unknown_binding_is_noop() {
        let mut state = AppState::new(SettingsStore::default());
        let saved_undo = state.history.can_undo();
        let _task = update(
            &mut state,
            Message::KeyCaptured("F9".into(), false, false, false),
        );
        assert_eq!(state.history.can_undo(), saved_undo);
        assert!(!state.metadata_panel_expanded);
    }

    fn setup_temp_dir_with_files(
        name: &str,
    ) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("mediasort_{}_{}", name, std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("test_image.jpg");
        std::fs::write(&file, b"fake jpeg data").unwrap();
        let dest = root.join("subfolder");
        std::fs::create_dir_all(&dest).unwrap();
        (root, file, dest)
    }

    fn setup_data_dir_with_files(
        name: &str,
    ) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("media-sort")
            .join("test");
        let root = base.join(format!("{}_{}", name, std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("test_image.jpg");
        std::fs::write(&file, b"fake jpeg data").unwrap();
        let dest = root.join("subfolder");
        std::fs::create_dir_all(&dest).unwrap();
        (root, file, dest)
    }

    #[test]
    fn test_move_to_folder_success() {
        let (root, file, dest) = setup_temp_dir_with_files("move_ok");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(0);

        assert!(file.exists());
        let dest_file = dest.join("test_image.jpg");
        assert!(!dest_file.exists());

        let _task = update(&mut state, Message::MoveToFolder(dest.clone()));

        assert!(!file.exists());
        assert!(dest_file.exists());
        assert!(state.history.can_undo());
        assert_eq!(state.history.done_len(), 1);
        assert_eq!(state.selected_index, None);
        assert!(state.media_entries.is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_move_to_folder_no_selection_is_noop() {
        let (root, _file, dest) = setup_temp_dir_with_files("move_nosel");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = None;

        let _task = update(&mut state, Message::MoveToFolder(dest.clone()));

        assert!(!state.history.can_undo());
        assert!(state.selected_index.is_none());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_move_to_folder_index_out_of_bounds() {
        let (root, _file, dest) = setup_temp_dir_with_files("move_oob");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(999);

        let _task = update(&mut state, Message::MoveToFolder(dest.clone()));

        assert!(!state.history.can_undo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_move_to_folder_nonexistent_target() {
        let (root, file, _dest) = setup_temp_dir_with_files("move_nodir");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(0);

        let nonexistent = root.join("does_not_exist");

        let _task = update(&mut state, Message::MoveToFolder(nonexistent));

        assert!(file.exists());
        assert!(!state.history.can_undo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_delete_entry_success() {
        let (root, file, _dest) = setup_data_dir_with_files("delete_ok");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);

        assert!(file.exists());

        let _task = update(&mut state, Message::DeleteEntry(file.clone()));

        assert!(!file.exists());
        assert!(state.history.can_undo());
        assert_eq!(state.history.done_len(), 1);
        assert_eq!(state.selected_index, None);
        assert!(state.media_entries.is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_delete_entry_nonexistent_file() {
        let (root, _file, _dest) = setup_data_dir_with_files("delete_nofile");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);

        let nonexistent = root.join("does_not_exist.jpg");

        let _task = update(&mut state, Message::DeleteEntry(nonexistent));

        assert!(!state.history.can_undo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_undo_after_move() {
        let (root, file, dest) = setup_temp_dir_with_files("undo_move");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(0);

        let _ = update(&mut state, Message::MoveToFolder(dest.clone()));
        assert!(!file.exists());
        let dest_file = dest.join("test_image.jpg");
        assert!(dest_file.exists());
        assert!(state.history.can_undo());

        let _task = update(&mut state, Message::Undo);

        assert!(file.exists());
        assert!(!dest_file.exists());
        assert!(!state.history.can_undo());
        assert!(state.history.can_redo());
        assert_eq!(state.selected_index, None);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_undo_after_delete() {
        let (root, file, _dest) = setup_data_dir_with_files("undo_delete");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);

        let _ = update(&mut state, Message::DeleteEntry(file.clone()));
        assert!(!file.exists());
        assert!(state.history.can_undo());

        let _task = update(&mut state, Message::Undo);

        assert!(file.exists());
        assert!(!state.history.can_undo());
        assert!(state.history.can_redo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_redo_after_undo_move() {
        let (root, file, dest) = setup_temp_dir_with_files("redo_move");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(0);

        let _ = update(&mut state, Message::MoveToFolder(dest.clone()));
        let _ = update(&mut state, Message::Undo);
        assert!(file.exists());
        assert!(state.history.can_redo());

        let _task = update(&mut state, Message::Redo);

        assert!(!file.exists());
        let dest_file = dest.join("test_image.jpg");
        assert!(dest_file.exists());
        assert!(state.history.can_undo());
        assert!(!state.history.can_redo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_undo_empty_history_no_panic() {
        let mut state = AppState::new(SettingsStore::default());
        assert!(!state.history.can_undo());

        let _task = update(&mut state, Message::Undo);
        assert!(!state.history.can_undo());
    }

    #[test]
    fn test_redo_empty_undone_no_panic() {
        let mut state = AppState::new(SettingsStore::default());
        assert!(!state.history.can_redo());

        let _task = update(&mut state, Message::Redo);
        assert!(!state.history.can_redo());
    }

    #[test]
    fn test_rename_entry_success() {
        let (root, file, _dest) = setup_temp_dir_with_files("rename_ok");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(0);

        assert!(file.exists());

        let _task = update(
            &mut state,
            Message::RenameEntry(file.clone(), "renamed_image".to_string()),
        );

        assert!(!file.exists());
        let renamed = root.join("renamed_image.jpg");
        assert!(renamed.exists());
        assert!(state.history.can_undo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_rename_entry_target_exists_is_noop() {
        let root =
            std::env::temp_dir().join(format!("mediasort_rename_conflict_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file1 = root.join("a.jpg");
        let file2 = root.join("b.jpg");
        std::fs::write(&file1, b"a").unwrap();
        std::fs::write(&file2, b"b").unwrap();

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);

        let _task = update(
            &mut state,
            Message::RenameEntry(file1.clone(), "b".to_string()),
        );

        assert!(file1.exists());
        assert!(file2.exists());
        assert!(!state.history.can_undo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_move_across_filesystems() {
        let root = std::env::temp_dir().join(format!("mediasort_xdev_src_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("test.jpg");
        std::fs::write(&file, b"cross-filesystem data").unwrap();

        let dest = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!("mediasort_xdev_dst_{}", std::process::id()));
        std::fs::create_dir_all(&dest).unwrap();

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.selected_index = Some(0);

        let _task = update(&mut state, Message::MoveToFolder(dest.clone()));

        assert!(!file.exists());
        let moved_file = dest.join("test.jpg");
        assert!(moved_file.exists());
        assert!(state.history.can_undo());

        let content = std::fs::read_to_string(&moved_file).unwrap();
        assert_eq!(content, "cross-filesystem data");

        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&dest).ok();
    }

    #[test]
    fn test_rename_or_copy_same_filesystem() {
        let dir = std::env::temp_dir().join(format!("mediasort_samefs_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("source.txt");
        let dst = dir.join("dest.txt");
        std::fs::write(&src, b"test data").unwrap();

        media_sort_core::path_utils::rename_or_copy_and_delete(&src, &dst).unwrap();
        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "test data");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_rename_or_copy_cross_filesystem() {
        let src_dir =
            std::env::temp_dir().join(format!("mediasort_xdev_test_src_{}", std::process::id()));
        std::fs::create_dir_all(&src_dir).unwrap();
        let src = src_dir.join("xdev_file.txt");
        std::fs::write(&src, b"cross-fs content").unwrap();

        let dst_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!("mediasort_xdev_test_dst_{}", std::process::id()));
        std::fs::create_dir_all(&dst_dir).unwrap();
        let dst = dst_dir.join("xdev_file.txt");

        let result = media_sort_core::path_utils::rename_or_copy_and_delete(&src, &dst);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "cross-fs content");

        std::fs::remove_dir_all(&src_dir).ok();
        let _ = std::fs::remove_file(&dst);
        let _ = std::fs::remove_dir(&dst_dir);
    }

    #[test]
    fn test_delete_undo_cross_filesystem() {
        let root = std::env::temp_dir().join(format!("mediasort_del_xdev_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("delete_me.jpg");
        std::fs::write(&file, b"delete me data").unwrap();

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        assert!(file.exists());

        let _task = update(&mut state, Message::DeleteEntry(file.clone()));
        assert!(!file.exists());
        assert!(state.history.can_undo());

        let _task = update(&mut state, Message::Undo);
        assert!(file.exists());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "delete me data");
        assert!(!state.history.can_undo());
        assert!(state.history.can_redo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_thumbnail_ready_empty_data() {
        let mut state = AppState::new(SettingsStore::default());
        let cache_size_before = state.thumbnail_cache.len();

        let _task = update(
            &mut state,
            Message::ThumbnailReady(std::path::PathBuf::from("/test/empty.jpg"), Vec::new()),
        );
        assert_eq!(state.thumbnail_cache.len(), cache_size_before);
    }

    #[test]
    fn test_thumbnail_ready_valid_data() {
        let mut state = AppState::new(SettingsStore::default());
        let path = std::path::PathBuf::from("/test/thumb.jpg");

        let _task = update(
            &mut state,
            Message::ThumbnailReady(path.clone(), vec![0x89, 0x50, 0x4E, 0x47]),
        );
        assert_eq!(state.thumbnail_cache.len(), 1);
        assert!(state.thumbnail_cache.contains(&path));
    }

    #[test]
    fn test_metadata_loaded_error_clears_metadata() {
        let mut state = AppState::new(SettingsStore::default());
        let mut existing = std::collections::BTreeMap::new();
        let mut inner = std::collections::BTreeMap::new();
        inner.insert("Width".to_string(), "1920".to_string());
        existing.insert("EXIF".to_string(), inner);
        state.current_metadata = Some(existing);

        let _task = update(
            &mut state,
            Message::MetadataLoaded(Err("load failed".to_string())),
        );
        assert!(state.current_metadata.is_none());
    }

    #[test]
    fn test_metadata_loaded_success() {
        let mut state = AppState::new(SettingsStore::default());
        let mut metadata = std::collections::BTreeMap::new();
        let mut section = std::collections::BTreeMap::new();
        section.insert("Width".to_string(), "1920".to_string());
        metadata.insert("EXIF".to_string(), section);

        let _task = update(&mut state, Message::MetadataLoaded(Ok(metadata)));
        assert!(state.current_metadata.is_some());
        let m = state.current_metadata.as_ref().unwrap();
        assert_eq!(m.get("EXIF").unwrap().get("Width").unwrap(), "1920");
    }

    #[test]
    fn test_tick_should_exit_saves_settings() {
        let mut state = AppState::new(SettingsStore::default());
        state.settings.general.dark_mode = true;
        state.should_exit = true;

        let _task = update(&mut state, Message::Tick(std::time::Instant::now()));
        let reloaded = SettingsStore::load().unwrap_or_default();
        assert!(reloaded.general.dark_mode);

        state.settings.general.dark_mode = false;
        state.settings.save().ok();
    }
}
