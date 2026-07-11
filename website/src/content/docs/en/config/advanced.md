---
title: Advanced Settings
description: Details on advanced configuration options, file deletion profiles, and folder behavior in Media Sort.
---

This page documents the advanced configuration properties and architectural designs of folder operations and file deletion profiles.

## Folder Behavior Profiles

Media Sort allows you to fine-tune how it interacts with the local directories you open:

### 1. Last Opened Folder
By default, the application remembers your last workspace and restores it on the next startup.
- **Config Key:** `general.reopen_last_opened_folder` (boolean)
- **Behavior:** If `true`, the app will save the path in `general.last_opened_folder` and automatically re-open it on launch. Set to `false` if you want the app to always start with the folder selector dialog.

### 2. Folder Tree Layout
- **Config Key:** `general.folder_tree_width` (integer)
- **Behavior:** Defines the default width of the left-hand folder navigation panel in pixels.

---

## File Deletion & System Trash

A core design principle of Media Sort is **Zero-Anxiety Sorting**, which relies on non-destructive deletions to support instant **Undo / Redo**.

### How Deletions Work

When you delete a file (by pressing `Down Arrow`), the file is not permanently deleted. Instead, it is moved to your operating system's native trash bin:

- **Windows:** Moved to the Windows Recycle Bin using the native shell API.
- **macOS:** Moved to the Apple Trash folder using Cocoa's `NSFileManager` API.
- **Linux:** Moved to the user's Desktop Trash according to the Freedesktop.org trash specification.

### Undo/Redo Mechanism (Restoration)

Because the files are sent to the native trash, they can be programmatically restored to their exact original location if you press the `Undo` key (`Q`):

1. **State Preservation:** Media Sort maintains a reference handle (`TrashRestoreHandle`) containing the original file path and details.
2. **Dropping & Flushing:** While the application is running, the handles remain in the undo stack. If you quit the application, the undo stack is cleared, and these handles are dropped. Upon dropping, the action is "flushed" (committed to the OS Trash bin, meaning they can no longer be restored using the application's Undo button, but they still remain in your system Recycle Bin/Trash where you can restore them manually).
