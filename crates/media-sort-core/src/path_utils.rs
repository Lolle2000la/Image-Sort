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

fn cross_device_error(e: &io::Error) -> bool {
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
