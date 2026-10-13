use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemEvent {
    Added(PathBuf),
    Removed(PathBuf),
    Modified(PathBuf),
    Renamed(PathBuf, PathBuf),
}

/// Holds the OS watcher and the debounce thread alive. Dropping it stops
/// both: the watcher's drop disconnects the raw event channel, which makes
/// the debounce thread flush its pending batch and exit.
///
/// Field drop order is load-bearing: `_watcher` must drop BEFORE
/// `_debounce_thread` so the channel disconnect happens first (Rust drops
/// struct fields in declaration order). Do not reorder.
pub struct FileWatcherHandle {
    _watcher: notify::RecommendedWatcher,
    _debounce_thread: std::thread::JoinHandle<()>,
}

/// Watches a single directory non-recursively (convenience wrapper around
/// [`watch_directories`]).
pub fn watch_directory(path: &Path) -> (FileWatcherHandle, mpsc::Receiver<Vec<FileSystemEvent>>) {
    watch_directories(&[path.to_path_buf()])
}

/// Watches every directory in `paths` non-recursively with a single OS
/// watcher. Unwatchable paths (deleted, not a directory, permission denied)
/// are skipped with a warning instead of failing the whole set.
///
/// Events are classified into [`FileSystemEvent`]s: `Create` → `Added`,
/// `Remove` → `Removed`, `Modify(Name)` with two paths → `Renamed`, other
/// `Modify` → `Modified`. `Access` events (thumbnail/backup scans touch
/// files constantly) are dropped entirely. Watcher errors degrade to
/// `Modified` for the offending path so a consumer never misses the fact
/// that something happened.
///
/// A 100 ms debounce coalesces bursts (e.g. an editor's save = write +
/// rename + metadata churn) into a single batch per quiet window, and each
/// batch is delivered as ONE channel message (a 500-file copy = 1 message,
/// not 500) so consumers process bursts as a unit. The `notify` event kinds
/// are preserved through the debounce — the debouncer-mini crate was
/// rejected precisely because it reduces every event to `(path, Any)`.
///
/// Note the platform backends vary in precision; the classification in
/// [`classify_event`] documents each case. In particular the macOS backend
/// (kqueue, pinned via the workspace `macos_kqueue` feature) cannot pair
/// rename sides and under-reports directory-child changes, so consumers
/// should treat any event batch as "something changed, re-scan" rather
/// than trusting it as a complete diff.
pub fn watch_directories(
    paths: &[PathBuf],
) -> (FileWatcherHandle, mpsc::Receiver<Vec<FileSystemEvent>>) {
    let (tx, rx) = mpsc::channel(256);
    let watch_paths: Vec<PathBuf> = paths
        .iter()
        .filter(|p| !p.as_os_str().is_empty())
        .cloned()
        .collect();

    let (raw_tx, raw_rx) = std::sync::mpsc::channel::<notify::Event>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let _ = raw_tx.send(event);
        }
    })
    .expect("file watcher setup");

    for path in &watch_paths {
        if !path.is_dir() {
            continue;
        }
        if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
            tracing::warn!(
                "Could not watch {} for filesystem changes: {e}",
                path.display()
            );
        }
    }

    let debounce_thread = std::thread::Builder::new()
        .name("fs-watcher-debounce".into())
        .spawn(move || debounce_loop(raw_rx, tx))
        .expect("file watcher debounce thread");

    let handle = FileWatcherHandle {
        _watcher: watcher,
        _debounce_thread: debounce_thread,
    };
    (handle, rx)
}

