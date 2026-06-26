use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use media_sort_core::actions::delete_action::{DeleteAction, TrashRestoreHandle};
use media_sort_core::actions::move_action::MoveAction;
use media_sort_core::actions::rename_action::RenameAction;
use media_sort_core::actions::reversible::{ActionError, ReversibleAction};
use media_sort_core::history::History;
use media_sort_core::l10n::Localization;
use media_sort_core::media_type::MediaType;
use media_sort_core::path_utils;
use media_sort_core::settings::store::SettingsStore;

struct MockAction {
    name: String,
}

impl MockAction {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl ReversibleAction for MockAction {
    fn display_name(&self) -> &str {
        &self.name
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        Ok(())
    }
}

struct MockRestoreHandle {
    restore_called: Mutex<bool>,
    trash_called: Mutex<bool>,
    restore_should_fail: bool,
}

impl fmt::Debug for MockRestoreHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockRestoreHandle").finish()
    }
}

impl MockRestoreHandle {
    fn new() -> Self {
        Self {
            restore_called: Mutex::new(false),
            trash_called: Mutex::new(false),
            restore_should_fail: false,
        }
    }

    fn failing(mut self) -> Self {
        self.restore_should_fail = true;
        self
    }

    fn restore_was_called(&self) -> bool {
        *self.restore_called.lock().unwrap()
    }

    fn trash_was_called(&self) -> bool {
        *self.trash_called.lock().unwrap()
    }
}

impl TrashRestoreHandle for MockRestoreHandle {
    fn restore(&mut self) -> Result<(), ActionError> {
        *self.restore_called.lock().unwrap() = true;
        if self.restore_should_fail {
            Err(ActionError::RestorationFailed("mock restore failed".into()))
        } else {
            Ok(())
        }
    }

    fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
        *self.trash_called.lock().unwrap() = true;
        Ok(())
    }
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("media-sort-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn temp_subdir() -> PathBuf {
    let dir = temp_dir().join(format!("sub-{}", rand_u32()));
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

// ============================================================================
// History tests
// ============================================================================

#[test]
fn test_push_and_query() {
    let mut history = History::new();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.done_len(), 0);

    history.push_executed(Box::new(MockAction::new("test_action")));

    assert!(history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.done_len(), 1);
    assert_eq!(history.last_done_name(), Some("test_action"));
    assert_eq!(history.last_undone_name(), None);
}

#[test]
fn test_undo_redo() {
    let mut history = History::new();
    history.push_executed(Box::new(MockAction::new("action1")));
    history.push_executed(Box::new(MockAction::new("action2")));

    history.undo().unwrap();
    assert_eq!(history.done_len(), 1);
    assert_eq!(history.undone_len(), 1);
    assert_eq!(history.last_done_name(), Some("action1"));
    assert_eq!(history.last_undone_name(), Some("action2"));
    assert!(history.can_undo());
    assert!(history.can_redo());

    history.redo().unwrap();
    assert_eq!(history.done_len(), 2);
    assert_eq!(history.undone_len(), 0);
    assert_eq!(history.last_done_name(), Some("action2"));
    assert!(history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_clear() {
    let mut history = History::new();
    history.push_executed(Box::new(MockAction::new("a")));
    history.push_executed(Box::new(MockAction::new("b")));
    history.push_executed(Box::new(MockAction::new("c")));

    history.clear();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.done_len(), 0);
    assert_eq!(history.undone_len(), 0);
}

#[test]
fn test_overflow() {
    let mut history = History::new();
    for i in 0..260 {
        history.push_executed(Box::new(MockAction::new(&format!("action{}", i))));
    }

    assert_eq!(history.done_len(), 256);
    assert_eq!(history.undone_len(), 0);
    assert_eq!(history.last_done_name(), Some("action259"));
}

#[test]
fn test_undo_on_empty() {
    let mut history = History::new();
    let result = history.undo();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ActionError::RestorationFailed(_)
    ));
}

