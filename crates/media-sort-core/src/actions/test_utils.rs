//! Shared test helpers for the `media-sort-core` crate.
//!
//! Unit tests live inside the library crate (`src/`), so helpers must live
//! there too — the `tests/` directory is a separate crate and cannot share
//! code with them. This module is the single implementation of the unique
//! temp-directory helper used by the action and `path_utils` test modules.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("media-sort-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// A uniquely-named directory under the crate's per-process temp root.
///
/// A process-global monotonic counter guarantees uniqueness across parallel
/// test threads; the nanos-derived `rand()` the individual test modules used
/// before could collide when tests ran in quick succession, sharing (and
/// then cross-contaminating) directories.
pub(crate) fn temp_subdir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = temp_dir().join(format!("sub-{}", COUNTER.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&dir).ok();
    dir
}
