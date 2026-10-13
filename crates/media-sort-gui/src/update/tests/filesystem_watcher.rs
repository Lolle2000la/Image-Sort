use super::*;

fn canonical(p: &std::path::Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

#[test]
fn test_added_file_refreshes_grid() {
    let dir = temp_media_dir("mediasort_fs_add");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);

    std::fs::write(dir.join("b.jpg"), b"x").unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Added(canonical(&dir.join("b.jpg")))]),
    );
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 2);
    assert!(
        state
            .media_grid
            .entries
            .iter()
            .any(|e| e.path == canonical(&dir.join("b.jpg")))
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_removed_file_updates_grid() {
    let dir = temp_media_dir("mediasort_fs_remove");
    let a = dir.join("a.jpg");
    let b = dir.join("b.jpg");
    std::fs::write(&a, b"x").unwrap();
    std::fs::write(&b, b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 2);

    std::fs::remove_file(&b).unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Removed(canonical(&b))]),
    );
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);
    assert!(
        !state
            .media_grid
            .entries
            .iter()
            .any(|e| e.path == canonical(&b))
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_event_during_initial_scan_is_followed_by_refresh() {
    let dir = temp_media_dir("mediasort_fs_coalesce");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    // The initial scan is in flight; the watcher event must be deferred
    // (pending_refresh) and trigger a second, replace-mode scan.
    std::fs::write(dir.join("late.jpg"), b"x").unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Added(canonical(
            &dir.join("late.jpg"),
        ))]),
    );
    assert!(state.media_grid.pending_refresh);
    drain_async_scan(&mut state);
    assert!(
        state
            .media_grid
            .entries
            .iter()
            .any(|e| e.path == canonical(&dir.join("late.jpg"))),
        "the refresh scan must pick up the file that arrived mid-scan"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_added_subfolder_refreshes_tree() {
    let dir = temp_media_dir("mediasort_fs_tree");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    drain_folder_tree(&mut state);

    let sub = dir.join("sub");
    std::fs::create_dir(&sub).unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Added(canonical(&sub))]),
    );
    drain_folder_tree(&mut state);
    assert!(
        state.tree_contains_path(&canonical(&sub)),
        "the new subfolder must appear in the folder tree"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_renamed_current_folder_is_followed() {
    let base = temp_media_dir("mediasort_fs_rename_base");
    let old = base.join("old");
    let new = base.join("new");
    std::fs::create_dir(&old).unwrap();
    std::fs::write(old.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&old);
    drain_async_scan(&mut state);
    drain_folder_tree(&mut state);

    let old_canonical = canonical(&old);
    assert_eq!(state.folder.current_folder, Some(old_canonical.clone()));

    std::fs::rename(&old, &new).unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Renamed(
            old_canonical,
            canonical(&new),
        )]),
    );
    assert_eq!(
        state.folder.current_folder,
        Some(canonical(&new)),
        "an externally renamed current folder must be followed"
    );
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);

    std::fs::remove_dir_all(&base).ok();
}

#[test]
fn test_removed_current_folder_moves_up_to_parent() {
    let base = temp_media_dir("mediasort_fs_removed_root");
    let child = base.join("child");
    std::fs::create_dir(&child).unwrap();
    std::fs::write(child.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&child);
    drain_async_scan(&mut state);
    drain_folder_tree(&mut state);

    let child_canonical = canonical(&child);
    std::fs::remove_dir_all(&child).unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Removed(child_canonical)]),
    );
    assert_eq!(
        state.folder.current_folder,
        Some(canonical(&base)),
        "a deleted current folder must move the view up to the closest existing ancestor"
    );
    drain_async_scan(&mut state);
    assert!(state.media_grid.entries.is_empty());

    std::fs::remove_dir_all(&base).ok();
}