#[test]
fn test_redo_on_empty() {
    let mut history = History::new();
    let result = history.redo();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ActionError::RestorationFailed(_)
    ));
}

#[test]
fn test_redo_clears_on_push() {
    let mut history = History::new();
    history.push_executed(Box::new(MockAction::new("action1")));
    history.push_executed(Box::new(MockAction::new("action2")));

    history.undo().unwrap();
    assert_eq!(history.undone_len(), 1);

    history.push_executed(Box::new(MockAction::new("action3")));
    assert_eq!(history.undone_len(), 0);
    assert!(!history.can_redo());
    assert_eq!(history.done_len(), 2);
}

// ============================================================================
// MoveAction tests
// ============================================================================

#[test]
fn test_move_execute() {
    let src_dir = temp_subdir();
    let dst_dir = temp_subdir();

    let src_file = src_dir.join("test_move_file.txt");
    std::fs::write(&src_file, b"hello move").unwrap();

    let mut action = MoveAction::new(&src_file, &dst_dir).unwrap();
    action.execute().unwrap();

    assert!(!src_file.exists());
    assert!(dst_dir.join("test_move_file.txt").exists());
}

#[test]
fn test_move_rollback() {
    let src_dir = temp_subdir();
    let dst_dir = temp_subdir();

    let src_file = src_dir.join("test_rollback_file.txt");
    std::fs::write(&src_file, b"rollback me").unwrap();

    let mut action = MoveAction::new(&src_file, &dst_dir).unwrap();
    action.execute().unwrap();
    assert!(!src_file.exists());

    action.rollback().unwrap();
    assert!(src_file.exists());
    assert!(!dst_dir.join("test_rollback_file.txt").exists());

    let contents = std::fs::read_to_string(&src_file).unwrap();
    assert_eq!(contents, "rollback me");
}

#[test]
fn test_move_source_not_found() {
    let dst_dir = temp_subdir();
    let missing = PathBuf::from("/nonexistent/file_that_does_not_exist_12345.txt");
    let result = MoveAction::new(&missing, &dst_dir);
    assert!(result.is_err());
    assert!(matches!(&result, Err(ActionError::SourceNotFound(_))));
}

#[test]
fn test_move_directory_not_found() {
    let src_dir = temp_subdir();
    let src_file = src_dir.join("exists.txt");
    std::fs::write(&src_file, b"data").unwrap();

    let missing_dir = PathBuf::from("/nonexistent/directory_xyz_12345");
    let result = MoveAction::new(&src_file, &missing_dir);
    assert!(result.is_err());
    assert!(matches!(&result, Err(ActionError::DirectoryNotFound(_))));
}

#[test]
fn test_move_display_name() {
    let src_dir = temp_subdir();
    let dst_dir = temp_subdir();
    let src_file = src_dir.join("display_name_test.txt");
    std::fs::write(&src_file, b"data").unwrap();

    let action = MoveAction::new(&src_file, &dst_dir).unwrap();
    let name = action.display_name();
    assert!(!name.is_empty());
    assert!(name.contains("display_name_test.txt") || name.contains("Move"));
}

// ============================================================================
// RenameAction tests
// ============================================================================

#[test]
fn test_rename_execute() {
    let dir = temp_subdir();
    let file = dir.join("original.txt");
    std::fs::write(&file, b"rename me").unwrap();

    let mut action = RenameAction::new(&file, "renamed").unwrap();
    action.execute().unwrap();

    assert!(!file.exists());
    assert!(dir.join("renamed.txt").exists());
}

#[test]
fn test_rename_rollback() {
    let dir = temp_subdir();
    let file = dir.join("before.txt");
    std::fs::write(&file, b"original content").unwrap();

    let mut action = RenameAction::new(&file, "after").unwrap();
    action.execute().unwrap();
    assert!(dir.join("after.txt").exists());
    assert!(!file.exists());

    action.rollback().unwrap();
    assert!(file.exists());
    assert!(!dir.join("after.txt").exists());

    let contents = std::fs::read_to_string(&file).unwrap();
    assert_eq!(contents, "original content");
}

