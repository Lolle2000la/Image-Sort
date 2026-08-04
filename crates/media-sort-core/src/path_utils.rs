use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

/// Atomically replace `dest` with `bytes`: write to `tmp` (same directory,
/// so the rename stays on one filesystem), sync to disk, then rename over
/// `dest`. A crash mid-write can never truncate `dest`.
///
/// Callers are responsible for `tmp` being on the same filesystem as `dest`
/// and for any symlink resolution they need before the rename (the rename
/// replaces the path itself, it does not follow links).
pub fn atomic_write(tmp: &Path, dest: &Path, bytes: &[u8]) -> io::Result<()> {
    {
        let mut file = std::fs::File::create(tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(tmp, dest)
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
    fn test_paths_equal_one_canonicalize_fails() {
        let dir = temp_subdir();
        let existing = dir.join("exists.txt");
        std::fs::write(&existing, b"test").unwrap();
        let nonexistent = std::path::PathBuf::from("/nonexistent/other.txt");
        assert!(!paths_equal(&existing, &nonexistent));
    }
}