#[test]
fn test_media_refresh_preserves_selection_by_path() {
    let dir = temp_media_dir("mediasort_fs_select");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();
    std::fs::write(dir.join("b.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);

    let _ = update(&mut state, Message::Media(MediaMessage::SelectEntry(1)));
    assert_eq!(state.media_grid.selected_index, Some(1));
    let selected = state.media_grid.entries[1].path.clone();

    std::fs::write(dir.join("c.jpg"), b"x").unwrap();
    state.start_media_refresh();
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 3);
    assert_eq!(
        state.media_grid.selected_index,
        Some(1),
        "selection index must follow the sorted position"
    );
    assert_eq!(
        state.media_grid.entries[state.media_grid.selected_index.unwrap()].path,
        selected,
        "the same file must stay selected after a refresh"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_media_refresh_falls_back_when_selected_file_removed() {
    let dir = temp_media_dir("mediasort_fs_select_removed");
    let a = dir.join("a.jpg");
    let b = dir.join("b.jpg");
    std::fs::write(&a, b"x").unwrap();
    std::fs::write(&b, b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);

    let _ = update(&mut state, Message::Media(MediaMessage::SelectEntry(1)));
    assert_eq!(state.media_grid.selected_index, Some(1));

    std::fs::remove_file(&b).unwrap();
    state.start_media_refresh();
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);
    assert!(
        state.media_grid.selected_index.is_some(),
        "a fallback entry must be selected after the previous one vanished"
    );
    assert_eq!(
        state.media_grid.entries[state.media_grid.selected_index.unwrap()].path,
        canonical(&a)
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_watched_directories_includes_current_and_expanded() {
    let dir = temp_media_dir("mediasort_fs_watched");
    let sub = dir.join("sub");
    // `sub` needs a subfolder of its own: an empty folder cannot be
    // expanded (no chevron), so its children are never displayed.
    std::fs::create_dir_all(sub.join("nested")).unwrap();
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    drain_folder_tree(&mut state);

    let watched = state.watched_directories();
    assert!(
        watched.contains(&canonical(&dir)),
        "the current folder must always be watched"
    );
    let parent = dir.parent().map(canonical).unwrap();
    assert!(
        watched.contains(&parent),
        "the current folder's parent must be watched so a rename/delete \
         of the current folder itself stays visible on every backend"
    );

    let sub_canonical = canonical(&sub);
    let sub_idx = state
        .folder
        .collect_visible_folders()
        .iter()
        .position(|p| p == &sub_canonical)
        .unwrap();
    state.toggle_folder_expand(&sub_canonical, sub_idx);
    let watched = state.watched_directories();
    assert!(
        watched.contains(&sub_canonical),
        "an expanded folder's children are displayed and must be watched"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_modified_dir_event_triggers_media_refresh() {
    // kqueue-style under-reporting: the only file in the folder is
    // deleted externally, the backend emits a directory-level
    // `Modified` (no per-child event), and the grid must still rescan
    // and drop the vanished entry.
    let dir = temp_media_dir("mediasort_fs_dir_modified");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);

    std::fs::remove_file(dir.join("a.jpg")).unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Modified(canonical(&dir))]),
    );
    drain_async_scan(&mut state);
    assert!(
        state.media_grid.entries.is_empty(),
        "a directory-level Modified must trigger a rescan that drops the vanished entry"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_modified_file_event_is_ignored() {
    // Content-only changes of files must not trigger rescans (they
    // change no visible structure).
    let dir = temp_media_dir("mediasort_fs_file_modified");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    // Let the initial tree rebuild from open_folder finish first, so
    // the assertions below observe only the event's own effects.
    drain_folder_tree(&mut state);

    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Modified(canonical(
            &dir.join("a.jpg"),
        ))]),
    );
    assert!(
        state.media_grid.scan_receiver.is_none(),
        "a file-level Modified must not start a media refresh"
    );
    assert!(
        state.folder.folder_tree_receiver.is_none() && !state.folder.tree_refresh_pending,
        "a file-level Modified must not start a tree rebuild"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_deleted_and_recreated_current_folder_rescans_and_bumps_generation() {
    // An external tool deletes the current folder and recreates it at
    // the same path before the watcher flush: the grid must rescan the
    // new content and the watch generation must change so the
    // subscription re-establishes the OS watch on the new directory.
    let dir = temp_media_dir("mediasort_fs_recreate");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    drain_folder_tree(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);

    let dir_canonical = canonical(&dir);
    let generation_before = state
        .watch_generation
        .load(std::sync::atomic::Ordering::Relaxed);

    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("b.jpg"), b"x").unwrap();

    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Removed(dir_canonical)]),
    );
    assert!(
        state
            .watch_generation
            .load(std::sync::atomic::Ordering::Relaxed)
            > generation_before,
        "a removed current folder must bump the watch generation so the \
         subscription restarts on the recreated directory"
    );
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);
    assert!(
        state
            .media_grid
            .entries
            .iter()
            .any(|e| e.path == canonical(&dir.join("b.jpg"))),
        "the grid must show the recreated folder's content"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_refresh_keeps_no_selection_when_none_was_made() {
    // An external change while nothing is selected must not silently
    // load entry 0.
    let dir = temp_media_dir("mediasort_fs_no_select");
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    state.media_grid.selected_index = None;
    state.cache.selected_image = None;

    std::fs::write(dir.join("b.jpg"), b"x").unwrap();
    state.start_media_refresh();
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 2);
    assert!(
        state.media_grid.selected_index.is_none(),
        "a refresh without a prior selection must keep the selection empty"
    );
    assert!(state.cache.selected_image.is_none());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_file_rename_skips_tree_rebuild() {
    // Renaming a FILE inside the current folder refreshes the media
    // grid but must not rebuild the folder tree (it shows folders
    // only).
    let dir = temp_media_dir("mediasort_fs_file_rename");
    std::fs::write(dir.join("old.jpg"), b"x").unwrap();

    let mut state = AppState::new(SettingsStore::default());
    state.open_folder(&dir);
    drain_async_scan(&mut state);
    drain_folder_tree(&mut state);

    std::fs::rename(dir.join("old.jpg"), dir.join("renamed.jpg")).unwrap();
    let _ = update(
        &mut state,
        Message::FileSystemChanged(vec![FileSystemEvent::Renamed(
            canonical(&dir.join("old.jpg")),
            canonical(&dir.join("renamed.jpg")),
        )]),
    );
    assert!(
        state.folder.folder_tree_receiver.is_none() && !state.folder.tree_refresh_pending,
        "a file rename must not start a tree rebuild"
    );
    drain_async_scan(&mut state);
    assert_eq!(state.media_grid.entries.len(), 1);
    assert!(
        state
            .media_grid
            .entries
            .iter()
            .any(|e| e.path == canonical(&dir.join("renamed.jpg"))),
        "the grid must show the renamed file"
    );

    std::fs::remove_dir_all(&dir).ok();
}
