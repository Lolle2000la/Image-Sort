use std::io;
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

pub fn cross_device_error(e: &io::Error) -> bool {
    e.raw_os_error() == Some(18)
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

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("media-sort-test-{}", std::process::id()));
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

    fn temp_subdir() -> std::path::PathBuf {
        let dir = temp_dir().join(format!("sub-{}", rand_u32()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }

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
        let err = std::io::Error::from_raw_os_error(18);
        assert!(cross_device_error(&err));
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
