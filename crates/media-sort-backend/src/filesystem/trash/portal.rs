//! Sending files to the *host* trash under Flatpak.
//!
//! Inside a Flatpak sandbox `$XDG_DATA_HOME` is remapped to a per-app
//! directory, so the `trash` crate's "home trash" (`$XDG_DATA_HOME/Trash`) ends
//! up in the sandbox's private data directory: the file never appears in the
//! user's trash, cannot be emptied from a file manager, and only survives until
//! the app is uninstalled with `--delete-data`. Opening a file read/write is
//! what the portal uses to authorise the request, so it is also what bounds
//! this path: for anything that cannot be opened read/write (a read-only mount,
//! for instance) the caller falls back to trashing inside the sandbox.
//!
//! The portal only *deletes*, so undoing a delete means finding the entry in
//! the host trash again. That lookup follows the FreeDesktop trash
//! specification: <https://specifications.freedesktop.org/trash-spec/trashspec-1.0.html>.

use std::fs;
use std::io;
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use media_sort_core::actions::delete_action::TrashRestoreHandle;
use media_sort_core::actions::reversible::ActionError;

/// Whether this process runs inside a Flatpak sandbox, where the host trash is
/// only reachable through the portal.
pub fn in_flatpak() -> bool {
    std::env::var_os("FLATPAK_ID").is_some_and(|id| !id.is_empty())
        || Path::new("/.flatpak-info").exists()
}

/// Ask the desktop to move `original_path` to the user's trash
/// (`org.freedesktop.portal.Trash.TrashFile`).
pub fn delete_to_trash(original_path: &Path) -> Result<Box<dyn TrashRestoreHandle>, ActionError> {
    // TrashFile takes a file descriptor, and the portal only trashes what the
    // caller can open read/write.
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(original_path)
        .map_err(ActionError::Io)?;

    // ashpd is async-only and the delete path is synchronous; a portal round
    // trip takes a few milliseconds, so drive it to completion here.
    pollster::block_on(ashpd::desktop::trash::trash_file(&file.as_fd())).map_err(to_io)?;

    Ok(Box::new(PortalTrashRestore {
        original_path: original_path.to_path_buf(),
        flushed: false,
    }))
}

fn to_io(err: impl std::error::Error + Send + Sync + 'static) -> ActionError {
    ActionError::Io(io::Error::other(err))
}

/// Restore handle for a file the host trashed on our behalf. The portal does
/// not report where the entry landed, so `restore` searches the host trash.
struct PortalTrashRestore {
    original_path: PathBuf,
    flushed: bool,
}

impl TrashRestoreHandle for PortalTrashRestore {
    fn restore(&mut self) -> Result<(), ActionError> {
        if self.flushed {
            return Err(ActionError::RestorationFailed("already flushed".into()));
        }

        restore_from_host_trash(&self.original_path)?;

        self.flushed = true;
        Ok(())
    }

    fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
        self.flushed = true;
        Ok(())
    }
}

impl Drop for PortalTrashRestore {
    fn drop(&mut self) {
        if !self.flushed {
            let _ = self.flush_to_native_trash();
        }
    }
}

/// Move a trashed entry back to its original path and drop its metadata.
fn restore_from_host_trash(original: &Path) -> Result<(), ActionError> {
    let Some((trashed_path, info_path)) = find_entry(original) else {
        return Err(ActionError::RestorationFailed(
            "the file is not in the host trash any more (it may have been emptied, or the host stores its trash elsewhere)"
                .into(),
        ));
    };

    if let Some(parent) = original.parent() {
        fs::create_dir_all(parent).map_err(ActionError::Io)?;
    }
    fs::rename(&trashed_path, original).map_err(ActionError::Io)?;
    // Removing the info file is what makes the entry disappear for the file
    // manager; a leftover would only be cosmetic, so its failure is not fatal.
    let _ = fs::remove_file(&info_path);

    Ok(())
}

/// The youngest trashed entry for `original`, as the file below the trash
/// directory's `files` directory and its `info` metadata file.
fn find_entry(original: &Path) -> Option<(PathBuf, PathBuf)> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    find_entry_in(&trash_directories(home.as_deref(), original), original)
}

fn find_entry_in(directories: &[PathBuf], original: &Path) -> Option<(PathBuf, PathBuf)> {
    let wanted = original.as_os_str().as_bytes();
    let mut youngest: Option<(String, PathBuf, PathBuf)> = None;

    for directory in directories {
        let Ok(entries) = fs::read_dir(directory.join("info")) else {
            continue;
        };
        for entry in entries.flatten() {
            let info_path = entry.path();
            if info_path.extension().and_then(|ext| ext.to_str()) != Some("trashinfo") {
                continue;
            }
            let Ok(contents) = fs::read_to_string(&info_path) else {
                continue;
            };
            let Some((trashed_from, deletion_date)) = parse_trash_info(&contents) else {
                continue;
            };
            if trashed_from != wanted {
                continue;
            }
            let Some(stem) = info_path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            // A name collision is resolved by renaming the entry in `files`,
            // and the info file carries that same name.
            let trashed_path = directory.join("files").join(stem);
            let is_younger = youngest
                .as_ref()
                .is_none_or(|(date, ..)| deletion_date > *date);
            if is_younger {
                youngest = Some((deletion_date, trashed_path, info_path));
            }
        }
    }

    youngest.map(|(_, trashed_path, info_path)| (trashed_path, info_path))
}