/// Collects raw `notify` events for a 100 ms quiet window and forwards the
/// classified batch as a single message. Exits when the watcher side
/// disconnects (flushing any pending batch first).
fn debounce_loop(
    raw_rx: std::sync::mpsc::Receiver<notify::Event>,
    tx: mpsc::Sender<Vec<FileSystemEvent>>,
) {
    const DEBOUNCE: Duration = Duration::from_millis(100);

    let mut pending: Vec<notify::Event> = Vec::new();
    let mut deadline: Option<Instant> = None;

    let flush = |pending: &mut Vec<notify::Event>| {
        let batch: Vec<FileSystemEvent> = std::mem::take(pending)
            .iter()
            .flat_map(classify_event)
            .collect();
        if !batch.is_empty() {
            let _ = tx.blocking_send(batch);
        }
    };

    loop {
        let timeout = deadline
            .map(|d| d.saturating_duration_since(Instant::now()))
            .unwrap_or(Duration::from_millis(1000));
        match raw_rx.recv_timeout(timeout) {
            Ok(event) => {
                pending.push(event);
                if deadline.is_none() {
                    deadline = Some(Instant::now() + DEBOUNCE);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if deadline.is_some_and(|d| Instant::now() >= d) {
                    deadline = None;
                    flush(&mut pending);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                flush(&mut pending);
                break;
            }
        }
    }
}

fn classify_event(event: &notify::Event) -> Vec<FileSystemEvent> {
    use notify::EventKind;
    use notify::event::ModifyKind;

    match event.kind {
        // Access events are noise (every thumbnail/backup/index scan
        // touches files); they never change the visible file set.
        EventKind::Access(_) => Vec::new(),
        EventKind::Create(_) => event
            .paths
            .iter()
            .map(|p| FileSystemEvent::Added(PathBuf::from(p)))
            .collect(),
        EventKind::Remove(_) => event
            .paths
            .iter()
            .map(|p| FileSystemEvent::Removed(PathBuf::from(p)))
            .collect(),
        EventKind::Modify(ModifyKind::Name(_)) => {
            // A rename carries both the source and destination path on the
            // same event when the backend can pair them (inotify). Windows
            // (ReadDirectoryChangesWatcher) delivers the two sides as
            // separate single-path events (`RenameMode::From` /
            // `RenameMode::To`), and macOS (kqueue, pinned via the
            // workspace `macos_kqueue` feature) emits `RenameMode::Any`
            // with the old path only. Classify a single-path side by
            // existence: the side that still exists is the destination
            // (`Added`), the vanished side is the source (`Removed`). This
            // keeps renames visible to the GUI on every platform instead
            // of degrading them to `Modified` (which the GUI deliberately
            // ignores for files).
            //
            // The existence check runs at debounce-flush time (100 ms+
            // after the event), so a create-then-delete within the window
            // misclassifies — acceptable for a refresh heuristic, since
            // the consumer responds with a full rescan either way.
            if event.paths.len() >= 2 {
                vec![FileSystemEvent::Renamed(
                    PathBuf::from(&event.paths[0]),
                    PathBuf::from(&event.paths[1]),
                )]
            } else {
                event
                    .paths
                    .iter()
                    .map(|p| {
                        if p.exists() {
                            FileSystemEvent::Added(PathBuf::from(p))
                        } else {
                            FileSystemEvent::Removed(PathBuf::from(p))
                        }
                    })
                    .collect()
            }
        }
        EventKind::Modify(_) => event
            .paths
            .iter()
            .map(|p| FileSystemEvent::Modified(PathBuf::from(p)))
            .collect(),
        EventKind::Any | EventKind::Other => event
            .paths
            .iter()
            .map(|p| FileSystemEvent::Modified(PathBuf::from(p)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: notify::EventKind, paths: &[&str]) -> notify::Event {
        notify::Event {
            kind,
            paths: paths.iter().map(std::path::PathBuf::from).collect(),
            attrs: Default::default(),
        }
    }

    #[test]
    fn test_classify_create() {
        use notify::event::CreateKind;
        let ev = event(notify::EventKind::Create(CreateKind::File), &["/a/new.jpg"]);
        assert_eq!(
            classify_event(&ev),
            vec![FileSystemEvent::Added(PathBuf::from("/a/new.jpg"))]
        );
    }

    #[test]
    fn test_classify_remove() {
        use notify::event::RemoveKind;
        let ev = event(
            notify::EventKind::Remove(RemoveKind::File),
            &["/a/gone.jpg"],
        );
        assert_eq!(
            classify_event(&ev),
            vec![FileSystemEvent::Removed(PathBuf::from("/a/gone.jpg"))]
        );
    }

    #[test]
    fn test_classify_rename_two_paths() {
        use notify::event::{ModifyKind, RenameMode};
        let ev = event(
            notify::EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            &["/a/old.jpg", "/a/new.jpg"],
        );
        assert_eq!(
            classify_event(&ev),
            vec![FileSystemEvent::Renamed(
                PathBuf::from("/a/old.jpg"),
                PathBuf::from("/a/new.jpg")
            )]
        );
    }

    #[test]
    fn test_classify_rename_from_side_becomes_removed() {
        use notify::event::{ModifyKind, RenameMode};
        // Windows delivers the old side of a rename as a single-path
        // `Name(From)` event; the path no longer exists → `Removed`.
        let ev = event(
            notify::EventKind::Modify(ModifyKind::Name(RenameMode::From)),
            &["/a/old.jpg"],
        );
        assert_eq!(
            classify_event(&ev),
            vec![FileSystemEvent::Removed(PathBuf::from("/a/old.jpg"))]
        );
    }

    #[test]
    fn test_classify_rename_to_side_becomes_added() {
        use notify::event::{ModifyKind, RenameMode};
        // Windows delivers the new side of a rename as a single-path
        // `Name(To)` event; the path exists → `Added`.
        let dir =
            std::env::temp_dir().join(format!("mediasort_watcher_cls_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let new_path = dir.join("new.jpg");
        std::fs::write(&new_path, b"data").unwrap();

        let ev = event(
            notify::EventKind::Modify(ModifyKind::Name(RenameMode::To)),
            &[new_path.to_str().unwrap()],
        );
        assert_eq!(classify_event(&ev), vec![FileSystemEvent::Added(new_path)]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_classify_access_is_dropped() {
        use notify::event::AccessKind;
        let ev = event(notify::EventKind::Access(AccessKind::Read), &["/a/x.jpg"]);
        assert!(classify_event(&ev).is_empty());
    }

    #[test]
    fn test_classify_modify_data() {
        use notify::event::{DataChange, ModifyKind};
        let ev = event(
            notify::EventKind::Modify(ModifyKind::Data(DataChange::Content)),
            &["/a/x.jpg"],
        );
        assert_eq!(
            classify_event(&ev),
            vec![FileSystemEvent::Modified(PathBuf::from("/a/x.jpg"))]
        );
    }
}
