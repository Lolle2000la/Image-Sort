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
pub struct FileWatcherHandle {
    _watcher: notify::RecommendedWatcher,
    _debounce_thread: std::thread::JoinHandle<()>,
}

/// Watches a single directory non-recursively (convenience wrapper around
/// [`watch_directories`]).
pub fn watch_directory(path: &Path) -> (FileWatcherHandle, mpsc::Receiver<FileSystemEvent>) {
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
/// rename + metadata churn) into a single batch per quiet window. The
/// `notify` event kinds are preserved through the debounce — the
/// debouncer-mini crate was rejected precisely because it reduces every
/// event to `(path, Any)`.
pub fn watch_directories(
    paths: &[PathBuf],
) -> (FileWatcherHandle, mpsc::Receiver<FileSystemEvent>) {
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
/// classified batch. Exits when the watcher side disconnects (flushing any
/// pending batch first).
fn debounce_loop(
    raw_rx: std::sync::mpsc::Receiver<notify::Event>,
    tx: mpsc::Sender<FileSystemEvent>,
) {
    const DEBOUNCE: Duration = Duration::from_millis(100);

    let mut pending: Vec<notify::Event> = Vec::new();
    let mut deadline: Option<Instant> = None;

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
                    for event in std::mem::take(&mut pending) {
                        for classified in classify_event(&event) {
                            let _ = tx.blocking_send(classified);
                        }
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                for event in std::mem::take(&mut pending) {
                    for classified in classify_event(&event) {
                        let _ = tx.blocking_send(classified);
                    }
                }
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
            // same event. Backends that cannot track renames deliver a
            // single path instead — degrade to `Modified`.
            if event.paths.len() >= 2 {
                vec![FileSystemEvent::Renamed(
                    PathBuf::from(&event.paths[0]),
                    PathBuf::from(&event.paths[1]),
                )]
            } else {
                event
                    .paths
                    .iter()
                    .map(|p| FileSystemEvent::Modified(PathBuf::from(p)))
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
    fn test_classify_rename_single_path_degrades_to_modified() {
        use notify::event::{ModifyKind, RenameMode};
        let ev = event(
            notify::EventKind::Modify(ModifyKind::Name(RenameMode::From)),
            &["/a/old.jpg"],
        );
        assert_eq!(
            classify_event(&ev),
            vec![FileSystemEvent::Modified(PathBuf::from("/a/old.jpg"))]
        );
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
