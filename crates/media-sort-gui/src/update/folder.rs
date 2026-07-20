use iced::Task;

use crate::message::{FolderMessage, Message};
use crate::state::AppState;
use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::reversible::ReversibleAction;

pub fn handle_folder_message(state: &mut AppState, msg: FolderMessage) -> Task<Message> {
    match msg {
        FolderMessage::Open(path) => {
            state.open_folder(&path);
            Task::none()
        }
        FolderMessage::Pick => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |result| Message::Folder(FolderMessage::PickResult(result)),
        ),
        FolderMessage::PickResult(Some(path)) => {
            Task::done(Message::Folder(FolderMessage::Open(path)))
        }
        FolderMessage::PickResult(None) => Task::none(),
        FolderMessage::PickPin => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |result| Message::Folder(FolderMessage::PickPinResult(result)),
        ),
        FolderMessage::PickPinResult(Some(path)) => {
            state.pin_folder(&path);
            let _ = state.settings.save();
            Task::none()
        }
        FolderMessage::PickPinResult(None) => Task::none(),
        FolderMessage::SelectedPinned(path, idx) => {
            state.folder.set_selected(path.clone(), idx);
            state.folder.dragging_pinned_folder = Some(path);
            Task::none()
        }
        FolderMessage::DragPinnedOver(target_path) => {
            if let Some(source_path) = state.folder.dragging_pinned_folder.clone()
                && source_path != target_path
                && let Some(pos_source) = state
                    .folder
                    .pinned_folders
                    .iter()
                    .position(|p| p.path == source_path)
                && let Some(pos_target) = state
                    .folder
                    .pinned_folders
                    .iter()
                    .position(|p| p.path == target_path)
            {
                state.swap_pinned_folders(pos_source, pos_target);
            }
            Task::none()
        }
        FolderMessage::DragPinnedReleased => {
            if state.folder.dragging_pinned_folder.is_some() {
                state.folder.dragging_pinned_folder = None;
                let _ = state.settings.save();
            }
            Task::none()
        }
        FolderMessage::HoverPinned(path) => {
            state.folder.hovered_pinned_folder = Some(path);
            Task::none()
        }
        FolderMessage::HoverPinnedNone => {
            state.folder.hovered_pinned_folder = None;
            Task::none()
        }
        FolderMessage::Selected(path, idx) => {
            state.folder.set_selected(path, idx);
            Task::none()
        }
        FolderMessage::ToggleExpand(path) => {
            state.toggle_folder_expand(&path);
            Task::none()
        }
        FolderMessage::PinSelected => {
            let path_to_pin = state
                .folder
                .selected_folder
                .clone()
                .or(state.folder.current_folder.clone());
            if let Some(path) = path_to_pin {
                state.pin_folder(&path);
                let _ = state.settings.save();
            }
            Task::none()
        }
        FolderMessage::UnpinCurrent(path) => {
            state.unpin_folder(&path);
            let _ = state.settings.save();
            Task::none()
        }
        FolderMessage::MovePinnedUp(path) => {
            state.move_pinned_folder_up(&path);
            let _ = state.settings.save();
            Task::none()
        }
        FolderMessage::MovePinnedDown(path) => {
            state.move_pinned_folder_down(&path);
            let _ = state.settings.save();
            Task::none()
        }
        FolderMessage::TriggerCreate => {
            if let Some(p) = state
                .folder
                .selected_folder
                .as_ref()
                .or(state.folder.current_folder.as_ref())
            {
                state.create_folder.creating_folder_parent = Some(p.clone());
                state.create_folder.create_folder_input = String::new();
            }
            Task::none()
        }
        FolderMessage::CreateInputChanged(val) => {
            state.create_folder.create_folder_input = val;
            Task::none()
        }
        FolderMessage::SubmitCreate(_parent) => {
            if let Some(parent) = state.create_folder.creating_folder_parent.take() {
                let folder_name = state.create_folder.create_folder_input.trim().to_string();
                if !folder_name.is_empty() {
                    let new_dir = parent.join(&folder_name);
                    if let Err(e) = std::fs::create_dir_all(&new_dir) {
                        tracing::error!("Failed to create folder: {e}");
                    } else if state.folder.current_folder.is_some() {
                        state.build_folder_tree();
                    }
                }
                state.create_folder.create_folder_input.clear();
            }
            Task::none()
        }
        FolderMessage::CancelCreate => {
            state.create_folder.creating_folder_parent = None;
            state.create_folder.create_folder_input.clear();
            Task::none()
        }
        FolderMessage::PinShortcut(n) => {
            let pinned_idx = (n.saturating_sub(1)) as usize;
            if let Some(pinned) = state.folder.pinned_folders.get(pinned_idx) {
                let target_folder = pinned.path.clone();
                if let Some(index) = state.media_grid.selected_index {
                    let filtered = state.media_grid.filtered_entries();
                    if let Some(entry) = filtered.get(index) {
                        let entry_path = entry.path.clone();
                        match MoveAction::new(&entry_path, &target_folder) {
                            Ok(mut action) => {
                                if let Err(e) = action.execute() {
                                    tracing::error!("Move failed: {e}");
                                } else {
                                    state.history.push_executed(Box::new(action));
                                    state.media_grid.entries.retain(|e| e.path != entry_path);
                                    return super::tasks::select_and_load_entry(state, index);
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
    }
}
