use super::tasks::relative_position_for;
use super::*;
use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};
use crate::state::AppState;
use media_sort_core::actions::rename_action::RenameAction;
use media_sort_core::actions::reversible::ReversibleAction;
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
    assert_eq!(relative_position_for(0, 7), Some(0.0));
    assert_eq!(relative_position_for(6, 7), Some(1.0));
    assert!((relative_position_for(3, 7).unwrap() - 0.5).abs() < 1e-6);
    assert_eq!(relative_position_for(0, 0), None);
    assert_eq!(relative_position_for(0, 1), None);
    assert_eq!(relative_position_for(99, 7), Some(1.0));
}

#[test]
fn test_tick_should_exit_saves_settings() {
    let tmp = std::env::temp_dir().join(format!("mediasort_test_tick_save_{}", std::process::id()));
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

#[test]
fn test_pinned_folder_drag_and_drop() {
    let mut state = AppState::new(SettingsStore::default());
    let path1 = PathBuf::from("/pinned1");
    let path2 = PathBuf::from("/pinned2");

    state.pinned_folders = vec![
        media_sort_core::models::PinnedFolder {
            path: path1.clone(),
            name: "p1".into(),
            numeric_shortcut: None,
        },
        media_sort_core::models::PinnedFolder {
            path: path2.clone(),
            name: "p2".into(),
            numeric_shortcut: None,
        },
    ];
    state.current_folder = Some(PathBuf::from("/current"));
    state.build_folder_tree();

    let _ = update(
        &mut state,
        Message::Folder(FolderMessage::HoverPinned(path1.clone())),
    );
    assert_eq!(state.hovered_pinned_folder, Some(path1.clone()));

    let _ = update(&mut state, Message::Folder(FolderMessage::HoverPinnedNone));
    assert_eq!(state.hovered_pinned_folder, None);

    let _ = update(
        &mut state,
        Message::Folder(FolderMessage::SelectedPinned(path1.clone(), 1)),
    );
    assert_eq!(state.selected_folder, Some(path1.clone()));
    assert_eq!(state.dragging_pinned_folder, Some(path1.clone()));

    let _ = update(
        &mut state,
        Message::Folder(FolderMessage::DragPinnedOver(path2.clone())),
    );
    assert_eq!(state.pinned_folders[0].path, path2);
    assert_eq!(state.pinned_folders[1].path, path1);
    assert_eq!(state.dragging_pinned_folder, Some(path1.clone()));

    let _ = update(
        &mut state,
        Message::Folder(FolderMessage::DragPinnedReleased),
    );
    assert_eq!(state.dragging_pinned_folder, None);
}

#[test]
#[ignore = "spawns dbus-send/xdg-open which opens a file manager window"]
fn test_reveal_in_explorer_message() {
    let mut state = AppState::new(SettingsStore::default());
    let test_path = PathBuf::from("nonexistent_test_file_reveal.jpg");
    let _task = update(
        &mut state,
        Message::Media(MediaMessage::RevealInExplorer(test_path)),
    );
}
