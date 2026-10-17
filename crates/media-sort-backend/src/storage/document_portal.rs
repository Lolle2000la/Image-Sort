//! The document-portal half of the portal storage backend.
//!
//! `org.freedesktop.portal.Documents` exports a file or directory the app can
//! already open into the *document store*, which the desktop then exposes to
//! the app under a FUSE mount (`/run/user/$UID/doc/<doc_id>/…`). An exported
//! directory is recursive: its whole subtree becomes readable and writable.
//!
//! These are the building blocks of a "granted locations" model — the list of
//! folders the user has handed to the app — rather than a general-purpose file
//! API. See the module docs on [`crate::storage`] for why the app does not run
//! on this alone.

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::os::fd::AsFd;
use std::path::{Component, Path, PathBuf};

use ashpd::desktop::open_uri::{OpenDirectoryRequest, OpenFileRequest};
use ashpd::documents::{DocumentFlags, DocumentID, Documents};

/// Failure of a document-portal request.
#[derive(Debug, thiserror::Error)]
pub enum PortalError {
    /// No portal is reachable (not sandboxed, or the desktop has none).
    #[error("the desktop portal is not available")]
    Unavailable,
    /// A portal request failed.
    #[error(transparent)]
    Portal(#[from] ashpd::Error),
    /// The path could not be opened. The portal authorises on an open file
    /// descriptor, so an unreadable path cannot be exported.
    #[error("failed to open {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The portal accepted the request but returned no document.
    #[error("the document portal returned no document id")]
    EmptyResponse,
}

/// A location the user handed to the app through the document portal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    /// The document-store id, the first component below the mount point.
    pub doc_id: String,
    /// The path the app works with, inside the portal's FUSE mount.
    pub path: PathBuf,
    /// The real location, for display only. Known when the grant was created
    /// by this app ([`grant`]); `None` for ids resolved from elsewhere.
    pub host_path: Option<PathBuf>,
}

/// The document portal's FUSE mount point, typically `/run/user/$UID/doc`.
pub async fn mount_point() -> Result<PathBuf, PortalError> {
    let documents = Documents::new().await?;
    Ok(path_buf(&documents.mount_point().await?))
}

/// Export `path` into the document store and return the resulting grant.
///
/// `path` must already be openable by this process: the portal treats an open
/// descriptor as proof of access, so this cannot reach anything the app could
/// not open anyway. Files go through `Add`; directories go through `AddFull`
/// with the `export-directory` flag (interface version 4), which makes the
/// whole subtree accessible and is what the file chooser does for a picked
/// folder. The entry is persisted, so the returned document path is meant to be
/// recorded and resolved again on later launches.
pub async fn grant(path: &Path) -> Result<Grant, PortalError> {
    let file = File::open(path).map_err(|source| PortalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let documents = Documents::new().await?;

    let doc_id = if path.is_dir() {
        let flags = DocumentFlags::ReuseExisting
            | DocumentFlags::Persistent
            | DocumentFlags::ExportDirectory;
        let (ids, _extra) = documents
            .add_full(&[file.as_fd()], flags, None, &[])
            .await?;
        ids.into_iter().next().ok_or(PortalError::EmptyResponse)?
    } else {
        documents.add(&file.as_fd(), true, true).await?
    };

    let doc_id = String::from(doc_id);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| doc_id.clone());
    let mount = path_buf(&documents.mount_point().await?);

    Ok(Grant {
        path: document_path(&mount, &doc_id, &name),
        doc_id,
        // The caller supplied the host path; no `GetHostPaths` round trip
        // (which also requires portal version 5) is needed.
        host_path: Some(path.to_path_buf()),
    })
}

/// The host paths for document ids the app only knows as ids
/// (`Documents.GetHostPaths`, interface version 5). Best effort: ids the portal
/// cannot resolve are omitted from the result.
pub async fn host_paths(doc_ids: &[String]) -> Result<HashMap<String, PathBuf>, PortalError> {
    let documents = Documents::new().await?;
    let ids: Vec<DocumentID> = doc_ids.iter().cloned().map(DocumentID::from).collect();
    let paths = documents.host_paths(&ids).await?;
    Ok(paths
        .into_iter()
        .map(|(id, path)| (String::from(id), path_buf(&path)))
        .collect())
}

/// Open a file with the user's default application (`OpenURI.OpenFile`).
pub async fn open_file(path: &Path) -> Result<(), PortalError> {
    let file = File::open(path).map_err(|source| PortalError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    OpenFileRequest::default().send_file(&file.as_fd()).await?;
    Ok(())
}

/// Ask the file manager to show `path`'s containing folder
/// (`OpenURI.OpenDirectory`).
///
/// `OpenURI` only takes a *directory* descriptor, so a file is revealed by
/// opening the folder it lives in rather than selecting the entry.
pub async fn reveal_folder(path: &Path) -> Result<(), PortalError> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(path);
    let handle = File::open(directory).map_err(|source| PortalError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    OpenDirectoryRequest::default()
        .send(&handle.as_fd())
        .await?;
    Ok(())
}

/// Whether `path` lies inside the document portal mount `mount`.
pub fn is_document_path(mount: &Path, path: &Path) -> bool {
    path.starts_with(mount)
}

/// The document id of `path`, if it lies inside the mount `mount`.
pub fn document_id_in(mount: &Path, path: &Path) -> Option<String> {
    let mut components = path.strip_prefix(mount).ok()?.components();
    match components.next()? {
        Component::Normal(id) => Some(id.to_str()?.to_owned()),
        _ => None,
    }
}

/// The path a document is exposed at inside the mount:
/// `<mount>/<doc_id>/<name>`.
pub fn document_path(mount: &Path, doc_id: &str, name: &str) -> PathBuf {
    mount.join(doc_id).join(name)
}

/// `ashpd::FilePath` is a nul-terminated byte path; convert it to `PathBuf`.
fn path_buf(path: &ashpd::FilePath) -> PathBuf {
    let path: &Path = path.as_ref();
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_paths_are_built_and_taken_apart() {
        let mount = Path::new("/run/user/1000/doc");
        let doc = document_path(mount, "a1b2c3d4", "Holiday");

        assert_eq!(doc, Path::new("/run/user/1000/doc/a1b2c3d4/Holiday"));
        assert!(is_document_path(mount, &doc));
        assert_eq!(document_id_in(mount, &doc).as_deref(), Some("a1b2c3d4"));

        // A nested path still reports the document, not the file.
        let nested = doc.join("2024/img.png");
        assert_eq!(document_id_in(mount, &nested).as_deref(), Some("a1b2c3d4"));

        // The bare document root has no file component but is valid.
        assert_eq!(
            document_id_in(mount, Path::new("/run/user/1000/doc/a1b2c3d4")).as_deref(),
            Some("a1b2c3d4")
        );
    }

    #[test]
    fn paths_outside_the_mount_are_not_documents() {
        let mount = Path::new("/run/user/1000/doc");

        for path in [
            "/home/user/Pictures",
            "/run/user/1000/docx/a1b2c3d4",
            "/run/user/1000",
        ] {
            let path = Path::new(path);
            assert!(!is_document_path(mount, path), "{path:?}");
            assert_eq!(document_id_in(mount, path), None, "{path:?}");
        }

        // The mount point itself is not a document either.
        assert_eq!(document_id_in(mount, mount), None);
    }

    #[test]
    fn a_document_id_component_must_be_named() {
        let mount = Path::new("/run/user/1000/doc");
        // `..` is a parent component, not a document name.
        assert_eq!(
            document_id_in(mount, Path::new("/run/user/1000/doc/..")),
            None
        );
    }
}
