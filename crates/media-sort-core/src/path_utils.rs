use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// Process-global counter for [`unique_temp_path`] names.
static TEMP_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn rename_or_copy_and_delete(src: &Path, dst: &Path) -> io::Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if cross_device_error(&e) => {
            std::fs::copy(src, dst)?;
            std::fs::remove_file(src)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Whether `e` is a cross-device (EXDEV) error, i.e. a rename that failed
/// only because source and destination live on different filesystems.
///
/// On unix this is errno 18 (EXDEV on Linux and the BSDs alike). On Windows
/// `std::fs::rename` already passes `MOVEFILE_COPY_ALLOWED`, so a
/// cross-volume rename never produces an error and no fallback is needed.
pub fn cross_device_error(e: &io::Error) -> bool {
    #[cfg(unix)]
    {
        e.raw_os_error() == Some(18)
    }
    #[cfg(not(unix))]
    {
        let _ = e;
        false
    }
}

/// Whether `path` is a symbolic link, without following it. Non-existent
/// paths and metadata errors are treated as `false`.
///
/// Single source of truth for the symlink-refusal policy shared by the
/// actions layer (`reversible::reject_symlink_source`), drag & drop, the
/// Windows trash implementation and the settings store.
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Unique temp path in the destination directory for a file that will be
/// renamed onto `final_path`: `{stem}.{label}.{pid}.{n}`. The pid +
/// process-global counter keep concurrent writers (two app instances, or
/// a future second thread) from racing on a shared temp name: with a
/// fixed temp name, one writer's `File::create` truncates the file another
/// writer is mid-fsync on, and the interleaved rename can land a
/// partial/corrupt config. With unique names only last-writer-wins
/// applies, which is the benign outcome. Stale temps on a crash are
/// harmless leftovers and are never picked up as staged content.
pub fn unique_temp_path(final_path: &Path, label: &str) -> PathBuf {
    let stem = final_path.file_stem().unwrap_or_default().to_string_lossy();
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    final_path.with_file_name(format!("{stem}.{label}.{}.{n}", std::process::id()))
}

/// Atomically replace `dest` with `bytes`: write to a unique temp file in
/// the same directory (so the rename stays on one filesystem), sync to
/// disk, then rename over `dest`. A crash mid-write can never truncate
/// `dest`, and the unique temp name (see [`unique_temp_path`]) keeps two
/// concurrent writers from clobbering each other's in-flight file. If the
/// rename fails, the temp file is removed so failures do not litter the
/// directory with `*.tmp.<pid>.<n>` files.
///
/// Callers are responsible for any symlink resolution they need before the
/// rename (the rename replaces the path itself, it does not follow links).
pub fn atomic_write(dest: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = unique_temp_path(dest, "tmp");
    let result = (|| {
        {
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, dest)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

pub fn paths_equal(a: &Path, b: &Path) -> bool {
    a == b
        || a.canonicalize()
            .ok()
            .zip(b.canonicalize().ok())
            .is_some_and(|(ca, cb)| ca == cb)
}

pub fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::test_utils::temp_subdir;

    #[test]
    fn test_paths_equal_same() {
        let dir = temp_subdir();
        let file = dir.join("equal_test.txt");
        std::fs::write(&file, b"data").unwrap();

        assert!(paths_equal(&file, &file));
    }

    #[test]
    fn test_paths_equal_different() {
        let dir = temp_subdir();
        let file1 = dir.join("diff_a.txt");
        let file2 = dir.join("diff_b.txt");
        std::fs::write(&file1, b"a").unwrap();
        std::fs::write(&file2, b"b").unwrap();

        assert!(!paths_equal(&file1, &file2));
    }

    #[test]
    fn test_paths_equal_relative_vs_absolute() {
        let dir = temp_subdir();
        let file = dir.join("rel_test.txt");
        std::fs::write(&file, b"data").unwrap();

        let canonical = file.canonicalize().unwrap();

        assert!(paths_equal(&canonical, &file));
    }

    #[test]
    fn test_paths_equal_non_existent() {
        let a = std::path::PathBuf::from("/nonexistent/a.txt");
        let b = std::path::PathBuf::from("/nonexistent/b.txt");
        assert!(!paths_equal(&a, &b));
    }

    #[test]
    fn test_normalize_path() {
        let dir = temp_subdir();
        let file = dir.join("normalize_test.txt");
        std::fs::write(&file, b"data").unwrap();

        let sub_dir = dir.join("inner");
        std::fs::create_dir_all(&sub_dir).unwrap();
        let non_canonical = sub_dir.join("../normalize_test.txt");

        let normalized = normalize_path(&non_canonical);
        let expected = file.canonicalize().unwrap();
        assert_eq!(normalized, expected);
    }

    #[test]
    fn test_normalize_path_non_existent() {
        let dir = temp_subdir();
        let missing = dir.join("subdir").join("nonexistent.txt");
        let result = normalize_path(&missing);
        assert_eq!(result, missing);
    }

    #[test]
    fn test_rename_or_copy_and_delete_same_device() {
        let dir = temp_subdir();
        let src = dir.join("rename_test_src.txt");
        let dst = dir.join("rename_test_dst.txt");
        std::fs::write(&src, b"same device rename").unwrap();

        rename_or_copy_and_delete(&src, &dst).unwrap();
        assert!(!src.exists());
        assert!(dst.exists());
        let contents = std::fs::read_to_string(&dst).unwrap();
        assert_eq!(contents, "same device rename");
    }

    #[test]
    fn test_rename_or_copy_and_delete_source_not_found() {
        let dir = temp_subdir();
        let src = dir.join("nonexistent_src_xyz.txt");
        let dst = dir.join("nonexistent_dst_xyz.txt");
        let result = rename_or_copy_and_delete(&src, &dst);
        assert!(result.is_err());
    }

    #[test]
    fn test_cross_device_error_linux_exdev() {
        // EXDEV (18) is the unix cross-filesystem rename error; on Windows
        // std::fs::rename already passes MOVEFILE_COPY_ALLOWED so a
        // cross-volume rename never errors, and raw error 18 means something
        // else entirely — the fallback must stay off there.
        let err = std::io::Error::from_raw_os_error(18);
        #[cfg(unix)]
        assert!(cross_device_error(&err));
        #[cfg(not(unix))]
        assert!(!cross_device_error(&err));
    }

    #[test]
    fn test_cross_device_error_other() {
        let err = std::io::Error::from_raw_os_error(2);
        assert!(!cross_device_error(&err));
    }

    #[test]
    fn test_cross_device_error_permission_denied() {
        let err = std::io::Error::from_raw_os_error(13);
        assert!(!cross_device_error(&err));
    }

    #[test]
    fn test_atomic_write_replaces_dest() {
        let dir = temp_subdir();
        let dest = dir.join("config.toml");
        std::fs::write(&dest, b"old").unwrap();

        atomic_write(&dest, b"new").unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), b"new");
        // No temp files must be left behind after a successful write.
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "leftover temps: {leftovers:?}");
    }

    #[test]
    fn test_atomic_write_removes_temp_on_rename_failure() {
        // A destination DIRECTORY makes the final rename fail (EISDIR on
        // unix): the temp file must be removed and the destination left
        // untouched, so a failed save does not litter `*.tmp.<pid>.<n>`
        // files next to the config.
        let dir = temp_subdir();
        let dest = dir.join("config.toml");
        std::fs::create_dir(&dest).unwrap();

        let result = atomic_write(&dest, b"data");

        assert!(result.is_err());
        assert!(dest.is_dir(), "destination must be untouched");
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "leftover temps: {leftovers:?}");
    }

    #[test]
    fn test_paths_equal_one_canonicalize_fails() {
        let dir = temp_subdir();
        let existing = dir.join("exists.txt");
        std::fs::write(&existing, b"test").unwrap();
        let nonexistent = std::path::PathBuf::from("/nonexistent/other.txt");
        assert!(!paths_equal(&existing, &nonexistent));
    }
}
