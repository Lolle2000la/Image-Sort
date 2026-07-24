use iced::Task;

use crate::message::{DragDropMessage, Message};
use crate::state::AppState;
use crate::state::drag_drop::{DragDropMode, DragZone};
use media_sort_core::actions::copy_action::CopyAction;
use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::reversible::ReversibleAction;

pub fn handle_drag_drop_message(state: &mut AppState, msg: DragDropMessage) -> Task<Message> {
    match msg {
        DragDropMessage::FileHovered(path) => {
            tracing::info!("DragDrop: FileHovered({path:?})");
            let has_open_folder = state.folder.current_folder.is_some();
            state.drag_drop.add_path(path, has_open_folder);
            if let Some(pos) = state.drag_drop.last_cursor_position {
                let win_w = state.settings.window_position.width as f32;
                let win_h = state.settings.window_position.height as f32;
                state.drag_drop.update_cursor(pos, (win_w, win_h));
            }
            Task::none()
        }
        DragDropMessage::FileHoveredCancelled => {
            tracing::info!("DragDrop: FileHoveredCancelled");
            state.drag_drop.reset();
            Task::none()
        }
        DragDropMessage::FileDropped(path) => {
            tracing::info!("DragDrop: FileDropped({path:?})");
            let has_open_folder = state.folder.current_folder.is_some();
            state.drag_drop.add_path(path, has_open_folder);
            if let Some(pos) = state.drag_drop.last_cursor_position {
                let win_w = state.settings.window_position.width as f32;
                let win_h = state.settings.window_position.height as f32;
                state.drag_drop.update_cursor(pos, (win_w, win_h));
            }
            execute_drop(state)
        }
        DragDropMessage::ZoneHovered(zone) => {
            if state.drag_drop.mode != DragDropMode::None
                && !matches!(state.drag_drop.mode, DragDropMode::Denied(_))
            {
                state.drag_drop.target_zone = zone;
            }
            Task::none()
        }
    }
}

pub fn execute_drop(state: &mut AppState) -> Task<Message> {
    let mode = state.drag_drop.mode.clone();
    let target_zone = state.drag_drop.target_zone;
    let paths = state.drag_drop.dragged_paths.clone();

    state.drag_drop.reset();

    match mode {
        DragDropMode::Denied(_) | DragDropMode::None => Task::none(),
        DragDropMode::Files {
            has_destination: true,
        } => {
            let Some(dest_dir) = state.folder.current_folder.clone() else {
                return Task::none();
            };

            match target_zone {
                DragZone::Copy => {
                    let mut count = 0;
                    for src in &paths {
                        match CopyAction::new(src, &dest_dir) {
                            Ok(mut action) => {
                                if let Err(e) = action.execute() {
                                    tracing::error!("Failed to copy dropped file {src:?}: {e}");
                                } else {
                                    state.history.push_executed(Box::new(action));
                                    count += 1;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Cannot create CopyAction for dropped file {src:?}: {e}"
                                );
                            }
                        }
                    }
                    if count > 0
                        && let Some(ref current) = state.folder.current_folder.clone()
                    {
                        state.open_folder(current);
                    }
                }
                DragZone::Move => {
                    let mut moved_count = 0;
                    for src in &paths {
                        match MoveAction::new(src, &dest_dir) {
                            Ok(mut action) => {
                                if let Err(e) = action.execute() {
                                    tracing::error!("Failed to move dropped file {src:?}: {e}");
                                } else {
                                    state.history.push_executed(Box::new(action));
                                    moved_count += 1;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Cannot create MoveAction for dropped file {src:?}: {e}"
                                );
                            }
                        }
                    }
                    if moved_count > 0
                        && let Some(ref current) = state.folder.current_folder.clone()
                    {
                        state.open_folder(current);
                    }
                }
                _ => {}
            }
            Task::none()
        }
        DragDropMode::SingleFolder => {
            if let Some(folder_path) = paths.first() {
                match target_zone {
                    DragZone::Open => {
                        state.open_folder(folder_path);
                    }
                    DragZone::Pin => {
                        state.pin_folder(folder_path);
                        let _ = state.settings.save();
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