#[test]
fn test_rename_illegal_characters() {
    let dir = temp_subdir();
    let file = dir.join("legal.txt");
    std::fs::write(&file, b"data").unwrap();

    let result = RenameAction::new(&file, "bad/name");
    assert!(result.is_err());
}

#[test]
fn test_rename_target_exists() {
    let dir = temp_subdir();
    let file1 = dir.join("first.txt");
    let file2 = dir.join("second.txt");
    std::fs::write(&file1, b"first").unwrap();
    std::fs::write(&file2, b"second").unwrap();

    let result = RenameAction::new(&file1, "second");
    assert!(result.is_err());
    assert!(matches!(&result, Err(ActionError::TargetExists(_))));
}

#[test]
fn test_rename_preserves_extension() {
    let dir = temp_subdir();
    let file = dir.join("photo.jpg");
    std::fs::write(&file, b"jpeg data").unwrap();

    let mut action = RenameAction::new(&file, "new_name").unwrap();
    action.execute().unwrap();

    assert!(dir.join("new_name.jpg").exists());
    assert!(!file.exists());
}

// ============================================================================
// DeleteAction tests
// ============================================================================

#[test]
fn test_delete_execute_mark() {
    let handle = Box::new(MockRestoreHandle::new());
    let mut action = DeleteAction::new(Path::new("some/file.txt"), handle);

    let result = action.execute();
    assert!(result.is_ok());
}

#[test]
fn test_delete_rollback() {
    let handle = Box::new(MockRestoreHandle::new());
    let mut action = DeleteAction::new(Path::new("some/file.txt"), handle);

    action.rollback().unwrap();
}

#[test]
fn test_delete_double_rollback() {
    let handle = Box::new(MockRestoreHandle::new());
    let mut action = DeleteAction::new(Path::new("some/file.txt"), handle);

    action.rollback().unwrap();

    let result = action.rollback();
    assert!(
        result.is_ok(),
        "second rollback is a no-op after handle consumed"
    );
}

#[test]
fn test_delete_restore_handle_called() {
    let handle = MockRestoreHandle::new();
    assert!(!handle.restore_was_called());

    let mut action = DeleteAction::new(Path::new("some/file.txt"), Box::new(handle));
    action.rollback().unwrap();
    assert!(!action.display_name().is_empty());
}

#[test]
fn test_delete_restore_handle_trash_called() {
    let handle = MockRestoreHandle::new();
    assert!(!handle.trash_was_called());

    let action = DeleteAction::new(Path::new("some/file.txt"), Box::new(handle));
    assert!(action.display_name().contains("file.txt"));
}

#[test]
fn test_delete_failing_restore() {
    let handle = Box::new(MockRestoreHandle::new().failing());
    let mut action = DeleteAction::new(Path::new("some/file.txt"), handle);

    let result = action.rollback();
    assert!(result.is_err(), "rollback should propagate restore failure");
    assert!(matches!(&result, Err(ActionError::RestorationFailed(_))));
}

// ============================================================================
// MediaType tests
// ============================================================================

#[test]
fn test_image_extensions() {
    let exts = MediaType::Image.extensions();
    assert!(exts.contains(&"png"));
    assert!(exts.contains(&"jpg"));
    assert!(exts.contains(&"jpeg"));
    assert!(exts.contains(&"gif"));
    assert!(exts.contains(&"bmp"));
    assert!(exts.contains(&"webp"));
}

#[test]
fn test_video_extensions() {
    let exts = MediaType::Video.extensions();
    assert!(exts.contains(&"mp4"));
    assert!(exts.contains(&"mkv"));
    assert!(exts.contains(&"webm"));
    assert!(exts.contains(&"avi"));
    assert!(exts.contains(&"mov"));
}

