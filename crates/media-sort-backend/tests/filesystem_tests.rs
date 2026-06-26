use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use media_sort_backend::filesystem::scanner::scan_media_files;
use media_sort_backend::filesystem::trash_staging::TrashStaging;
use media_sort_backend::filesystem::watcher::FileSystemEvent;

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

    fn cleanup_staging_root() {
        if let Some(config_dir) = dirs::config_dir() {
            let staging_root = config_dir.join("media-sort").join("trash");
            let _ = fs::remove_dir_all(&staging_root);
        }
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
        .join("..")
        .join("..")
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
    let results = scan_media_files(tmp.path());
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

    let results = scan_media_files(tmp.path());
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

    let results = scan_media_files(tmp.path());
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

    let results = scan_media_files(tmp.path());
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

    let results = scan_media_files(tmp.path());
    assert_eq!(results.len(), 5, "expected all 5 fixture files to be found");
}

fn config_media_sort_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("media-sort")
}

#[test]
fn test_stage_and_restore() {
    TempDir::cleanup_staging_root();

    let base = config_media_sort_dir();
    let tmp = TempDir::new_in(&base, "stage_restore");
    let file = copy_fixture("test_image.jpg", tmp.path());
    let original_content = fs::read(&file).unwrap();

    let staging = TrashStaging::new().expect("failed to create TrashStaging");

    let mut handle = staging.stage_file(&file).expect("failed to stage file");
    assert!(!file.exists(), "original file should be gone after staging");

    handle.restore().expect("failed to restore file");
    assert!(file.exists(), "original file should be back after restore");
    let restored_content = fs::read(&file).unwrap();
    assert_eq!(
        original_content, restored_content,
        "restored file content should match original"
    );

    TempDir::cleanup_staging_root();
}

#[test]
fn test_double_stage() {
    TempDir::cleanup_staging_root();

    let base = config_media_sort_dir();
    let tmp = TempDir::new_in(&base, "double_stage");
    let file1 = copy_fixture("test_image.jpg", tmp.path());
    let file2 = copy_fixture("test_image.png", tmp.path());

    let staging = TrashStaging::new().expect("failed to create TrashStaging");

    let mut handle1 = staging
        .stage_file(&file1)
        .expect("failed to stage first file");
    let mut handle2 = staging
        .stage_file(&file2)
        .expect("failed to stage second file");

    assert!(!file1.exists(), "first file should be staged");
    assert!(!file2.exists(), "second file should be staged");

    handle1.restore().expect("failed to restore first file");
    handle2.restore().expect("failed to restore second file");

    assert!(file1.exists(), "first file should be restored");
    assert!(file2.exists(), "second file should be restored");

    TempDir::cleanup_staging_root();
}

#[test]
#[ignore = "calls trash::delete() which interacts with the system trash daemon (KDE)"]
fn test_stage_and_flush() {
    TempDir::cleanup_staging_root();

    let base = config_media_sort_dir();
    let tmp = TempDir::new_in(&base, "stage_flush");
    let file = copy_fixture("test_image.jpg", tmp.path());

    let staging = TrashStaging::new().expect("failed to create TrashStaging");

    let mut handle = staging.stage_file(&file).expect("failed to stage file");
    assert!(!file.exists(), "original should be gone");

    handle
        .flush_to_native_trash()
        .expect("failed to flush to native trash");

    match handle.restore() {
        Err(e) => assert!(e.to_string().contains("already flushed")),
        Ok(()) => panic!("restore after flush should fail"),
    }

    TempDir::cleanup_staging_root();
}

#[test]
fn test_orphan_reconciliation_no_trash() {
    TempDir::cleanup_staging_root();

    let staging_root = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("media-sort")
        .join("trash");

    fs::create_dir_all(&staging_root).unwrap();

    let orphan_dir1 = staging_root.join("0000000000000001");
    fs::create_dir_all(&orphan_dir1).unwrap();
    fs::write(orphan_dir1.join("orphan.txt"), b"leftover").unwrap();

    let orphan_dir2 = staging_root.join("0000000000000002");
    fs::create_dir_all(&orphan_dir2).unwrap();
    fs::write(orphan_dir2.join("orphan2.txt"), b"also leftover").unwrap();

    TrashStaging::reconcile_orphaned_trash(&staging_root);

    assert!(!orphan_dir1.exists(), "orphan dir 1 should be cleaned up");
    assert!(!orphan_dir2.exists(), "orphan dir 2 should be cleaned up");
}

