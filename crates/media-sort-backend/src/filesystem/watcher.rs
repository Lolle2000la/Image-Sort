use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum FileSystemEvent {
    Added(PathBuf),
    Removed(PathBuf),
    Modified(PathBuf),
    Renamed(PathBuf, PathBuf),
}

pub struct FileWatcherHandle {
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

pub fn watch_directory(path: &Path) -> (FileWatcherHandle, mpsc::Receiver<FileSystemEvent>) {
    let (tx, rx) = mpsc::channel(256);
    let path = path.to_path_buf();

    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        move |events: DebounceEventResult| match events {
            Ok(events) => {
                for event in &events {
                    for p in &event.path {
                        let _ = tx.blocking_send(FileSystemEvent::Modified(PathBuf::from(p)));
                    }
                }
            }
            Err(error) => {
                for p in &error.paths {
                    let _ = tx.blocking_send(FileSystemEvent::Modified(PathBuf::from(p)));
                }
            }
        },
    )
    .expect("file watcher setup");

    debouncer
        .watcher()
        .watch(&path, RecursiveMode::NonRecursive)
        .expect("watch directory");

    let handle = FileWatcherHandle {
        _debouncer: debouncer,
    };
    (handle, rx)
}