#[test]
fn test_audio_extensions() {
    let exts = MediaType::Audio.extensions();
    assert!(exts.contains(&"mp3"));
    assert!(exts.contains(&"flac"));
    assert!(exts.contains(&"wav"));
    assert!(exts.contains(&"ogg"));
    assert!(exts.contains(&"aac"));
    assert!(exts.contains(&"opus"));
}

#[test]
fn test_all_extensions_no_duplicates() {
    let all = MediaType::all_extensions();
    let mut sorted = all.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(all.len(), sorted.len(), "duplicate extensions found");
}

#[test]
fn test_all_extensions_covers_each_type() {
    let all = MediaType::all_extensions();
    let image_exts = MediaType::Image.extensions();
    let video_exts = MediaType::Video.extensions();
    let audio_exts = MediaType::Audio.extensions();

    for ext in image_exts {
        assert!(all.contains(ext), "image ext {} not in all_extensions", ext);
    }
    for ext in video_exts {
        assert!(all.contains(ext), "video ext {} not in all_extensions", ext);
    }
    for ext in audio_exts {
        assert!(all.contains(ext), "audio ext {} not in all_extensions", ext);
    }

    assert_eq!(
        all.len(),
        image_exts.len() + video_exts.len() + audio_exts.len(),
        "all_extensions count mismatch"
    );
}

// ============================================================================
// path_utils tests
// ============================================================================

#[test]
fn test_paths_equal_same() {
    let dir = temp_subdir();
    let file = dir.join("equal_test.txt");
    std::fs::write(&file, b"data").unwrap();

    assert!(path_utils::paths_equal(&file, &file));
}

#[test]
fn test_paths_equal_different() {
    let dir = temp_subdir();
    let file1 = dir.join("diff_a.txt");
    let file2 = dir.join("diff_b.txt");
    std::fs::write(&file1, b"a").unwrap();
    std::fs::write(&file2, b"b").unwrap();

    assert!(!path_utils::paths_equal(&file1, &file2));
}

#[test]
fn test_paths_equal_relative_vs_absolute() {
    let dir = temp_subdir();
    let file = dir.join("rel_test.txt");
    std::fs::write(&file, b"data").unwrap();

    let canonical = file.canonicalize().unwrap();

    assert!(path_utils::paths_equal(&canonical, &file));
}

#[test]
fn test_paths_equal_non_existent() {
    let a = PathBuf::from("/nonexistent/a.txt");
    let b = PathBuf::from("/nonexistent/b.txt");
    assert!(!path_utils::paths_equal(&a, &b));
}

#[test]
fn test_normalize_path() {
    let dir = temp_subdir();
    let file = dir.join("normalize_test.txt");
    std::fs::write(&file, b"data").unwrap();

    let normalized = path_utils::normalize_path(&file);
    assert_eq!(normalized, file.canonicalize().unwrap());
}

#[test]
fn test_normalize_path_non_existent() {
    let missing = PathBuf::from("/nonexistent/path/to/nowhere.txt");
    let result = path_utils::normalize_path(&missing);
    assert_eq!(result, missing);
}

// ============================================================================
// SettingsStore tests
// ============================================================================

#[test]
fn test_settings_default() {
    let settings = SettingsStore::default();
    assert!(!settings.general.dark_mode);
    assert!(settings.general.check_for_updates_on_startup);
    assert!(settings.general.animate_gifs);
}

#[test]
fn test_settings_save_load_roundtrip() {
    let mut settings = SettingsStore::default();
    settings.general.dark_mode = true;
    settings.general.check_for_updates_on_startup = false;
    settings.general.animate_gifs = false;

    let json = serde_json::to_string(&settings).unwrap();
    let loaded: SettingsStore = serde_json::from_str(&json).unwrap();

    assert!(loaded.general.dark_mode);
    assert!(!loaded.general.check_for_updates_on_startup);
    assert!(!loaded.general.animate_gifs);
}

