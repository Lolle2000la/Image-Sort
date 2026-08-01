use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use media_sort_backend::filesystem::scanner::scan_media_files;
use media_sort_backend::filesystem::trash::delete_to_trash;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        Self::new_in(&std::env::temp_dir(), prefix)
    }

    fn new_in(base: &Path, prefix: &str) -> Self {
        let pid = std::process::id();
        let count = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = base.join(format!("{}_{}_{}", prefix, pid, count));
        fs::create_dir_all(&dir).unwrap();
        Self { path: dir }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

impl Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn copy_fixture(fixture_name: &str, dest_dir: &Path) -> PathBuf {
    let src = fixtures_dir().join(fixture_name);
    let dest = dest_dir.join(fixture_name);
    fs::copy(&src, &dest).expect("failed to copy fixture");
    dest
}

#[test]
fn test_scan_empty_dir() {
    let tmp = TempDir::new("scan_empty");
    let results: Vec<_> = scan_media_files(tmp.path()).into_iter().collect();
    assert!(
        results.is_empty(),
        "expected empty vec for dir with no media files"
    );
}

#[test]
fn test_scan_images_only() {
    let tmp = TempDir::new("scan_images");
    copy_fixture("test_image.jpg", tmp.path());
    copy_fixture("test_image.png", tmp.path());

    let results: Vec<_> = scan_media_files(tmp.path()).into_iter().collect();
    assert_eq!(results.len(), 2, "expected 2 image files");
    let names: Vec<&str> = results
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .collect();
    assert!(names.contains(&"test_image.jpg"));
    assert!(names.contains(&"test_image.png"));
}

#[test]
fn test_scan_filtered_by_extension() {
    let tmp = TempDir::new("scan_filtered");
    copy_fixture("test_image.jpg", tmp.path());
    fs::write(tmp.join("notes.txt"), b"not media").unwrap();

    let results: Vec<_> = scan_media_files(tmp.path()).into_iter().collect();
    assert_eq!(results.len(), 1, "expected only the jpg, not the txt");
    let name = results[0].file_name().unwrap().to_str().unwrap();
    assert_eq!(name, "test_image.jpg");
}

#[test]
fn test_scan_no_recursion() {
    let tmp = TempDir::new("scan_norecurse");
    copy_fixture("test_image.jpg", tmp.path());
    let subdir = tmp.join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    copy_fixture("test_image.png", &subdir);

    let results: Vec<_> = scan_media_files(tmp.path()).into_iter().collect();
    assert_eq!(
        results.len(),
        1,
        "expected only top-level jpg, not the one in subdir"
    );
    let name = results[0].file_name().unwrap().to_str().unwrap();
    assert_eq!(name, "test_image.jpg");
}

#[test]
fn test_scan_with_fixtures() {
    let tmp = TempDir::new("scan_fixtures");
    copy_fixture("test_image.jpg", tmp.path());
    copy_fixture("test_image.png", tmp.path());
    copy_fixture("test_image.gif", tmp.path());
    copy_fixture("test_audio.mp3", tmp.path());
    copy_fixture("test_audio.flac", tmp.path());

    let results: Vec<_> = scan_media_files(tmp.path()).into_iter().collect();
    assert_eq!(results.len(), 5, "expected all 5 fixture files to be found");
}

// ============================================================
// Trash tests
// ============================================================

#[test]
fn test_delete_to_trash_no_filename() {
    let result = delete_to_trash(std::path::Path::new("/"));
    assert!(result.is_err(), "should fail for path with no file name");
}

// ============================================================
// Scanner channel disconnect tests
// ============================================================

#[test]
fn test_scanner_channel_disconnect() {
    let rx = scan_media_files(Path::new("/tmp"));
    drop(rx);
}

#[test]
fn test_scanner_nonexistent_directory() {
    let nonexistent = Path::new("/nonexistent/scanner_test_dir_12345");
    let results: Vec<_> = scan_media_files(nonexistent).into_iter().collect();
    assert!(results.is_empty());
}

// ============================================================
// Additional trash test
// ============================================================

#[test]
fn test_delete_to_trash_nonexistent_file() {
    let result = delete_to_trash(Path::new("/nonexistent/trash_test_12345.txt"));
    assert!(result.is_err());
}

#[test]
fn test_delete_to_trash_and_restore_roundtrip() {
    let tmp = TempDir::new("trash_roundtrip");
    let file = tmp.path.join("roundtrip.txt");
    fs::write(&file, b"trash roundtrip").unwrap();

    let mut handle = delete_to_trash(&file).expect("delete_to_trash failed");
    assert!(!file.exists(), "file should be in trash after delete");

    handle.restore().expect("restore failed");
    assert!(file.exists(), "file should be back after restore");
    assert_eq!(fs::read_to_string(&file).unwrap(), "trash roundtrip");
}

/// Windows regression test: the shell's trash display name
/// (SIGDN_PARENTRELATIVE) drops the extension for known file types
/// (Explorer's default "hide extensions" setting), which used to break
/// both finding the item and restoring it under its real name.
#[cfg(target_os = "windows")]
#[test]
fn test_windows_trash_restore_preserves_extension() {
    let tmp = TempDir::new("trash_ext");
    // .jpg is a "known file type" subject to Explorer's extension hiding.
    let file = tmp.path.join("photo.jpg");
    fs::write(&file, b"jpeg data").unwrap();

    let mut handle = delete_to_trash(&file).expect("delete_to_trash failed");
    assert!(!file.exists());

    handle.restore().expect("restore failed");

    assert!(
        file.exists(),
        "file must be restored WITH its .jpg extension"
    );
    assert!(
        !tmp.path.join("photo").exists(),
        "file must NOT be restored as an extension-less sibling"
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), "jpeg data");
}

/// Windows: when several items with the same name/extension from the same
/// folder sit in the recycle bin, restore must pick OUR (most recently
/// deleted) item, not an older same-name one.
#[cfg(target_os = "windows")]
#[test]
fn test_windows_trash_restore_picks_most_recent() {
    let tmp = TempDir::new("trash_recent");
    let file = tmp.path.join("same.txt");

    fs::write(&file, b"first version").unwrap();
    let mut handle1 = delete_to_trash(&file).expect("first delete failed");

    // time_deleted has second granularity; force the two deletes into
    // different seconds for a deterministic tiebreak.
    std::thread::sleep(std::time::Duration::from_millis(1100));

    fs::write(&file, b"second version").unwrap();
    let mut handle2 = delete_to_trash(&file).expect("second delete failed");

    // Restore OLDEST first: the trash crate's restore refuses to overwrite
    // the existing path (RestoreCollision), and undo semantics dictate the
    // newest entry is undone first anyway.
    handle1.restore().expect("first restore failed");
    assert!(file.exists());
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "first version",
        "restore must pick the oldest same-name item first"
    );

    // The file now collides with the newer trash entry; move it away so
    // the newer entry can be restored.
    let moved = tmp.path.join("moved_away.txt");
    fs::rename(&file, &moved).unwrap();

    handle2.restore().expect("second restore failed");
    assert!(file.exists());
    assert_eq!(fs::read_to_string(&file).unwrap(), "second version");
}
