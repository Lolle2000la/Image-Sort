# Portal-only storage: feasibility

Can Media Sort get its files entirely through the XDG Desktop Portals, so the
Flatpak stops asking for `--filesystem=home` (and `/media`, `/mnt`,
`/run/media`)?

**Short answer: not entirely, and not yet.** The portal can supply *scoped,
user-approved access to a picked folder* — reading, thumbnails, previews,
metadata, and create/rename/delete inside it all work through ordinary path
APIs on the document-portal FUSE mount. What it cannot supply are four things
the app leans on: **external-change monitoring**, **arbitrary folder-tree
navigation**, **cross-folder move/rename**, and **reliable re-access of a
granted folder across reboots**. Two of those (the watcher and navigation) are
core features, so the app cannot ship portal-only today.

This document records the evidence and the decision. The pieces that *are*
safe next to direct access have been implemented in
`crates/media-sort-backend/src/storage/` (see “What was built”).

## How the model works

A file or folder the user picks is *exported* into the **document store**, and
the desktop exposes it to the sandbox on a FUSE mount
(`/run/user/$UID/doc/<doc_id>/…`). Inside a picked folder the whole subtree is
reachable, and normal filesystem calls work: `readdir`, `open`/`read`,
`create`, `unlink`, `mkdir`, `rename`. The POSIX mode bits on the mount reflect
the permissions the app holds.

| Capability | Portal | Method | Result |
|---|---|---|---|
| Pick folder(s) | `org.freedesktop.portal.FileChooser` | `OpenFile` with `directory=true`, `multiple` | document-portal path(s); recursive for a folder |
| FUSE mount base | `org.freedesktop.portal.Documents` | `GetMountPoint` | typically `/run/user/$UID/doc` |
| Export a path the app can already open | `Documents` | `Add` (files) / `AddFull` + `export-directory` (folders, v4) | doc id; `persistent` flag keeps the entry |
| Real path for display | `Documents` | `GetHostPaths` (v5) | host path, display-only |
| Trash | `org.freedesktop.portal.Trash` | `TrashFile(fd)` | host trash (already used on this branch) |
| Open file / reveal folder | `org.freedesktop.portal.OpenURI` | `OpenFile(fd)` / `OpenDirectory(fd)` | host app opens it; **fd-based, not path-based** |
| Receive dropped files | `org.freedesktop.portal.FileTransfer` | `StartTransfer`/`AddFiles`/`RetrieveFiles` | needs the *source* app to cooperate |

`Documents.List` / `Info` / `Lookup` are **host-only** — a sandboxed app cannot
enumerate its own grants; it can only remember the paths it was handed. The
`xdg-document-portal` FUSE implementation also has no `mknod`, and its
attribute/entry cache timeouts are `0`, so every `lookup`/`open` is a userspace
round trip (a large library scans noticeably slower than direct I/O).

## What breaks

