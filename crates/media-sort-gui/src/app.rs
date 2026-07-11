use iced::window;
use iced::{Element, Subscription, Task};

use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::rename_action::RenameAction;
use media_sort_core::actions::reversible::ReversibleAction;
use media_sort_core::media_type::MediaType;
use media_sort_core::models::MediaEntry;

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage, VideoMessage};
use crate::state::AppState;
use crate::subscriptions::keyboard;
use crate::view;

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    #[cfg(feature = "demo")]
    if let Some(task) =
        iced_automation::intercept_update(state, &message, Message::AutomationBounds, |state| {
            let mut bg_tasks = Vec::new();

            if let Some(ref rx) = state.folder_tree_receiver
                && let Ok(tree) = rx.try_recv()
            {
                state.folder_tree_receiver = None;
                state.folder_tree = tree;
                state.sync_selected_folder_idx();
            }

            let scan_finished = if let Some(ref rx) = state.scan_receiver {
                for path in rx.try_iter() {
                    let media_type =
                        crate::state::detect_media_type(&path, state.settings.general.animate_gifs);
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.display().to_string());
                    state.media_entries.push(MediaEntry {
                        path,
                        media_type,
                        file_name,
                    });
                }
                matches!(
                    rx.try_recv(),
                    Err(std::sync::mpsc::TryRecvError::Disconnected)
                )
            } else {
                false
            };

            if scan_finished {
                state.scan_receiver = None;
                state
                    .media_entries
                    .sort_by(|a, b| a.file_name.cmp(&b.file_name));
                let select_idx = state.pending_select_index.take().unwrap_or(0);
                state.thumbnail_tracker.cancel_debounce();
                bg_tasks.push(load_visible_thumbnails(state));
                bg_tasks.push(select_and_load_entry(state, select_idx));
            }

            Task::batch(bg_tasks)
        })
    {
        return task;
    }

    match message {
        Message::Tick(_instant) => {
            if state.should_exit {
                let _ = state.settings.save();
                return window::latest().and_then(window::close);
            }

            #[cfg(feature = "demo")]
            let automation_task =
                iced_automation::try_tick_state(state, _instant, update, Message::AutomationBounds);
            #[cfg(not(feature = "demo"))]
            let automation_task = Task::none();

            if let Some(ref rx) = state.folder_tree_receiver
                && let Ok(tree) = rx.try_recv()
            {
                state.folder_tree_receiver = None;
                state.folder_tree = tree;
                state.sync_selected_folder_idx();
            }

            let refresh_thumbnails = state.thumbnail_tracker.tick();

            let scan_finished = if let Some(ref rx) = state.scan_receiver {
                for path in rx.try_iter() {
                    let media_type =
                        crate::state::detect_media_type(&path, state.settings.general.animate_gifs);
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.display().to_string());
                    state.media_entries.push(MediaEntry {
                        path,
                        media_type,
                        file_name,
                    });
                }
                matches!(
                    rx.try_recv(),
                    Err(std::sync::mpsc::TryRecvError::Disconnected)
                )
            } else {
                false
            };

            if scan_finished {
                state.scan_receiver = None;
                state
                    .media_entries
                    .sort_by(|a, b| a.file_name.cmp(&b.file_name));
                let select_idx = state.pending_select_index.take().unwrap_or(0);
                state.thumbnail_tracker.cancel_debounce();
                let thumbnails = load_visible_thumbnails(state);
                let select = select_and_load_entry(state, select_idx);
                return Task::batch(vec![select, thumbnails, automation_task]);
            }

            if let Some(ref player) = state.audio_player
                && state.audio_playing
            {
                if player.empty() {
                    state.audio_playing = false;
                    state.audio_position = 0.0;
                } else {
                    state.audio_position = player.position();
                    state.audio_duration = player.duration();
                }
            }
            if refresh_thumbnails {
                Task::batch(vec![load_visible_thumbnails(state), automation_task])
            } else {
                automation_task
            }
        }
        Message::Video(VideoMessage::PlayerReady(sender)) => {
            state.video_sender = Some(sender);
            Task::none()
        }
        Message::Video(VideoMessage::Event(event)) => {
            match event {
                media_sort_backend::media::mpv_context::VideoEvent::FrameReady {
                    path,
                    width,
                    height,
                    rgba,
                } => {
                    let current_path = state.selected_index.and_then(|idx| {
                        state
                            .filtered_media_entries()
                            .get(idx)
                            .map(|e| e.path.clone())
                    });
                    if Some(path) == current_path && state.video_ready {
                        state.video_rgba = Some(rgba);
                        state.video_width = width;
                        state.video_height = height;
                        state.video_frame =
                            Some(iced::widget::image::Handle::from_rgba(1, 1, vec![0]));
                    }
                }
                media_sort_backend::media::mpv_context::VideoEvent::PlaybackProgress {
                    position,
                    duration,
                } => {
                    state.video_position = position;
                    state.video_duration = duration;
                    state.video_ready = true;
                }
                media_sort_backend::media::mpv_context::VideoEvent::Muted(muted) => {
                    state.video_muted = muted;
                }
                media_sort_backend::media::mpv_context::VideoEvent::Volume(vol) => {
                    state.video_volume = vol;
                }
                media_sort_backend::media::mpv_context::VideoEvent::Paused(paused) => {
                    state.video_paused = paused;
                }
            }
            Task::none()
        }
        Message::Video(VideoMessage::Seek(pos)) => {
            state.video_seek_position = Some(pos);
            let should_seek = state
                .video_last_seek_time
                .is_none_or(|t| t.elapsed() >= std::time::Duration::from_millis(333));
            if should_seek {
                if let Some(ref sender) = state.video_sender {
                    let _ = sender.try_send(
                        media_sort_backend::media::mpv_context::VideoCommand::SeekAbsolute(pos),
                    );
                }
                state.video_last_seek_time = Some(std::time::Instant::now());
            }
            Task::none()
        }
        Message::Video(VideoMessage::Volume(vol)) => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::SetVolume(vol));
            }
            Task::none()
        }
        Message::Video(VideoMessage::Mute) => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetMute(
                        !state.video_muted,
                    ),
                );
            }
            Task::none()
        }
        Message::Video(VideoMessage::PlayPause) => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::TogglePause);
            }
            Task::none()
        }
        Message::Video(VideoMessage::Stop) => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Stop);
            }
            Task::none()
        }
        Message::SettingsLoaded(result) => match *result {
            Ok(settings) => {
                state.settings = settings;
                #[cfg(target_os = "windows")]
                {
                    if state.settings.general.integration_with_windows {
                        if let Ok(exe) = std::env::current_exe()
                            && let Some(exe_str) = exe.to_str()
                        {
                            let _ = media_sort_backend::platform::windows_shell::register(exe_str);
                        }
                    }
                }
                Task::none()
            }
            Err(err) => {
                tracing::error!("Failed to load settings: {err}");
                Task::none()
            }
        },
        #[cfg(feature = "demo")]
        Message::AutomationBounds(_) | Message::AutomationVirtualTick(_) => Task::none(),
        Message::MediaScanCompleted(Ok(entries)) => {
            state.media_entries = entries;
            let select_idx = state.pending_select_index.take().unwrap_or(0);
            Task::batch(vec![
                select_and_load_entry(state, select_idx),
                load_visible_thumbnails(state),
            ])
        }
        Message::MediaScanCompleted(Err(err)) => {
            tracing::error!("Asynchronous media retrieval failed: {}", err);
            Task::none()
        }
        Message::Quit => {
            let _ = state.settings.save();
            state.should_exit = true;
            Task::none()
        }

        #[cfg(feature = "velopack")]
        Message::Update(update_msg) => handle_update_message(state, update_msg),

        Message::Folder(FolderMessage::Open(path)) => {
            state.open_folder(&path);
            Task::none()
        }
        Message::Folder(FolderMessage::Pick) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |result| Message::Folder(FolderMessage::PickResult(result)),
        ),
        Message::Folder(FolderMessage::PickResult(Some(path))) => {
            Task::done(Message::Folder(FolderMessage::Open(path)))
        }
        Message::Folder(FolderMessage::PickResult(None)) => Task::none(),
        Message::Folder(FolderMessage::PickPin) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |result| Message::Folder(FolderMessage::PickPinResult(result)),
        ),
        Message::Folder(FolderMessage::PickPinResult(Some(path))) => {
            state.pin_folder(&path);
            let _ = state.settings.save();
            Task::none()
        }
        Message::Folder(FolderMessage::PickPinResult(None)) => Task::none(),
        Message::Folder(FolderMessage::Selected(path, idx)) => {
            state.set_selected_folder(path, idx);
            Task::none()
        }
        Message::Folder(FolderMessage::ToggleExpand(path)) => {
            state.toggle_folder_expand(&path);
            Task::none()
        }

        Message::Media(MediaMessage::SelectEntry(index)) => select_and_load_entry(state, index),
        Message::Media(MediaMessage::SearchQueryChanged(query)) => {
            let previously_selected_path = state.selected_index.and_then(|idx| {
                state
                    .filtered_media_entries()
                    .get(idx)
                    .map(|entry| entry.path.clone())
            });

            state.search_query = query;
            state.search_focused = true;

            let filtered = state.filtered_media_entries();
            if filtered.is_empty() {
                state.selected_index = None;
                state.current_metadata = None;
                state.selected_image = None;
                Task::none()
            } else {
                let target_index = previously_selected_path
                    .and_then(|prev_path| filtered.iter().position(|entry| entry.path == prev_path))
                    .unwrap_or(0);
                select_and_load_entry(state, target_index)
            }
        }

        Message::Media(MediaMessage::MoveToFolder(target_folder)) => {
            if let Some(index) = state.selected_index {
                let filtered = state.filtered_media_entries();
                if let Some(entry) = filtered.get(index) {
                    let entry_path = entry.path.clone();
                    match MoveAction::new(&entry_path, &target_folder) {
                        Ok(mut action) => {
                            if let Err(e) = action.execute() {
                                tracing::error!("Move failed: {e}");
                            } else {
                                state.history.push_executed(Box::new(action));
                                state.media_entries.retain(|e| e.path != entry_path);
                                return select_and_load_entry(state, index);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Cannot create move action: {e}");
                        }
                    }
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::DeleteEntry(path)) => {
            let index_to_select = state.selected_index.unwrap_or(0);
            match media_sort_backend::filesystem::trash::delete_to_trash(&path) {
                Ok(handle) => {
                    let action =
                        media_sort_core::actions::delete_action::DeleteAction::new(&path, handle);
                    state.history.push_executed(Box::new(action));
                    state.media_entries.retain(|e| e.path != path);
                    return select_and_load_entry(state, index_to_select);
                }
                Err(e) => {
                    tracing::error!("Cannot delete to trash: {e}");
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::TriggerRename) => {
            if let Some(index) = state.selected_index {
                let filtered = state.filtered_media_entries();
                if let Some(entry) = filtered.get(index) {
                    let stem = entry
                        .path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    state.renaming_path = Some(entry.path.clone());
                    state.rename_input_value = stem;
                }
            }
            Task::none()
        }

        Message::Media(MediaMessage::CopyToFolder(target_folder)) => {
            let Some(index) = state.selected_index else {
                return Task::none();
            };
            let filtered = state.filtered_media_entries();
            let Some(entry) = filtered.get(index) else {
                return Task::none();
            };
            match media_sort_core::actions::copy_action::CopyAction::new(
                &entry.path,
                &target_folder,
            ) {
                Ok(mut action) => {
                    if let Err(e) = action.execute() {
                        tracing::error!("Copy failed: {e}");
                    } else {
                        state.history.push_executed(Box::new(action));
                    }
                }
                Err(e) => {
                    tracing::error!("Cannot create copy action: {e}");
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::RenameEntry(path, new_name)) => {
            match RenameAction::new(&path, &new_name) {
                Ok(mut action) => {
                    if let Err(e) = action.execute() {
                        tracing::error!("Rename failed: {e}");
                    } else {
                        let new_path = action.new_path().to_path_buf();
                        state.history.push_executed(Box::new(action));
                        if let Some(pos) = state.media_entries.iter().position(|e| e.path == path) {
                            state.media_entries[pos].path = new_path.clone();
                            state.media_entries[pos].file_name = new_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| new_path.display().to_string());
                            return select_and_load_entry(state, pos);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Cannot create rename action: {e}");
                }
            }
            Task::none()
        }

        Message::Media(MediaMessage::RenameInputChanged(val)) => {
            state.rename_input_value = val;
            Task::none()
        }
        Message::Media(MediaMessage::SubmitRename) => {
            if let Some(path) = state.renaming_path.take() {
                let new_name = state.rename_input_value.trim().to_string();
                if !new_name.is_empty() {
                    state.rename_input_value.clear();
                    return Task::done(Message::Media(MediaMessage::RenameEntry(path, new_name)));
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::CancelRename) => {
            state.renaming_path = None;
            state.rename_input_value.clear();
            Task::none()
        }
        Message::Folder(FolderMessage::CreateInputChanged(val)) => {
            state.create_folder_input = val;
            Task::none()
        }
        Message::Folder(FolderMessage::SubmitCreate(_parent)) => {
            if let Some(parent) = state.creating_folder_parent.take() {
                let folder_name = state.create_folder_input.trim().to_string();
                if !folder_name.is_empty() {
                    let new_dir = parent.join(&folder_name);
                    if let Err(e) = std::fs::create_dir_all(&new_dir) {
                        tracing::error!("Failed to create folder: {e}");
                    } else if state.current_folder.is_some() {
                        state.build_folder_tree();
                    }
                }
                state.create_folder_input.clear();
            }
            Task::none()
        }
        Message::Folder(FolderMessage::CancelCreate) => {
            state.creating_folder_parent = None;
            state.create_folder_input.clear();
            Task::none()
        }

        Message::Media(MediaMessage::Undo) => {
            let index = state.selected_index.unwrap_or(0);
            if let Err(e) = state.history.undo() {
                tracing::error!("Undo failed: {e}");
            } else {
                state.scan_media();
                return select_and_load_entry(state, index);
            }
            Task::none()
        }
        Message::Media(MediaMessage::Redo) => {
            let index = state.selected_index.unwrap_or(0);
            if let Err(e) = state.history.redo() {
                tracing::error!("Redo failed: {e}");
            } else {
                state.scan_media();
                return select_and_load_entry(state, index);
            }
            Task::none()
        }

        Message::Folder(FolderMessage::PinSelected) => {
            let path_to_pin = state
                .selected_folder
                .clone()
                .or(state.current_folder.clone());
            if let Some(path) = path_to_pin {
                state.pin_folder(&path);
                let _ = state.settings.save();
            }
            Task::none()
        }
        Message::Folder(FolderMessage::UnpinCurrent(path)) => {
            state.unpin_folder(&path);
            let _ = state.settings.save();
            Task::none()
        }
        Message::Folder(FolderMessage::MovePinnedUp(path)) => {
            state.move_pinned_folder_up(&path);
            let _ = state.settings.save();
            Task::none()
        }
        Message::Folder(FolderMessage::MovePinnedDown(path)) => {
            state.move_pinned_folder_down(&path);
            let _ = state.settings.save();
            Task::none()
        }
        Message::Folder(FolderMessage::TriggerCreate) => {
            if let Some(p) = state
                .selected_folder
                .as_ref()
                .or(state.current_folder.as_ref())
            {
                state.creating_folder_parent = Some(p.clone());
                state.create_folder_input = String::new();
            }
            Task::none()
        }

        Message::Settings(SettingsMessage::ToggleMetadataPanel) => {
            state.metadata_panel_expanded = !state.metadata_panel_expanded;
            state.settings.metadata_panel.is_expanded = state.metadata_panel_expanded;
            let _ = state.settings.save();
            Task::none()
        }

        Message::Media(MediaMessage::MetadataLoaded(result)) => match result {
            Ok(metadata) => {
                state.current_metadata = Some(metadata);
                Task::none()
            }
            Err(err) => {
                tracing::error!("Metadata load failed: {err}");
                state.current_metadata = None;
                Task::none()
            }
        },

        Message::Settings(SettingsMessage::EditKeyBinding(index)) => {
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

            if state.renaming_path.is_some() {
                if key == "Enter" {
                    return Task::done(Message::Media(MediaMessage::SubmitRename));
                } else if key == "Esc" {
                    return Task::done(Message::Media(MediaMessage::CancelRename));
                }
                return Task::none();
            }

            if state.creating_folder_parent.is_some() {
                if key == "Enter" {
                    return Task::done(Message::Folder(FolderMessage::SubmitCreate(
                        std::path::PathBuf::new(),
                    )));
                } else if key == "Esc" {
                    return Task::done(Message::Folder(FolderMessage::CancelCreate));
                }
                return Task::none();
            }

            if state.search_focused {
                if key == "Enter" || key == "Esc" || key == "Tab" {
                    return Task::done(Message::Media(MediaMessage::SearchBlurred));
                }
                return Task::none();
            }

            if key == "Space"
                && !state.search_focused
                && state.renaming_path.is_none()
                && state.creating_folder_parent.is_none()
            {
                if let Some(ref sender) = state.video_sender {
                    let _ = sender.try_send(
                        media_sort_backend::media::mpv_context::VideoCommand::TogglePause,
                    );
                }
                return Task::none();
            }

            if ctrl && key == "Q" {
                return Task::done(Message::Quit);
            }

            if (key == "MediaPlayPause" || key == "MediaPlay" || key == "MediaPause")
                && let Some(ref sender) = state.video_sender
            {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::TogglePause);
                return Task::none();
            }
            if key == "MediaStop"
                && let Some(ref sender) = state.video_sender
            {
                let _ = sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Stop);
                return Task::none();
            }
            if key == "AudioVolumeUp"
                && let Some(ref sender) = state.video_sender
            {
                let new_vol = (state.video_volume + 5.0).min(100.0);
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetVolume(new_vol),
                );
                return Task::none();
            }
            if key == "AudioVolumeDown"
                && let Some(ref sender) = state.video_sender
            {
                let new_vol = (state.video_volume - 5.0).max(0.0);
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetVolume(new_vol),
                );
                return Task::none();
            }
            if key == "AudioVolumeMute"
                && let Some(ref sender) = state.video_sender
            {
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetMute(
                        !state.video_muted,
                    ),
                );
                return Task::none();
            }
            if key == "MediaTrackNext" {
                return Task::done(Message::Media(MediaMessage::GoRight));
            }
            if key == "MediaTrackPrevious" {
                return Task::done(Message::Media(MediaMessage::GoLeft));
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
                            return Task::done(Message::Media(MediaMessage::Undo));
                        }
                        "redo" if state.history.can_redo() => {
                            return Task::done(Message::Media(MediaMessage::Redo));
                        }
                        "open_folder" => {
                            if let Ok(p) = std::env::current_dir() {
                                return Task::done(Message::Folder(FolderMessage::Open(p)));
                            }
                        }
                        "toggle_metadata_panel" => {
                            return Task::done(Message::Settings(
                                SettingsMessage::ToggleMetadataPanel,
                            ));
                        }
                        "pin" => {
                            return Task::done(Message::Folder(FolderMessage::PickPin));
                        }
                        "unpin" => {
                            if let Some(ref c) = state.current_folder {
                                return Task::done(Message::Folder(FolderMessage::UnpinCurrent(
                                    c.clone(),
                                )));
                            }
                        }
                        "go_left" => {
                            return Task::done(Message::Media(MediaMessage::GoLeft));
                        }
                        "go_right" => {
                            return Task::done(Message::Media(MediaMessage::GoRight));
                        }
                        "move_to_folder" => {
                            if let Some(index) = state.selected_index {
                                let filtered = state.filtered_media_entries();
                                if let Some(_entry) = filtered.get(index)
                                    && let Some(ref dest) = state.selected_folder
                                {
                                    return Task::done(Message::Media(MediaMessage::MoveToFolder(
                                        dest.clone(),
                                    )));
                                }
                            }
                        }
                        "copy_to_folder" => {
                            if let Some(index) = state.selected_index {
                                let filtered = state.filtered_media_entries();
                                if let Some(_entry) = filtered.get(index)
                                    && let Some(ref dest) = state.selected_folder
                                {
                                    return Task::done(Message::Media(MediaMessage::CopyToFolder(
                                        dest.clone(),
                                    )));
                                }
                            }
                        }
                        "delete" => {
                            if let Some(index) = state.selected_index {
                                let filtered = state.filtered_media_entries();
                                if let Some(entry) = filtered.get(index) {
                                    return Task::done(Message::Media(MediaMessage::DeleteEntry(
                                        entry.path.clone(),
                                    )));
                                }
                            }
                        }
                        "rename" => {
                            if let Some(index) = state.selected_index {
                                let filtered = state.filtered_media_entries();
                                if let Some(entry) = filtered.get(index) {
                                    let stem = entry
                                        .path
                                        .file_stem()
                                        .map(|s| s.to_string_lossy().to_string())
                                        .unwrap_or_default();
                                    state.renaming_path = Some(entry.path.clone());
                                    state.rename_input_value = stem;
                                }
                            }
                        }
                        "create_folder" => {
                            if let Some(p) = state
                                .selected_folder
                                .as_ref()
                                .or(state.current_folder.as_ref())
                            {
                                state.creating_folder_parent = Some(p.clone());
                                state.create_folder_input = String::new();
                            }
                        }
                        "open_selected_folder" => {
                            if let Some(ref selected_path) = state.selected_folder {
                                return Task::done(Message::Folder(FolderMessage::Open(
                                    selected_path.clone(),
                                )));
                            }
                        }
                        "pin_selected" => {
                            return Task::done(Message::Folder(FolderMessage::PinSelected));
                        }
                        "move_pinned_up" => {
                            if let Some(selected_path) = state.selected_folder.clone() {
                                return Task::done(Message::Folder(FolderMessage::MovePinnedUp(
                                    selected_path,
                                )));
                            }
                        }
                        "move_pinned_down" => {
                            if let Some(selected_path) = state.selected_folder.clone() {
                                return Task::done(Message::Folder(FolderMessage::MovePinnedDown(
                                    selected_path,
                                )));
                            }
                        }
                        "folder_up" => {
                            state.select_folder_above();
                            return scroll_to_selected_folder(state);
                        }
                        "folder_down" => {
                            state.select_folder_below();
                            return scroll_to_selected_folder(state);
                        }
                        "folder_left" => {
                            state.collapse_selected_folder();
                            return scroll_to_selected_folder(state);
                        }
                        "folder_right" => {
                            state.expand_selected_folder();
                            return scroll_to_selected_folder(state);
                        }
                        "search_images" => {
                            return Task::done(Message::Media(MediaMessage::SearchFocused));
                        }
                        _ => {}
                    }
                }
            }

            if alt
                && !ctrl
                && !shift
                && let Some(c) = key.chars().next()
                && c.is_ascii_digit()
                && c != '0'
            {
                let digit = c.to_digit(10).unwrap() as u8;
                return Task::done(Message::Folder(FolderMessage::PinShortcut(digit)));
            }

            Task::none()
        }

        Message::Settings(SettingsMessage::Open) => {
            state.show_settings = true;
            state.show_keybindings = false;
            Task::none()
        }
        Message::Settings(SettingsMessage::Close) => {
            state.show_settings = false;
            state.show_keybindings = false;
            state.editing_keybinding = None;
            state.waiting_for_key = false;
            Task::done(Message::Settings(SettingsMessage::Save))
        }
        Message::Settings(SettingsMessage::ChangeLanguage(locale)) => {
            state.l10n.set_locale(&locale);
            state.settings.general.locale = Some(locale);
            let _ = state.settings.save();
            state.search_placeholder = state.l10n.tr("keybindings-search-images");
            state.rename_placeholder = state.l10n.tr("ui-enter-new-name");
            state.create_folder_placeholder = state.l10n.tr("ui-folder-name-placeholder");
            Task::none()
        }
        Message::Video(VideoMessage::PlayExternally(path)) => {
            open_externally(&path);
            Task::none()
        }
        Message::Settings(SettingsMessage::SetTheme(theme)) => {
            state.settings.general.theme = theme;
            let _ = state.settings.save();
            Task::none()
        }
        Message::Settings(SettingsMessage::ToggleReopenFolder) => {
            state.settings.general.reopen_last_opened_folder =
                !state.settings.general.reopen_last_opened_folder;
            let _ = state.settings.save();
            Task::none()
        }
        Message::Settings(SettingsMessage::StartDragFolderDivider) => {
            state.dragging_folder_divider = true;
            Task::none()
        }
        Message::Settings(SettingsMessage::StartDragMetadataDivider) => {
            state.dragging_metadata_divider = true;
            Task::none()
        }
        Message::EventOccurred(event) => match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                let _ = state.settings.save();
                window::latest().and_then(window::close)
            }
            iced::Event::Window(iced::window::Event::Resized(size)) => {
                state.settings.window_position.width = size.width.round() as u32;
                state.settings.window_position.height = size.height.round() as u32;
                #[cfg(feature = "demo")]
                if let Some(ref mut automation) = state.automation {
                    automation.update_window_size(size.width, size.height);
                }

                state.thumbnail_tracker.handle_scroll();
                Task::none()
            }
            iced::Event::Window(iced::window::Event::Moved(point)) => {
                state.settings.window_position.left = point.x.round() as i32;
                state.settings.window_position.top = point.y.round() as i32;
                Task::none()
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                if state.dragging_folder_divider {
                    let new_width = position.x.round().clamp(100.0, 800.0) as u16;
                    state.settings.general.folder_tree_width = new_width;
                }
                if state.dragging_metadata_divider {
                    let window_width = state.settings.window_position.width as f32;
                    let raw_width = window_width - position.x;
                    let new_width = raw_width.round().clamp(100.0, 800.0) as u16;
                    state.settings.metadata_panel.panel_width = new_width;
                }
                Task::none()
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                if state.dragging_folder_divider || state.dragging_metadata_divider {
                    state.dragging_folder_divider = false;
                    state.dragging_metadata_divider = false;
                    let _ = state.settings.save();
                }
                Task::none()
            }
            _ => Task::none(),
        },
        #[cfg(feature = "velopack")]
        Message::Settings(SettingsMessage::ToggleCheckForUpdates) => {
            state.settings.general.check_for_updates_on_startup =
                !state.settings.general.check_for_updates_on_startup;
            let _ = state.settings.save();
            Task::none()
        }
        #[cfg(feature = "velopack")]
        Message::Settings(SettingsMessage::ToggleInstallPrerelease) => {
            state.settings.general.install_prerelease_builds =
                !state.settings.general.install_prerelease_builds;
            let _ = state.settings.save();
            Task::none()
        }
        #[cfg(target_os = "windows")]
        Message::Settings(SettingsMessage::ToggleIntegrationWithWindows) => {
            state.settings.general.integration_with_windows =
                !state.settings.general.integration_with_windows;
            let enabled = state.settings.general.integration_with_windows;
            let _ = state.settings.save();

            if enabled {
                if let Ok(exe) = std::env::current_exe()
                    && let Some(exe_str) = exe.to_str()
                {
                    let _ = media_sort_backend::platform::windows_shell::register(exe_str);
                }
            } else {
                let _ = media_sort_backend::platform::windows_shell::unregister();
            }
            Task::none()
        }
        Message::Settings(SettingsMessage::ToggleAnimateGifs) => {
            state.settings.general.animate_gifs = !state.settings.general.animate_gifs;
            let _ = state.settings.save();
            Task::none()
        }
        Message::Settings(SettingsMessage::Save) => {
            let _ = state.settings.save();
            Task::none()
        }
        Message::Settings(SettingsMessage::OpenKeybindings) => {
            state.show_settings = true;
            state.show_keybindings = true;
            Task::none()
        }
        Message::Settings(SettingsMessage::RestoreDefaultKeyBindings) => {
            state.settings.keybindings =
                media_sort_core::settings::keybindings::KeyBindings::default();
            let _ = state.settings.save();
            Task::none()
        }
        Message::OpenCredits => {
            state.show_credits = true;
            Task::none()
        }
        Message::CloseCredits => {
            state.show_credits = false;
            Task::none()
        }

        Message::Media(MediaMessage::AudioPlayPause) => {
            if let Some(ref player) = state.audio_player {
                if player.is_paused() {
                    player.resume();
                    state.audio_playing = true;
                } else if state.audio_playing {
                    player.pause();
                    state.audio_playing = false;
                } else if let Some(index) = state.selected_index {
                    let entries = state.filtered_media_entries();
                    if let Some(entry) = entries.get(index) {
                        if let Err(e) = player.play(&entry.path) {
                            tracing::error!("Audio play failed: {e}");
                        } else {
                            state.audio_playing = true;
                            state.audio_duration = player.duration();
                        }
                    }
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::StopAudio) => {
            if let Some(ref player) = state.audio_player {
                player.stop();
            }
            state.audio_playing = false;
            state.audio_position = 0.0;
            Task::none()
        }
        Message::Media(MediaMessage::AudioSeek(pos)) => {
            if let Some(ref player) = state.audio_player
                && let Err(e) = player.seek(pos)
            {
                tracing::error!("Audio seek failed: {e}");
            }
            Task::none()
        }
        Message::Media(MediaMessage::AudioSetVolume(vol)) => {
            if let Some(ref player) = state.audio_player {
                player.set_volume(vol as f32 / 100.0);
            }
            state.audio_volume = vol;
            Task::none()
        }
        Message::Media(MediaMessage::AudioToggleMute) => {
            state.audio_muted = !state.audio_muted;
            if let Some(ref player) = state.audio_player {
                if state.audio_muted {
                    player.set_volume(0.0);
                } else {
                    player.set_volume(state.audio_volume as f32 / 100.0);
                }
            }
            Task::none()
        }

        Message::Media(MediaMessage::ThumbnailReady(path, w, h, data)) => {
            if !data.is_empty() && w > 0 && h > 0 {
                let handle = iced::widget::image::Handle::from_rgba(w, h, data);
                state.thumbnail_cache.push(path, handle);
            }
            Task::none()
        }
        Message::Media(MediaMessage::ThumbnailFailed(path)) => {
            state.unsupported_files.insert(path);
            Task::none()
        }
        Message::Media(MediaMessage::ThumbnailCancelled(_path)) => Task::none(),
        Message::Media(MediaMessage::OpenExternal(path)) => {
            open_externally(&path);
            Task::none()
        }
        Message::Media(MediaMessage::ImageLoaded(path, result)) => {
            match result {
                Ok((w, h, pixels)) => {
                    let handle = iced::widget::image::Handle::from_rgba(w, h, pixels);
                    state.image_cache.push(path.clone(), handle.clone());
                    if let Some(idx) = state.selected_index {
                        let entries = state.filtered_media_entries();
                        if let Some(entry) = entries.get(idx)
                            && entry.path == path
                        {
                            state.selected_image = Some((path, handle));
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("Failed to load full image: {err}");
                    if let Some(idx) = state.selected_index {
                        let entries = state.filtered_media_entries();
                        if let Some(entry) = entries.get(idx)
                            && entry.path == path
                        {
                            state.selected_image = None;
                        }
                    }
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::GoLeft) => {
            if let Some(idx) = state.selected_index
                && idx > 0
            {
                return select_and_load_entry(state, idx - 1);
            }
            Task::none()
        }
        Message::Media(MediaMessage::GoRight) => {
            if let Some(idx) = state.selected_index {
                let filtered_len = state.filtered_media_entries().len();
                if idx + 1 < filtered_len {
                    return select_and_load_entry(state, idx + 1);
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::MoveActive) => {
            if let Some(index) = state.selected_index
                && let Some(ref target_folder) = state.selected_folder
            {
                let filtered = state.filtered_media_entries();
                if let Some(entry) = filtered.get(index) {
                    let entry_path = entry.path.clone();
                    match MoveAction::new(&entry_path, target_folder) {
                        Ok(mut action) => {
                            if let Err(e) = action.execute() {
                                tracing::error!("Move failed: {e}");
                            } else {
                                state.history.push_executed(Box::new(action));
                                state.media_entries.retain(|e| e.path != entry_path);
                                return select_and_load_entry(state, index);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Cannot create move action: {e}");
                        }
                    }
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::CopyActive) => {
            if let Some(index) = state.selected_index
                && let Some(ref target_folder) = state.selected_folder
            {
                let filtered = state.filtered_media_entries();
                if let Some(entry) = filtered.get(index) {
                    match media_sort_core::actions::copy_action::CopyAction::new(
                        &entry.path,
                        target_folder,
                    ) {
                        Ok(mut action) => {
                            if let Err(e) = action.execute() {
                                tracing::error!("Copy failed: {e}");
                            } else {
                                state.history.push_executed(Box::new(action));
                            }
                        }
                        Err(e) => {
                            tracing::error!("Cannot create copy action: {e}");
                        }
                    }
                }
            }
            Task::none()
        }
        Message::Folder(FolderMessage::PinShortcut(n)) => {
            let pinned_idx = (n.saturating_sub(1)) as usize;
            if let Some(pinned) = state.pinned_folders.get(pinned_idx) {
                let target_folder = pinned.path.clone();
                if let Some(index) = state.selected_index {
                    let filtered = state.filtered_media_entries();
                    if let Some(entry) = filtered.get(index) {
                        let entry_path = entry.path.clone();
                        match MoveAction::new(&entry_path, &target_folder) {
                            Ok(mut action) => {
                                if let Err(e) = action.execute() {
                                    tracing::error!("Move failed: {e}");
                                } else {
                                    state.history.push_executed(Box::new(action));
                                    state.media_entries.retain(|e| e.path != entry_path);
                                    return select_and_load_entry(state, index);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Cannot create move action: {e}");
                            }
                        }
                    }
                }
            }
            Task::none()
        }
        Message::Media(MediaMessage::SearchFocused) => {
            state.search_focused = true;
            iced::widget::operation::focus(crate::view::search_bar::SEARCH_INPUT_ID.clone())
        }
        Message::Media(MediaMessage::SearchBlurred) => {
            state.search_focused = false;
            iced::advanced::widget::operate(iced::advanced::widget::operation::focusable::unfocus())
        }
        Message::Media(MediaMessage::GridScrolled(offset, viewport_width, content_width)) => {
            state.media_grid_scroll.offset_x = offset.x;
            state.media_grid_scroll.viewport_width = viewport_width;
            state.media_grid_scroll.content_width = content_width;

            state.thumbnail_tracker.handle_scroll();
            Task::none()
        }
    }
}

fn select_and_load_entry(state: &mut AppState, index: usize) -> Task<Message> {
    let filtered = state.filtered_media_entries();
    let filtered_len = filtered.len();
    if filtered_len > 0 {
        let index = index.min(filtered_len - 1);
        let entry = filtered[index];
        let path = entry.path.clone();
        let media_type = entry.media_type;

        let start = index.saturating_sub(5);
        let end = (index + 6).min(filtered_len);
        let mut thumbnail_paths = Vec::new();
        for entry in filtered.iter().take(end).skip(start) {
            thumbnail_paths.push(entry.path.clone());
        }

        // Pre-load the next and previous full images!
        let mut preload_tasks = Vec::new();
        if index + 1 < filtered_len {
            let next_entry = filtered[index + 1];
            if next_entry.media_type == media_sort_core::media_type::MediaType::Image
                && !state.image_cache.contains(&next_entry.path)
            {
                preload_tasks.push(load_full_image(
                    next_entry.path.clone(),
                    next_entry.media_type,
                ));
            }
        }
        if index > 0 {
            let prev_entry = filtered[index - 1];
            if prev_entry.media_type == media_sort_core::media_type::MediaType::Image
                && !state.image_cache.contains(&prev_entry.path)
            {
                preload_tasks.push(load_full_image(
                    prev_entry.path.clone(),
                    prev_entry.media_type,
                ));
            }
        }

        drop(filtered);

        state
            .thumbnail_tracker
            .retain_paths(thumbnail_paths.clone());

        state.selected_index = Some(index);
        state.current_metadata = None;

        if media_type == media_sort_core::media_type::MediaType::Video {
            if let Some(ref sender) = state.video_sender {
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::Load(path.clone()),
                );
            }
            state.video_frame = None;
            state.video_rgba = None;
            state.video_width = 0;
            state.video_height = 0;
            state.video_position = 0.0;
            state.video_duration = 0.0;
            state.video_ready = false;
            state.video_seek_position = None;
            state.video_last_seek_time = None;
            if let Some(ref mut ap) = state.audio_player {
                ap.stop();
            }
            state.audio_playing = false;
            state.audio_position = 0.0;
        } else if media_type == media_sort_core::media_type::MediaType::Audio {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
            }
            state.video_frame = None;
            state.video_rgba = None;
            state.video_width = 0;
            state.video_height = 0;
            state.video_ready = false;
            if state.audio_playing
                && let Some(ref player) = state.audio_player
            {
                player.stop();
                if let Err(e) = player.play(&path) {
                    tracing::error!("Audio play failed: {e}");
                    state.audio_playing = false;
                } else {
                    state.audio_duration = player.duration();
                }
            }
        } else {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
            }
            state.video_frame = None;
            state.video_rgba = None;
            state.video_width = 0;
            state.video_height = 0;
            state.video_ready = false;
            if state.audio_playing {
                if let Some(ref player) = state.audio_player {
                    player.stop();
                }
                state.audio_playing = false;
                state.audio_position = 0.0;
            }
        }

        let mut tasks = vec![load_metadata(state, index)];

        tasks.push(scroll_to_selected_entry(state, index));

        state.selected_audio_cover = None;
        if media_type == media_sort_core::media_type::MediaType::Audio
            && let Some(bytes) = media_sort_backend::media::thumbnail::extract_audio_cover(&path)
            && let Ok(img) = image::load_from_memory(&bytes)
        {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            state.selected_audio_cover = Some(iced::widget::image::Handle::from_rgba(
                w,
                h,
                rgba.into_raw(),
            ));
        }

        if let Some(handle) = state.image_cache.get(&path) {
            state.selected_image = Some((path, handle.clone()));
        } else {
            state.selected_image = None;
            tasks.push(load_full_image(path, media_type));
        }

        for t in preload_tasks {
            tasks.push(t);
        }

        for p in thumbnail_paths {
            if !state.thumbnail_cache.contains(&p) {
                tasks.push(load_thumbnail(p, state.thumbnail_tracker.clone_checker()));
            }
        }
        Task::batch(tasks)
    } else {
        state.selected_index = None;
        state.current_metadata = None;
        state.selected_image = None;
        if let Some(ref sender) = state.video_sender {
            let _ =
                sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
        }
        state.video_frame = None;
        state.video_rgba = None;
        state.video_width = 0;
        state.video_height = 0;
        state.video_ready = false;
        Task::none()
    }
}

#[cfg(feature = "velopack")]
fn handle_update_message(
    state: &mut AppState,
    msg: crate::message::UpdateMessage,
) -> Task<Message> {
    use crate::message::UpdateMessage;
    match msg {
        UpdateMessage::CheckForUpdates => {
            let settings = state.settings.general.clone();
            Task::perform(
                async move { crate::check_for_update_async(&settings).await },
                |result| match result {
                    Ok(Some(info)) => {
                        Message::Update(UpdateMessage::UpdateAvailable(Box::new(info)))
                    }
                    Ok(None) => Message::Update(UpdateMessage::NoUpdateFound),
                    Err(e) => Message::Update(UpdateMessage::UpdateFailed(e)),
                },
            )
        }
        UpdateMessage::UpdateAvailable(info) => {
            state.show_update_prompt = true;
            state.pending_update = Some(*info);
            Task::none()
        }
        UpdateMessage::NoUpdateFound => {
            tracing::info!("Update check completed, no update available");
            Task::none()
        }
        UpdateMessage::UserConfirmedUpdate(info) => {
            state.show_update_prompt = false;
            state.pending_update = None;
            Task::perform(
                crate::download_and_apply_async(*info),
                |result| match result {
                    Ok(()) => Message::Update(UpdateMessage::UpdateFailed(
                        "Update applied, restarting...".into(),
                    )),
                    Err(e) => Message::Update(UpdateMessage::UpdateFailed(e)),
                },
            )
        }
        UpdateMessage::UpdateFailed(e) => {
            tracing::error!("Update failed: {e}");
            Task::none()
        }
        UpdateMessage::DismissUpdatePrompt => {
            state.show_update_prompt = false;
            state.pending_update = None;
            Task::none()
        }
    }
}

/// Build a [`Task`] that scrolls the media grid so that the entry at
/// `index` is clearly visible. We use a *relative* scroll position so we
/// don't have to depend on `state.media_grid_scroll.viewport_width` being
/// up to date — that snapshot can briefly lag behind the actual layout
/// (e.g. when the user has just resized the window, or the very first
/// `on_scroll` after opening a folder hasn't fired yet). If we used an
/// absolute pixel offset computed from a stale viewport, we could end up
/// "scrolling" to a position that doesn't actually bring the selected
/// card into view. The scrollable resolves the relative position using
/// its own, always-current, content and viewport widths.
///
/// The relative position is `index / (n - 1)`, so the selected card ends
/// up at the corresponding proportional position in the content, well
/// inside the viewport for any sane window size.
fn scroll_to_selected_entry(state: &AppState, index: usize) -> Task<Message> {
    use crate::view::media_grid::MEDIA_GRID_SCROLLABLE_ID;

    let n = state.filtered_media_entries().len();
    let Some(relative_x) = relative_position_for(index, n) else {
        return Task::none();
    };

    iced::widget::operation::snap_to(
        MEDIA_GRID_SCROLLABLE_ID.clone(),
        iced::widget::scrollable::RelativeOffset {
            x: Some(relative_x),
            y: None,
        },
    )
}

/// Compute the relative horizontal scroll position (in `[0.0, 1.0]`) that
/// corresponds to `index` in a list of `total` entries. Returns `None` when
/// the list has zero or one entries, in which case there is nothing to
/// scroll to.
fn relative_position_for(index: usize, total: usize) -> Option<f32> {
    if total <= 1 {
        return None;
    }
    let clamped_index = index.min(total - 1);
    Some(clamped_index as f32 / (total - 1) as f32)
}

fn scroll_to_selected_folder(state: &AppState) -> Task<Message> {
    use crate::view::folder_panel::FOLDER_TREE_SCROLLABLE_ID;

    let visible = state.collect_visible_folders();
    let Some(idx) = state.selected_folder_idx.filter(|i| *i < visible.len()) else {
        return Task::none();
    };
    let Some(relative_y) = relative_position_for(idx, visible.len()) else {
        return Task::none();
    };

    iced::widget::operation::snap_to(
        FOLDER_TREE_SCROLLABLE_ID.clone(),
        iced::widget::scrollable::RelativeOffset {
            x: None,
            y: Some(relative_y),
        },
    )
}

fn load_visible_thumbnails(state: &mut AppState) -> Task<Message> {
    let entry_paths: Vec<std::path::PathBuf> = state
        .filtered_media_entries()
        .iter()
        .map(|e| e.path.clone())
        .collect();
    let load_queue = state.thumbnail_tracker.update_viewport(
        &state.media_grid_scroll,
        &entry_paths,
        state.settings.window_position.width,
    );

    Task::batch(
        load_queue
            .into_iter()
            .filter(|path| {
                !state.thumbnail_cache.contains(path) && !state.unsupported_files.contains(path)
            })
            .map(|path| load_thumbnail(path, state.thumbnail_tracker.clone_checker())),
    )
}

fn load_thumbnail(
    path: std::path::PathBuf,
    visible_tracker: std::sync::Arc<
        std::sync::RwLock<std::collections::HashSet<std::path::PathBuf>>,
    >,
) -> Task<Message> {
    Task::perform(
        async move {
            let path_clone = path.clone();
            let tracker = visible_tracker.clone();
            let result = tokio::task::spawn_blocking(move || {
                if let Ok(guard) = tracker.read()
                    && !guard.contains(&path_clone)
                {
                    return Ok(None);
                }
                match crate::subscriptions::prefetch::generate_thumbnail(&path_clone) {
                    Ok((w, h, rgba)) => Ok(Some((w, h, rgba))),
                    Err(()) => Err(()),
                }
            })
            .await
            .unwrap_or(Ok(None));
            (path, result)
        },
        |(path, result)| {
            Message::Media(match result {
                Ok(Some((w, h, rgba))) => MediaMessage::ThumbnailReady(path, w, h, rgba),
                Ok(None) => MediaMessage::ThumbnailCancelled(path),
                Err(()) => MediaMessage::ThumbnailFailed(path),
            })
        },
    )
}

fn open_externally(path: &std::path::Path) {
    let res = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(path).spawn()
    };
    if let Err(e) = res {
        tracing::error!("Failed to open file externally: {e}");
    }
}

fn load_full_image(path: std::path::PathBuf, media_type: MediaType) -> Task<Message> {
    if media_type != MediaType::Image {
        return Task::none();
    }
    Task::perform(
        async move {
            let path_clone = path.clone();
            let res = tokio::task::spawn_blocking(move || {
                media_sort_backend::media::image_decoder::load_image(&path_clone)
                    .map(|img| {
                        use image::GenericImageView;
                        let (w, h) = img.dimensions();
                        let rgba = img.to_rgba8().into_raw();
                        (w, h, rgba)
                    })
                    .map_err(|e| e.to_string())
            })
            .await
            .unwrap_or_else(|e| Err(format!("Join error: {e}")));
            (path, res)
        },
        |(path, res)| Message::Media(MediaMessage::ImageLoaded(path, res)),
    )
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
        |result| Message::Media(MediaMessage::MetadataLoaded(result)),
    )
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let base_view = view::main_layout::main_layout_view(state);
    #[cfg(feature = "demo")]
    if let Some(ref automation) = state.automation {
        return iced_automation::wrap_view(base_view, automation);
    }
    base_view
}

pub fn theme(state: &AppState) -> iced::Theme {
    match state.settings.general.theme.as_str() {
        "Dark" => iced::Theme::Dark,
        "Dracula" => iced::Theme::Dracula,
        "Nord" => iced::Theme::Nord,
        "SolarizedLight" => iced::Theme::SolarizedLight,
        "SolarizedDark" => iced::Theme::SolarizedDark,
        "GruvboxLight" => iced::Theme::GruvboxLight,
        "GruvboxDark" => iced::Theme::GruvboxDark,
        "CatppuccinLatte" => iced::Theme::CatppuccinLatte,
        "CatppuccinFrappe" => iced::Theme::CatppuccinFrappe,
        "CatppuccinMacchiato" => iced::Theme::CatppuccinMacchiato,
        "CatppuccinMocha" => iced::Theme::CatppuccinMocha,
        "TokyoNight" => iced::Theme::TokyoNight,
        "TokyoNightStorm" => iced::Theme::TokyoNightStorm,
        "TokyoNightLight" => iced::Theme::TokyoNightLight,
        "KanagawaWave" => iced::Theme::KanagawaWave,
        "KanagawaDragon" => iced::Theme::KanagawaDragon,
        "KanagawaLotus" => iced::Theme::KanagawaLotus,
        "Moonfly" => iced::Theme::Moonfly,
        "Nightfly" => iced::Theme::Nightfly,
        "Oxocarbon" => iced::Theme::Oxocarbon,
        "Ferra" => iced::Theme::Ferra,
        _ => iced::Theme::Light,
    }
}

pub fn subscription(_state: &AppState) -> Subscription<Message> {
    let tick_sub = iced::time::every(std::time::Duration::from_millis(16)).map(Message::Tick);

    let keyboard_sub = crate::subscriptions::keyboard::keyboard_subscription();

    let event_sub = iced::event::listen().map(Message::EventOccurred);

    let video_sub = crate::subscriptions::video_player::video_player_subscription();

    Subscription::batch(vec![tick_sub, keyboard_sub, event_sub, video_sub])
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
        let _task = update(&mut state, Message::Media(MediaMessage::SelectEntry(0)));
        assert_eq!(state.selected_index, Some(0));
        assert!(state.current_metadata.is_none());
    }

    #[test]
    fn test_select_entry_out_of_bounds() {
        let mut state = AppState::new(SettingsStore::default());
        state.media_entries = vec![];
        state.search_query = String::new();
        state.selected_index = None;
        let _task = update(&mut state, Message::Media(MediaMessage::SelectEntry(99)));
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
        let _task = update(&mut state, Message::Media(MediaMessage::SelectEntry(0)));
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
        let _ = update(&mut state, Message::Media(MediaMessage::Undo));
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
        let _ = update(&mut state, Message::Media(MediaMessage::Redo));
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
        let _ = update(
            &mut state,
            Message::Settings(SettingsMessage::ToggleMetadataPanel),
        );
        assert!(state.metadata_panel_expanded);

        let _ = update(
            &mut state,
            Message::KeyCaptured("M".into(), false, false, false),
        );
        let _ = update(
            &mut state,
            Message::Settings(SettingsMessage::ToggleMetadataPanel),
        );
        assert!(!state.metadata_panel_expanded);
    }

    #[test]
    fn test_keycaptured_pin_dispatches_pick() {
        let mut state = AppState::new(SettingsStore::default());

        // The pin shortcut now dispatches PickPin; verify it doesn't panic.
        let _task = update(
            &mut state,
            Message::KeyCaptured("P".into(), false, false, false),
        );
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
        let _ = update(
            &mut state,
            Message::Folder(FolderMessage::UnpinCurrent(folder.clone())),
        );
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
            .unwrap_or_else(std::env::temp_dir)
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
        state.scan_media();
        state.selected_index = Some(0);

        assert!(file.exists());
        let dest_file = dest.join("test_image.jpg");
        assert!(!dest_file.exists());

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(dest.clone())),
        );

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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(dest.clone())),
        );

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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(dest.clone())),
        );

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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(nonexistent)),
        );

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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::DeleteEntry(file.clone())),
        );

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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::DeleteEntry(nonexistent)),
        );

        assert!(!state.history.can_undo());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_undo_after_move() {
        let (root, file, dest) = setup_temp_dir_with_files("undo_move");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);
        state.scan_media();
        state.selected_index = Some(0);

        let _ = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(dest.clone())),
        );
        assert!(!file.exists());
        let dest_file = dest.join("test_image.jpg");
        assert!(dest_file.exists());
        assert!(state.history.can_undo());

        let _task = update(&mut state, Message::Media(MediaMessage::Undo));

        assert!(file.exists());
        assert!(!dest_file.exists());
        assert!(!state.history.can_undo());
        assert!(state.history.can_redo());
        assert_eq!(state.selected_index, Some(0));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_undo_after_delete() {
        let (root, file, _dest) = setup_data_dir_with_files("undo_delete");

        let mut state = AppState::new(SettingsStore::default());
        state.open_folder(&root);

        let _ = update(
            &mut state,
            Message::Media(MediaMessage::DeleteEntry(file.clone())),
        );
        assert!(!file.exists());
        assert!(state.history.can_undo());

        let _task = update(&mut state, Message::Media(MediaMessage::Undo));

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
        state.scan_media();
        state.selected_index = Some(0);

        let _ = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(dest.clone())),
        );
        let _ = update(&mut state, Message::Media(MediaMessage::Undo));
        assert!(file.exists());
        assert!(state.history.can_redo());

        let _task = update(&mut state, Message::Media(MediaMessage::Redo));

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

        let _task = update(&mut state, Message::Media(MediaMessage::Undo));
        assert!(!state.history.can_undo());
    }

    #[test]
    fn test_redo_empty_undone_no_panic() {
        let mut state = AppState::new(SettingsStore::default());
        assert!(!state.history.can_redo());

        let _task = update(&mut state, Message::Media(MediaMessage::Redo));
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
            Message::Media(MediaMessage::RenameEntry(
                file.clone(),
                "renamed_image".to_string(),
            )),
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
            Message::Media(MediaMessage::RenameEntry(file1.clone(), "b".to_string())),
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
        state.scan_media();
        state.selected_index = Some(0);

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::MoveToFolder(dest.clone())),
        );

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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::DeleteEntry(file.clone())),
        );
        assert!(!file.exists());
        assert!(state.history.can_undo());

        let _task = update(&mut state, Message::Media(MediaMessage::Undo));
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
            Message::Media(MediaMessage::ThumbnailReady(
                std::path::PathBuf::from("/test/empty.jpg"),
                0,
                0,
                Vec::new(),
            )),
        );
        assert_eq!(state.thumbnail_cache.len(), cache_size_before);
    }

    #[test]
    fn test_thumbnail_ready_valid_data() {
        let mut state = AppState::new(SettingsStore::default());
        let path = std::path::PathBuf::from("/test/thumb.jpg");

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::ThumbnailReady(
                path.clone(),
                1,
                1,
                vec![255, 0, 0, 255],
            )),
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
            Message::Media(MediaMessage::MetadataLoaded(Err("load failed".to_string()))),
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

        let _task = update(
            &mut state,
            Message::Media(MediaMessage::MetadataLoaded(Ok(metadata))),
        );
        assert!(state.current_metadata.is_some());
        let m = state.current_metadata.as_ref().unwrap();
        assert_eq!(m.get("EXIF").unwrap().get("Width").unwrap(), "1920");
    }

    #[test]
    fn test_grid_scrolled_updates_viewport_state() {
        let mut state = AppState::new(SettingsStore::default());
        assert_eq!(state.media_grid_scroll.viewport_width, 0.0);

        let _ = update(
            &mut state,
            Message::Media(MediaMessage::GridScrolled(
                iced::widget::scrollable::AbsoluteOffset { x: 120.0, y: 0.0 },
                400.0,
                1200.0,
            )),
        );
        assert_eq!(state.media_grid_scroll.offset_x, 120.0);
        assert_eq!(state.media_grid_scroll.viewport_width, 400.0);
        assert_eq!(state.media_grid_scroll.content_width, 1200.0);
    }

    #[test]
    fn test_relative_position_for_scrolling() {
        // The relative position is the index normalised to [0.0, 1.0] over
        // the filtered entry list. We rely on this internally so that the
        // scroll resolves correctly even when our cached viewport snapshot
        // is briefly stale.
        assert_eq!(relative_position_for(0, 7), Some(0.0));
        assert_eq!(relative_position_for(6, 7), Some(1.0));
        assert!((relative_position_for(3, 7).unwrap() - 0.5).abs() < 1e-6);
        assert_eq!(relative_position_for(0, 0), None);
        assert_eq!(relative_position_for(0, 1), None);
        // Out-of-range indices are clamped to the last entry.
        assert_eq!(relative_position_for(99, 7), Some(1.0));
    }

    #[test]
    fn test_tick_should_exit_saves_settings() {
        let tmp =
            std::env::temp_dir().join(format!("mediasort_test_tick_save_{}", std::process::id()));
        let settings = SettingsStore {
            custom_path: Some(tmp.clone()),
            ..SettingsStore::default()
        };
        let mut state = AppState::new(settings);
        state.settings.general.theme = "Dark".to_string();
        state.should_exit = true;

        let _task = update(&mut state, Message::Tick(std::time::Instant::now()));

        let data = std::fs::read_to_string(&tmp).unwrap();
        let reloaded: SettingsStore = toml::from_str(&data).unwrap();
        assert_eq!(reloaded.general.theme, "Dark");

        let _ = std::fs::remove_file(&tmp);
    }
}
