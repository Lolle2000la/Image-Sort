//! Portal-backed storage (Linux).
//!
//! The app's default "storage backend" is the direct filesystem: every path it
//! is handed is opened with `std::fs`. Under Flatpak that needs a broad
//! `--filesystem=home` grant (plus the removable-media mounts), which is by far
//! the largest permission Media Sort asks for and the one Flathub's linter
//! flags (`finish-args-home-filesystem-access`).
//!
//! This module is the alternative backend: reach files *only* through the XDG
//! Desktop Portals, so a sandbox can be built without any `--filesystem=`
//! argument. The user picks a location with the file chooser, the desktop
//! exports it into the *document portal*, and the app then sees it under the
//! portal's FUSE mount (`/run/user/$UID/doc/<doc_id>/…`). Within a picked
//! folder, ordinary path APIs keep working: recursive listing, reading,
//! thumbnails/previews/metadata, and create/rename/delete.
//!
//! What a portal-only backend can **not** do is why the app does not switch to
//! it yet (the full analysis lives in `flatpak/portal-storage-feasibility.md`):
//!
//! - **The OS watcher goes blind.** inotify on the document-portal FUSE mount
//!   only reports operations performed *through that mount*, never changes a
//!   host application makes to the backing file
//!   (<https://github.com/flatpak/xdg-desktop-portal/issues/567>). External
//!   adds/renames/deletes would be missed until a manual refresh.
//! - **A grant is a snapshot, not a bookmark.** Document paths are not
//!   guaranteed to resolve again after a reboot
//!   (<https://github.com/flatpak/xdg-desktop-portal/issues/1362>), and inside
//!   the sandbox `Documents.List`/`Info`/`Lookup` are host-only, so the app can
//!   only remember the paths it was handed itself.
//! - **No arbitrary navigation.** Only picked locations (and their subtrees)
//!   exist; there is no home/drive enumeration and `..` above a picked root is
//!   not readable.
//! - **Cross-grant moves degrade.** `rename(2)` between two document domains
//!   returns `EXDEV`, so moving into a *different* picked folder becomes
//!   copy+delete (non-atomic, is not covered by the existing
//!   `rename_or_copy_and_delete` error handling for free).
//!
//! What is safe to use today, next to direct access, is here: the sandbox check
//! and the OpenURI actions, which are the correct way to hand a file to a host
//! application from inside the sandbox (`xdg-open`/`dbus-send` reach the host
//! only by accident and cannot see document paths).

pub mod document_portal;

pub use document_portal::{Grant, PortalError};

use std::path::Path;

/// Whether this process runs inside a Flatpak sandbox, where the host's files
/// are reachable only through the desktop portals.
pub fn in_flatpak() -> bool {
    std::env::var_os("FLATPAK_ID").is_some_and(|id| !id.is_empty())
        || Path::new("/.flatpak-info").exists()
}

/// Hand `path` to the user's default application through
/// `org.freedesktop.portal.OpenURI`.
///
/// Returns [`PortalError::Unavailable`] when the process is not sandboxed or
/// `path` is not an existing file (a URL, for instance), so the caller keeps
/// its native `open::that` path. Non-availability and failures are logged at
/// debug/warn respectively and surfaced so the caller can fall back.
pub fn open_externally(path: &Path) -> Result<(), PortalError> {
    // `open_externally` is also called with web URLs; OpenURI.open_file takes a
    // file descriptor, so anything that is not an existing file is left to the
    // caller's `xdg-open`/`open::that` handling.
    if !in_flatpak() || !path.exists() {
        return Err(PortalError::Unavailable);
    }
    match pollster::block_on(document_portal::open_file(path)) {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::warn!(%err, path = %path.display(), "OpenURI.open_file failed");
            Err(err)
        }
    }
}

/// Show `path`'s containing folder in the file manager through
/// `org.freedesktop.portal.OpenURI`.
///
/// `OpenURI` has no "select this file" call, so this opens the folder rather
/// than highlighting the entry. Returns [`PortalError::Unavailable`] outside a
/// sandbox.
pub fn reveal_in_file_manager(path: &Path) -> Result<(), PortalError> {
    if !in_flatpak() {
        return Err(PortalError::Unavailable);
    }
    match pollster::block_on(document_portal::reveal_folder(path)) {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::warn!(%err, path = %path.display(), "OpenURI.open_directory failed");
            Err(err)
        }
    }
}
