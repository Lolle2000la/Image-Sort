use iced::Task;

use media_sort_core::settings::keybindings::Key;

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};
use crate::state::AppState;
use crate::subscriptions::keyboard;

pub fn handle_key_captured(
    state: &mut AppState,
    key: Key,
    ctrl: bool,
    shift: bool,
    alt: bool,
) -> Task<Message> {
    if state.waiting_for_key {
        if let Some(idx) = state.editing_keybinding {
            let bindings = keyboard::keybinding_list(state);
            if idx < bindings.len() {
                let (name, _) = &bindings[idx];
                keyboard::update_keybinding(
                    &mut state.settings.keybindings,
                    name,
                    key,
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
        match key {
            Key::Enter => {
                return Task::done(Message::Media(MediaMessage::SubmitRename));
            }
            Key::Escape => {
                return Task::done(Message::Media(MediaMessage::CancelRename));
            }
            _ => return Task::none(),
        }
    }

    if state.creating_folder_parent.is_some() {
        match key {
            Key::Enter => {
                return Task::done(Message::Folder(FolderMessage::SubmitCreate(
                    std::path::PathBuf::new(),
                )));
            }
            Key::Escape => {
                return Task::done(Message::Folder(FolderMessage::CancelCreate));
            }
            _ => return Task::none(),
        }
    }

    if state.search_focused {
        match key {
            Key::Enter | Key::Escape | Key::Tab => {
                return Task::done(Message::Media(MediaMessage::SearchBlurred));
            }
            _ => return Task::none(),
        }
    }

    if key == Key::Space
        && !state.search_focused
        && state.renaming_path.is_none()
        && state.creating_folder_parent.is_none()
    {
        if let Some(ref sender) = state.video_sender {
            let _ =
                sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::TogglePause);
        }
        return Task::none();
    }

    if ctrl && key == Key::Character('Q') {
        return Task::done(Message::Quit);
    }

    if let Some(ref sender) = state.video_sender {
        match key {
            Key::MediaPlayPause | Key::MediaPlay | Key::MediaPause => {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::TogglePause);
                return Task::none();
            }
            Key::MediaStop => {
                let _ = sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Stop);
                return Task::none();
            }
            Key::AudioVolumeUp => {
                let new_vol = (state.video_volume + 5.0).min(100.0);
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetVolume(new_vol),
                );
                return Task::none();
            }
            Key::AudioVolumeDown => {
                let new_vol = (state.video_volume - 5.0).max(0.0);
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetVolume(new_vol),
                );
                return Task::none();
            }
            Key::AudioVolumeMute => {
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetMute(
                        !state.video_muted,
                    ),
                );
                return Task::none();
            }
            _ => {}
        }
    }

    match key {
        Key::MediaTrackNext => {
            return Task::done(Message::Media(MediaMessage::GoRight));
        }
        Key::MediaTrackPrevious => {
            return Task::done(Message::Media(MediaMessage::GoLeft));
        }
        _ => {}
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
                    return Task::done(Message::Folder(FolderMessage::Pick));
                }
                "toggle_metadata_panel" => {
                    return Task::done(Message::Settings(SettingsMessage::ToggleMetadataPanel));
                }
                "reveal_in_file_manager" => {
                    if let Some(index) = state.selected_index {
                        let filtered = state.filtered_media_entries();
                        if let Some(entry) = filtered.get(index) {
                            return Task::done(Message::Media(MediaMessage::RevealInExplorer(
                                entry.path.clone(),
                            )));
                        }
                    }
                }
                "pin" => {
                    return Task::done(Message::Folder(FolderMessage::PickPin));
                }
                "unpin" => {
                    if let Some(ref c) = state.current_folder {
                        return Task::done(Message::Folder(FolderMessage::UnpinCurrent(c.clone())));
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
                    return super::tasks::scroll_to_selected_folder(state);
                }
                "folder_down" => {
                    state.select_folder_below();
                    return super::tasks::scroll_to_selected_folder(state);
                }
                "folder_left" => {
                    state.collapse_selected_folder();
                    return super::tasks::scroll_to_selected_folder(state);
                }
                "folder_right" => {
                    state.expand_selected_folder();
                    return super::tasks::scroll_to_selected_folder(state);
                }
                "search_images" => {
                    return Task::done(Message::Media(MediaMessage::SearchFocused));
                }
                _ => {}
            }
        }
    }

    match key {
        Key::Character(c) if c.is_ascii_digit() && c != '0' && alt && !ctrl && !shift => {
            let digit = c.to_digit(10).unwrap() as u8;
            return Task::done(Message::Folder(FolderMessage::PinShortcut(digit)));
        }
        _ => {}
    }

    Task::none()
}