#[test]
#[ignore = "calls flush_all_to_native() which uses trash::delete()"]
fn test_flush_all() {
    TempDir::cleanup_staging_root();

    let base = config_media_sort_dir();
    let tmp = TempDir::new_in(&base, "flush_all");
    let file1 = copy_fixture("test_image.jpg", tmp.path());
    let file2 = copy_fixture("test_image.png", tmp.path());

    let staging = TrashStaging::new().expect("failed to create TrashStaging");

    let mut handle1 = staging.stage_file(&file1).expect("failed to stage file1");
    let mut handle2 = staging.stage_file(&file2).expect("failed to stage file2");

    staging.flush_all_to_native();

    assert!(
        handle1.restore().is_err(),
        "handle1 should fail after flush_all"
    );
    assert!(
        handle2.restore().is_err(),
        "handle2 should fail after flush_all"
    );

    TempDir::cleanup_staging_root();
}

#[test]
#[ignore = "Drop calls flush_to_native_trash() which uses trash::delete()"]
fn test_drop_flushes() {
    TempDir::cleanup_staging_root();

    let base = config_media_sort_dir();
    let tmp = TempDir::new_in(&base, "drop_flushes");
    let file = copy_fixture("test_image.jpg", tmp.path());

    let staging = TrashStaging::new().expect("failed to create TrashStaging");

    let handle = staging.stage_file(&file).expect("failed to stage file");

    let staging_path = {
        let staging_root = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort")
            .join("trash");
        let staged_entries: Vec<_> = fs::read_dir(&staging_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !staged_entries.is_empty(),
            "staging dir should have entries"
        );
        staged_entries[0].path()
    };
    assert!(
        staging_path.exists(),
        "staged copy should exist before drop"
    );

    drop(handle);

    assert!(
        !staging_path.exists(),
        "staged copy should be gone after drop"
    );

    TempDir::cleanup_staging_root();
}

// ============================================================
// FileSystemEvent constructors
// ============================================================

#[test]
fn test_file_system_event_added() {
    let p = std::path::PathBuf::from("/tmp/test.txt");
    let ev = FileSystemEvent::Added(p.clone());
    match ev {
        FileSystemEvent::Added(ref path) => assert_eq!(path, &p),
        _ => panic!("Expected Added variant"),
    }
}

#[test]
fn test_file_system_event_removed() {
    let p = std::path::PathBuf::from("/tmp/test.txt");
    let ev = FileSystemEvent::Removed(p.clone());
    match ev {
        FileSystemEvent::Removed(ref path) => assert_eq!(path, &p),
        _ => panic!("Expected Removed variant"),
    }
}

#[test]
fn test_file_system_event_modified() {
    let p = std::path::PathBuf::from("/tmp/test.txt");
    let ev = FileSystemEvent::Modified(p.clone());
    match ev {
        FileSystemEvent::Modified(ref path) => assert_eq!(path, &p),
        _ => panic!("Expected Modified variant"),
    }
}

#[test]
fn test_file_system_event_renamed() {
    let old = std::path::PathBuf::from("/tmp/old.txt");
    let new = std::path::PathBuf::from("/tmp/new.txt");
    let ev = FileSystemEvent::Renamed(old.clone(), new.clone());
    match ev {
        FileSystemEvent::Renamed(ref o, ref n) => {
            assert_eq!(o, &old);
            assert_eq!(n, &new);
        }
        _ => panic!("Expected Renamed variant"),
    }
}

#[test]
fn test_file_system_event_debug() {
    let ev = FileSystemEvent::Modified(std::path::PathBuf::from("/tmp/test.txt"));
    let dbg = format!("{:?}", ev);
    assert!(dbg.contains("Modified"));
    assert!(dbg.contains("test.txt"));
}
