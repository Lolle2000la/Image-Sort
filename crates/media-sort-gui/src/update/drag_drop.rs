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

            // Symbolic links are refused up front: MoveAction/CopyAction
            // would resolve the link and relocate the link's TARGET instead
            // of the entry the user sees. Report the refusal via the status
            // banner instead of failing silently.
            let (symlink_paths, regular_paths): (Vec<_>, Vec<_>) = paths.iter().partition(|p| {
                p.symlink_metadata()
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
            });
            if !symlink_paths.is_empty() {
                // Cap the banner text: a drop with thousands of symlinks
                // must not allocate an unbounded string for the toast.
                const MAX_SHOWN_NAMES: usize = 3;
                let names = symlink_paths
                    .iter()
                    .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                    .collect::<Vec<_>>();
                let names = if names.len() > MAX_SHOWN_NAMES {
                    let shown = names[..MAX_SHOWN_NAMES].join(", ");
                    format!("{shown}, …")
                } else {
                    names.join(", ")
                };
                state.set_status(state.l10n.get(
                    "status-symlinks-skipped",
                    &[
                        ("count", &symlink_paths.len().to_string()),
                        ("names", &names),
                    ],
                ));
            }

            match target_zone {
                DragZone::Copy => {
                    let mut count = 0;
                    for src in &regular_paths {
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
                                report_refused_action(state, src, e);
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
                    for src in &regular_paths {
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
                                report_refused_action(state, src, e);
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
                        state.settings.mark_dirty();
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

/// Surfaces a refused action to the user. Symbolic-link refusals get a
/// status banner (the entry may have become a link between scan and action);
/// everything else is logged as before.
fn report_refused_action(
    state: &mut AppState,
    src: &std::path::Path,
    e: media_sort_core::actions::reversible::ActionError,
) {
    match e {
        media_sort_core::actions::reversible::ActionError::SourceIsSymlink(_) => {
            let name = src
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| src.display().to_string());
            state.set_status(state.l10n.get("status-symlink-refused", &[("name", &name)]));
        }
        other => {
            tracing::error!("Cannot create action for dropped file {src:?}: {other}");
        }
    }
}