1. **Filesystem watcher — dead.** inotify on the FUSE mount fires for
   operations performed *through the mount* (including the app's own writes),
   but never for changes a host process makes to the backing file. The app's
   whole watcher exists to notice *external* changes, so this feature cannot be
   implemented portal-only. Kernel/libfuse limitation, not a portal bug:
   <https://github.com/flatpak/xdg-desktop-portal/issues/567>,
   <https://github.com/libfuse/libfuse/issues/328>.

2. **Move/rename between two picked folders — `EXDEV`.** Each picked location
   is its own document *domain*; `xdp_fuse_rename` refuses cross-domain renames.
   Sorting into a different picked folder becomes copy+delete (non-atomic,
   doubles I/O, and the rollback story needs rework). Renames *within* one
   granted folder are fine.

3. **A singly-picked *file* cannot be renamed.** For non-directory documents the
   FUSE layer only allows the tempfile/atomic-save idiom; arbitrary renames
   return `EACCES`/`ENOENT`. Works only if the user granted the *containing
   folder*. Likewise `mkdir`/`unlink`/`rmdir` require a directory document.

4. **Grants are not durable bookmarks.** Stored document paths frequently fail
   to resolve again after a reboot (identity was keyed on `st_dev`+`st_ino`;
   see <https://github.com/flatpak/xdg-desktop-portal/issues/1362> and
   <https://github.com/flatpak/xdg-desktop-portal/issues/1845>), and the app
   cannot re-enumerate its grants from inside the sandbox. Expect periodic
   "please pick the folder again" prompts.

5. **No arbitrary navigation.** Only picked locations and their subtrees exist.
   There is no home/drive enumeration, and `..` above a picked root is not
   readable. The current folder tree (`build_parent_chain` walking to `/`,
   `build_children` reading any directory) would have to be re-rooted on the
   set of grants.

6. **Paths become opaque.** Users see `/run/user/1000/doc/<id>/…`, not their
   real paths; `canonicalize()` on a doc path either fails or resolves to a host
   path the sandbox cannot use, which interacts badly with the app's
   path-as-identity assumptions (caches, selection restore, undo/redo, the
   watcher subscription key, mpv frame matching).

## Precedent

Apps that stay portal-only do it for *content* work, not library browsing:
Metadata Cleaner has no `--filesystem` at all and processes picked files and
folders via the document portal. Apps that browse/watch libraries take the
broad grant instead — Loupe (`--filesystem=host`), GNOME Text Editor
(`--filesystem=host`), Curtail (`home`), Amberol (`xdg-music:ro`). Upstream's
own answer to "file-manager-like access to specific trees" is the FileChooser
with `directory=true`, and it calls a dynamic per-path filesystem grant
"technically not possible" without a new FUSE overlay layer
(<https://github.com/flatpak/xdg-desktop-portal/issues/663>).

## What was built

`crates/media-sort-backend/src/storage/` is a storage module that lives next to
the direct backend (Linux-only):

- `document_portal.rs` — the document-portal backend: `mount_point`, `grant`
  (export a folder/file and return a `Grant { doc_id, doc_path, host_path }`),
  `host_paths`, `open_file`, `reveal_folder`, and the pure path helpers
  (`is_document_path`, `document_id_in`, `document_path`) with unit tests.
- `mod.rs` — the sandbox check (`in_flatpak`, now shared with the trash portal)
  and the two fire-and-forget OpenURI actions.

Wired where it is safe and strictly better under a sandbox:

- **Open externally** (`update/tasks.rs`) uses `OpenURI.OpenFile` when inside
  Flatpak, falling back to `open::that` otherwise.
- **Reveal in file manager** uses `OpenURI.OpenDirectory` inside Flatpak
  (opens the containing folder — OpenURI cannot select a file), falling back to
  the existing `dbus-send`/`xdg-open` path.
- The trash portal now shares the single `in_flatpak()` predicate.

These need no `--filesystem` change and make the sandbox more correct, but they
do **not** by themselves let the manifest drop `--filesystem=home`.

## Path to a full portal-only mode (not done)

If the four blockers are acceptable (or the watcher is disabled with a clear
status message), the remaining work is:

1. **Granted-locations model**: persist `Grant`s (doc id + intended host path);
   re-resolve on launch, prompting for the ones that no longer resolve.
2. **Re-root the tree** on grants: treat each grant like a pinned root, stop
   `build_parent_chain` at the grant, drop the unconditional
   `current_folder.parent()` watch entry.
3. **Watcher**: disable under portal mode (or accept self-write-only events).
4. **Moves**: route cross-grant moves through copy+delete; keep same-folder
   renames as `rename(2)`.
5. **Path identity**: key caches/selection/history on `(doc_id, relative path)`
   rather than the volatile mount path.
6. **Manifest**: drop `--filesystem=home`, `/media`, `/mnt`, `/run/media`; keep
   `--share=ipc`, the sockets, and `--device=dri`.

Even then the UX is "pick a folder, then work inside it", not "browse your
computer", and the watcher gap is permanent until a FUSE notification mechanism
exists upstream.

## Verification status

The code compiles, is clippy-clean and formatted, and the pure helpers are
unit-tested. The D-Bus calls (`grant`, `host_paths`, `open_file`,
`reveal_folder`) **cannot be exercised here** — there is no running desktop
portal in this environment. They should be smoke-tested on a real Flatpak
build (and on both the GNOME and KDE portal backends, which differ in the
FileChooser and OpenURI implementations) before being relied on.
