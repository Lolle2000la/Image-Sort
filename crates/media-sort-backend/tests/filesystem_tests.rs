use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use media_sort_backend::filesystem::scanner::scan_media_files;
use media_sort_backend::filesystem::trash::delete_to_trash;
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
