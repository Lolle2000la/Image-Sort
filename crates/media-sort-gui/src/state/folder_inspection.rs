use std::path::{Path, PathBuf};

use media_sort_core::models::FolderKind;

/// Everything the folder tree needs to know about a directory, gathered in
/// one pass: the [`FolderKind`] for icon selection, the resolved target of a
/// symlink (for display), and whether it has subdirectories (to decide the
/// expand chevron).
pub(crate) struct FolderInspection {
    pub(crate) kind: FolderKind,
    pub(crate) symlink_target: Option<PathBuf>,
    pub(crate) has_child_dir: bool,
}

pub(crate) fn inspect_folder(path: &Path) -> FolderInspection {
    let is_symlink = media_sort_core::path_utils::is_symlink(path);

    // The full target path is shown next to symlink nodes. canonicalize()
    // resolves the final destination; read_link() is the fallback for
    // dangling links (which have no canonical path).
    let symlink_target = if is_symlink {
        path.canonicalize()
            .ok()
            .or_else(|| std::fs::read_link(path).ok())
    } else {
        None
    };

    // Listing determines both `has_child_dir` and whether the contents can
    // be read. read_dir follows symlinks, so this reads the link target.
    let (has_child_dir, listable) = match std::fs::read_dir(path) {
        Ok(entries) => {
            let has_child_dir = entries.flatten().any(|entry| {
                entry
                    .file_type()
                    .is_ok_and(|ft| ft.is_dir() || (ft.is_symlink() && entry.path().is_dir()))
            });
            (has_child_dir, true)
        }
        Err(e) => (false, e.kind() != std::io::ErrorKind::PermissionDenied),
    };

    let writable = std::fs::metadata(path)
        .map(|m| is_writable_metadata(&m))
        .unwrap_or(true);

    let contains_git = path.join(".git").exists();

    // Priority: Locked > Symlink > Git > Default. Git is intentionally the
    // lowest-priority classification, so a git repo that is also a symlink
    // (or restricted) shows the more meaningful icon.
    let kind = if !listable || !writable {
        FolderKind::Locked
    } else if is_symlink {
        FolderKind::Symlink
    } else if contains_git {
        FolderKind::Git
    } else {
        FolderKind::Default
    };

    FolderInspection {
        kind,
        symlink_target,
        has_child_dir,
    }
}

/// Whether the folder is writable: on unix any write bit (owner/group/other)
/// counts, on Windows the read-only attribute decides. Follows symlinks, so
/// a symlinked folder is judged by its target.
fn is_writable_metadata(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o222 != 0
    }
    #[cfg(windows)]
    {
        !metadata.permissions().readonly()
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = metadata;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_folder_git_kind() {
        let dir = std::env::temp_dir().join(format!("mediasort_git_kind_{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".git")).unwrap();

        let inspection = inspect_folder(&dir);
        assert_eq!(inspection.kind, FolderKind::Git);
        assert!(inspection.symlink_target.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_inspect_folder_plain_kind() {
        let dir = std::env::temp_dir().join(format!("mediasort_plain_kind_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let inspection = inspect_folder(&dir);
        assert_eq!(inspection.kind, FolderKind::Default);
        assert!(inspection.symlink_target.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_inspect_folder_read_only_is_locked() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("mediasort_locked_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        let inspection = inspect_folder(&dir);
        assert_eq!(inspection.kind, FolderKind::Locked);

        // Restore writability so the cleanup can remove the directory.
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_inspect_folder_locked_wins_over_git() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("mediasort_locked_git_{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        let inspection = inspect_folder(&dir);
        assert_eq!(inspection.kind, FolderKind::Locked);

        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_inspect_folder_symlink_kind_and_target() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_symlink_kind_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let real = dir.join("real");
        std::fs::create_dir(&real).unwrap();
        let link = dir.join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let inspection = inspect_folder(&link);
        assert_eq!(inspection.kind, FolderKind::Symlink);
        assert_eq!(
            inspection.symlink_target,
            Some(real.canonicalize().unwrap())
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_inspect_folder_symlink_wins_over_git() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_symlink_git_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let repo = dir.join("repo");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        let link = dir.join("link");
        std::os::unix::fs::symlink(&repo, &link).unwrap();

        let inspection = inspect_folder(&link);
        assert_eq!(inspection.kind, FolderKind::Symlink);

        std::fs::remove_dir_all(&dir).ok();
    }
}