/// The trash directories that can hold `original`: the home trash, plus the
/// per-volume trash (`.Trash/<uid>`, or `.Trash-<uid>` where the shared one is
/// not usable) of every directory above it. Walking upwards finds them without
/// parsing the mount table, since they sit at a mount point.
fn trash_directories(home: Option<&Path>, original: &Path) -> Vec<PathBuf> {
    let mut directories = Vec::new();

    // The spec puts the home trash at `$XDG_DATA_HOME/Trash`. Inside the
    // sandbox that variable is the app-private data directory, so it is
    // resolved from the (real) home instead.
    if let Some(home) = home {
        directories.push(home.join(".local/share/Trash"));
    }

    let Some(uid) = home.and_then(user_id_of) else {
        return directories;
    };
    if let Some(parent) = original.parent() {
        for ancestor in parent.ancestors() {
            let shared = ancestor.join(".Trash").join(uid.to_string());
            if shared.is_dir() {
                directories.push(shared);
            }
            let private = ancestor.join(format!(".Trash-{uid}"));
            if private.is_dir() {
                directories.push(private);
            }
        }
    }

    directories
}

fn user_id_of(path: &Path) -> Option<u32> {
    fs::metadata(path).ok().map(|metadata| metadata.uid())
}

/// The `Path` (percent-encoded) and `DeletionDate` of a `.trashinfo` file.
fn parse_trash_info(contents: &str) -> Option<(Vec<u8>, String)> {
    let mut trashed_from = None;
    let mut deletion_date = None;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("Path=") {
            trashed_from = Some(percent_decode(value.trim()));
        } else if let Some(value) = line.strip_prefix("DeletionDate=") {
            deletion_date = Some(value.trim().to_owned());
        }
    }

    Some((trashed_from?, deletion_date?))
}

/// Decode the percent-encoding the trash specification uses for paths.
fn percent_decode(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = (bytes[index + 1] as char).to_digit(16);
            let low = (bytes[index + 2] as char).to_digit(16);
            if let (Some(high), Some(low)) = (high, low) {
                decoded.push((high * 16 + low) as u8);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that cleans up after itself.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "media-sort-trash-portal-{name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_trashinfo(trash: &Path, stem: &str, original: &Path, date: &str) {
        fs::create_dir_all(trash.join("info")).expect("create info dir");
        let encoded = original
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|byte| {
                if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'-' | b'_') {
                    (*byte as char).to_string()
                } else {
                    format!("%{byte:02X}")
                }
            })
            .collect::<String>();
        fs::write(
            trash.join("info").join(format!("{stem}.trashinfo")),
            format!("[Trash Info]\nPath={encoded}\nDeletionDate={date}\n"),
        )
        .expect("write trashinfo");
    }

    #[test]
    fn percent_decoding_handles_escapes_and_plain_bytes() {
        assert_eq!(percent_decode("/home/a%20b"), b"/home/a b");
        assert_eq!(percent_decode("%C3%A4.jpg"), "ä.jpg".as_bytes());
        // A truncated or malformed escape is left alone rather than dropping
        // bytes.
        assert_eq!(percent_decode("100%"), b"100%");
        assert_eq!(percent_decode("%zz"), b"%zz");
    }

    #[test]
    fn trashinfo_parsing_reads_path_and_date() {
        let parsed = parse_trash_info(
            "[Trash Info]\nPath=/tmp/a%20b.jpg\nDeletionDate=2026-10-16T01:02:03\n",
        );
        assert_eq!(
            parsed,
            Some((b"/tmp/a b.jpg".to_vec(), "2026-10-16T01:02:03".to_owned()))
        );
        assert_eq!(parse_trash_info("[Trash Info]\nPath=/tmp/a.jpg\n"), None);
        assert_eq!(parse_trash_info("garbage"), None);
    }

    #[test]
    fn finds_the_youngest_entry_for_the_path() {
        let temp = TempDir::new("youngest");
        let trash = temp.path().join("Trash");
        let original = temp.path().join("photo.jpg");

        write_trashinfo(
            &trash,
            "other.jpg",
            &temp.path().join("other.jpg"),
            "2026-10-16T05:00:00",
        );
        write_trashinfo(&trash, "photo.jpg", &original, "2026-10-16T03:00:00");
        // A later delete of a file that had the same name.
        write_trashinfo(&trash, "photo.jpg.2", &original, "2026-10-16T04:00:00");

        let (trashed_path, info_path) =
            find_entry_in(std::slice::from_ref(&trash), &original).expect("entry found");

        assert_eq!(trashed_path, trash.join("files/photo.jpg.2"));
        assert_eq!(info_path, trash.join("info/photo.jpg.2.trashinfo"));
    }

    #[test]
    fn ignores_entries_for_other_paths() {
        let temp = TempDir::new("other-paths");
        let trash = temp.path().join("Trash");
        write_trashinfo(
            &trash,
            "other.jpg",
            &temp.path().join("other.jpg"),
            "2026-10-16T05:00:00",
        );

        assert!(find_entry_in(&[trash], &temp.path().join("photo.jpg")).is_none());
    }

    #[test]
    fn trash_directories_cover_home_and_a_volume() {
        let temp = TempDir::new("directories");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create home");
        let uid = user_id_of(&home).expect("uid of the temp home");

        let volume = temp.path().join("media/stick");
        let original = volume.join("DCIM/photo.jpg");
        fs::create_dir_all(&original).expect("create fixture");
        fs::create_dir_all(volume.join(format!(".Trash-{uid}"))).expect("create volume trash");

        let directories = trash_directories(Some(&home), &original);

        assert!(directories.contains(&home.join(".local/share/Trash")));
        assert!(directories.contains(&volume.join(format!(".Trash-{uid}"))));
        // Directories below the volume root are not trash locations.
        assert!(!directories.contains(&volume.join("DCIM").join(format!(".Trash-{uid}"))));
    }
}