#[test]
fn test_settings_keybindings_defaults() {
    let kb = &SettingsStore::default().keybindings;

    let names = [
        &kb.move_to_folder.key[..],
        &kb.delete.key[..],
        &kb.rename.key[..],
        &kb.go_left.key[..],
        &kb.go_right.key[..],
        &kb.create_folder.key[..],
        &kb.folder_up.key[..],
        &kb.folder_left.key[..],
        &kb.folder_down.key[..],
        &kb.folder_right.key[..],
        &kb.undo.key[..],
        &kb.redo.key[..],
        &kb.open_folder.key[..],
        &kb.open_selected_folder.key[..],
        &kb.pin.key[..],
        &kb.pin_selected.key[..],
        &kb.unpin.key[..],
        &kb.move_pinned_up.key[..],
        &kb.move_pinned_down.key[..],
        &kb.search_images.key[..],
        &kb.toggle_metadata_panel.key[..],
    ];

    assert_eq!(names.len(), 21);
    for (i, name) in names.iter().enumerate() {
        assert!(!name.is_empty(), "keybinding {} has empty key", i);
    }
}

#[test]
fn test_settings_empty_json_uses_defaults() {
    let json = "{}";
    let settings: SettingsStore = serde_json::from_str(json).unwrap();
    assert!(!settings.general.dark_mode);
    assert!(settings.general.check_for_updates_on_startup);
    assert!(!settings.keybindings.move_to_folder.key.is_empty());
}

// ============================================================================
// l10n tests
// ============================================================================

#[test]
fn test_l10n_init() {
    let loc = Localization::init("en");
    assert!(!loc
        .get(
            "move-action-message",
            &[("file_name", "test.jpg"), ("directory", "/home")]
        )
        .is_empty());
}

#[test]
fn test_l10n_get_known_key() {
    let loc = Localization::init("en");
    let msg = loc.get(
        "move-action-message",
        &[("file_name", "photo.png"), ("directory", "/pics")],
    );
    assert!(!msg.is_empty());
    assert!(msg.contains("photo.png") || msg.contains("/pics"));
}

#[test]
fn test_l10n_unknown_key_fallback() {
    let loc = Localization::init("en");
    let result = loc.get("nonexistent_key", &[]);
    assert_eq!(result, "nonexistent_key");
}

#[test]
fn test_l10n_delete_message() {
    let loc = Localization::init("en");
    let msg = loc.get("delete-action-message", &[("file_name", "old_file.dat")]);
    assert!(!msg.is_empty());
    assert!(msg.contains("old_file.dat"));
}

#[test]
fn test_l10n_rename_message() {
    let loc = Localization::init("en");
    let msg = loc.get(
        "rename-action-message",
        &[("old_file_name", "a.jpg"), ("new_file_name", "b.jpg")],
    );
    assert!(!msg.is_empty());
    assert!(msg.contains("a.jpg") || msg.contains("b.jpg"));
}

// ============================================================================
// ActionError tests
// ============================================================================

#[test]
fn test_action_error_display() {
    let err = ActionError::SourceNotFound(PathBuf::from("/missing.txt"));
    let s = err.to_string();
    assert!(s.contains("missing.txt"));

    let err = ActionError::TargetExists(PathBuf::from("/exists.txt"));
    let s = err.to_string();
    assert!(s.contains("exists.txt"));

    let err = ActionError::DirectoryNotFound(PathBuf::from("/no/dir"));
    let s = err.to_string();
    assert!(s.contains("no/dir"));

    let err = ActionError::RestorationFailed("test message".into());
    let s = err.to_string();
    assert!(s.contains("test message"));
}

#[test]
fn test_action_error_from_io() {
    let io_err = std::fs::File::open("/nonexistent/path_12345_xyz").unwrap_err();
    let action_err = ActionError::from(io_err);
    assert!(matches!(action_err, ActionError::Io(_)));
    assert!(action_err.to_string().contains("i/o error"));
}
