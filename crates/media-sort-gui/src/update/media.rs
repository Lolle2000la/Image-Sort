use iced::Task;

use crate::message::{MediaMessage, Message};
use crate::state::AppState;
use media_sort_core::actions::copy_action::CopyAction;
use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::rename_action::RenameAction;
use media_sort_core::actions::reversible::{ActionError, ReversibleAction};

pub fn handle_media_message(state: &mut AppState, msg: MediaMessage) -> Task<Message> {
    match msg {
        MediaMessage::SelectEntry(index) => super::tasks::select_and_load_entry(state, index),
        MediaMessage::SearchQueryChanged(query) => {
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
                super::tasks::select_and_load_entry(state, target_index)
            }
        }
        MediaMessage::MoveToFolder(target_folder) => {
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
                                return super::tasks::select_and_load_entry(state, index);
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
        MediaMessage::DeleteEntry(path) => {
            let index_to_select = state.selected_index.unwrap_or(0);
            match media_sort_backend::filesystem::trash::delete_to_trash(&path) {
                Ok(handle) => {
                    let action =
                        media_sort_core::actions::delete_action::DeleteAction::new(&path, handle);
                    state.history.push_executed(Box::new(action));
                    state.media_entries.retain(|e| e.path != path);
                    return super::tasks::select_and_load_entry(state, index_to_select);
                }
                Err(e) => {
                    tracing::error!("Cannot delete to trash: {e}");
                }
            }
            Task::none()
        }
        MediaMessage::TriggerRename => {
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
                    state.rename_error = None;
                }
            }
            Task::none()
        }
        MediaMessage::CopyToFolder(target_folder) => {
            let Some(index) = state.selected_index else {
                return Task::none();
            };
            let filtered = state.filtered_media_entries();
            let Some(entry) = filtered.get(index) else {
                return Task::none();
            };
            match CopyAction::new(&entry.path, &target_folder) {
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
        MediaMessage::RenameEntry(path, new_name) => {
            match RenameAction::new(&path, &new_name) {
                Ok(mut action) => {
                    if let Err(e) = action.execute() {
                        tracing::error!("Rename failed: {e}");
                    } else {
                        state.rename_error = None;
                        let new_path = action.new_path().to_path_buf();
                        state.history.push_executed(Box::new(action));
                        if let Some(pos) = state.media_entries.iter().position(|e| e.path == path) {
                            state.media_entries[pos].path = new_path.clone();
                            state.media_entries[pos].file_name = new_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| new_path.display().to_string());
                            state.renaming_path = None;
                            state.rename_input_value.clear();
                            return super::tasks::select_and_load_entry(state, pos);
                        }
                    }
                }
                Err(e) => {
                    if let ActionError::IllegalCharacters { character, .. } = &e {
                        state.rename_error = Some(state.l10n.get(
                            "ui-rename-illegal-char",
                            &[("char", &character.to_string())],
                        ));
                    }
                    tracing::error!("Cannot create rename action: {e}");
                }
            }
            Task::none()
        }
        MediaMessage::RenameInputChanged(val) => {
            let trimmed = val.trim().to_string();
            let error = RenameAction::validate_stem(&trimmed).err().map(|e| {
                match &e {
                    ActionError::IllegalCharacters { character, .. } if *character == '\0' => {
                        // Empty stem — no message, just disable submit button
                        String::new()
                    }
                    ActionError::IllegalCharacters { character, .. } => state.l10n.get(
                        "ui-rename-illegal-char",
                        &[("char", &character.to_string())],
                    ),
                    _ => e.to_string(),
                }
            });
            state.rename_input_value = val;
            state.rename_error = error;
            Task::none()
        }
        MediaMessage::SubmitRename => {
            let new_name = state.rename_input_value.trim().to_string();
            // Guard: don't submit empty or invalid stems
            if new_name.is_empty() || RenameAction::validate_stem(&new_name).is_err() {
                return Task::none();
            }
            if let Some(path) = state.renaming_path.take() {
                state.rename_input_value.clear();
                return Task::done(Message::Media(MediaMessage::RenameEntry(path, new_name)));
            }
            Task::none()
        }
        MediaMessage::CancelRename => {
            state.renaming_path = None;
            state.rename_input_value.clear();
            state.rename_error = None;
            Task::none()
        }
        MediaMessage::Undo => {
            let index = state.selected_index.unwrap_or(0);
            if let Err(e) = state.history.undo() {
                tracing::error!("Undo failed: {e}");
            } else {
                state.scan_media();
                return super::tasks::select_and_load_entry(state, index);
            }
            Task::none()
        }
        MediaMessage::Redo => {
            let index = state.selected_index.unwrap_or(0);
            if let Err(e) = state.history.redo() {
                tracing::error!("Redo failed: {e}");
            } else {
                state.scan_media();
                return super::tasks::select_and_load_entry(state, index);
            }
            Task::none()
        }
        MediaMessage::MetadataLoaded(result) => match result {
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
        MediaMessage::ThumbnailReady(path, w, h, data) => {
            if !data.is_empty() && w > 0 && h > 0 {
                let handle = iced::widget::image::Handle::from_rgba(w, h, data);
                state.thumbnail_cache.push(path, handle);
            }
            Task::none()
        }
        MediaMessage::ThumbnailFailed(path) => {
            state.unsupported_files.insert(path);
            Task::none()
        }
        MediaMessage::ThumbnailCancelled(_path) => Task::none(),
        MediaMessage::OpenExternal(path) => {
            super::tasks::open_externally(&path);
            Task::none()
        }
        MediaMessage::RevealInExplorer(path) => {
            super::tasks::reveal_in_file_manager(&path);
            Task::none()
        }
        MediaMessage::ImageLoaded(path, result) => {
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
        MediaMessage::GoLeft => {
            if let Some(idx) = state.selected_index
                && idx > 0
            {
                return super::tasks::select_and_load_entry(state, idx - 1);
            }
            Task::none()
        }
        MediaMessage::GoRight => {
            if let Some(idx) = state.selected_index {
                let filtered_len = state.filtered_media_entries().len();
                if idx + 1 < filtered_len {
                    return super::tasks::select_and_load_entry(state, idx + 1);
                }
            }
            Task::none()
        }
        MediaMessage::MoveActive => {
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
                                return super::tasks::select_and_load_entry(state, index);
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
        MediaMessage::CopyActive => {
            if let Some(index) = state.selected_index
                && let Some(ref target_folder) = state.selected_folder
            {
                let filtered = state.filtered_media_entries();
                if let Some(entry) = filtered.get(index) {
                    match CopyAction::new(&entry.path, target_folder) {
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
        MediaMessage::SearchFocused => {
            state.search_focused = true;
            iced::widget::operation::focus(crate::view::search_bar::SEARCH_INPUT_ID.clone())
        }
        MediaMessage::SearchBlurred => {
            state.search_focused = false;
            iced::advanced::widget::operate(iced::advanced::widget::operation::focusable::unfocus())
        }
        MediaMessage::GridScrolled(offset, viewport_width, content_width) => {
            state.media_grid_scroll.offset_x = offset.x;
            state.media_grid_scroll.viewport_width = viewport_width;
            state.media_grid_scroll.content_width = content_width;

            state.thumbnail_tracker.handle_scroll();
            Task::none()
        }
        MediaMessage::AudioPlayPause => {
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
        MediaMessage::StopAudio => {
            if let Some(ref player) = state.audio_player {
                player.stop();
            }
            state.audio_playing = false;
            state.audio_position = 0.0;
            Task::none()
        }
        MediaMessage::AudioSeek(pos) => {
            if let Some(ref player) = state.audio_player
                && let Err(e) = player.seek(pos)
            {
                tracing::error!("Audio seek failed: {e}");
            }
            Task::none()
        }
        MediaMessage::AudioSetVolume(vol) => {
            if let Some(ref player) = state.audio_player {
                player.set_volume(vol as f32 / 100.0);
            }
            state.audio_volume = vol;
            Task::none()
        }
        MediaMessage::AudioToggleMute => {
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
    }
}

pub fn handle_media_scan_completed(
    state: &mut AppState,
    result: Result<Vec<media_sort_core::models::MediaEntry>, String>,
) -> Task<Message> {
    match result {
        Ok(entries) => {
            state.media_entries = entries;
            let select_idx = state.pending_select_index.take().unwrap_or(0);
            Task::batch(vec![
                super::tasks::select_and_load_entry(state, select_idx),
                super::tasks::load_visible_thumbnails(state),
            ])
        }
        Err(err) => {
            tracing::error!("Asynchronous media retrieval failed: {}", err);
            Task::none()
        }
    }
}
