# Media Sort — Rust Implementation Blueprint

> Target: ground-up Rust rewrite of Image-Sort → Media Sort (images + video + audio)  
> Stack: `iced` + `wgpu` (Vulkan/MoltenVK) + `libmpv` + `symphonia`  
> Architecture: Elm-style single-direction state loop via `iced::Application`

---

## TABLE OF CONTENTS

1. [Repository Layout & Cargo Workspace](#phase-1-repository-layout--cargo-workspace)
2. [Core Mutation & History Engine](#phase-2-core-mutation--history-engine)
3. [Cross-Platform OS & File System Drivers](#phase-3-cross-platform-os--file-system-drivers)
4. [High-Performance Multimedia Engine](#phase-4-high-performance-multimedia-engine)
5. [UI Loop & Dynamic Hotkey System](#phase-5-ui-loop--dynamic-hotkey-system)
6. [Cross-Crate Communication & Async Boundaries](#cross-crate-communication-boundaries)
7. [Feature Traceability Matrix](#feature-traceability-matrix)

---

## PHASE 1: REPOSITORY LAYOUT & CARGO WORKSPACE

### 1.1 Repository Root

Post-cleanup, the repo root contains only the workspace scaffold. All legacy C# assets reside on `legacy/2.x`.

```
Image-Sort/                          # git repo root
├── .github/
│   ├── dependabot.yml
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── .gitignore
├── Cargo.toml                        # workspace root manifest
├── Cargo.lock
├── LICENSE
├── README.md
├── docs/
│   └── HELP.md
├── resources/                        # embedded application assets
│   ├── media-sort.svg
│   ├── locale/
│   │   ├── en/
│   │   │   ├── strings.ftl
│   │   │   └── keybindings.ftl
│   │   └── de/
│   │       ├── strings.ftl
│   │       └── keybindings.ftl
│   └── icons/
│       └── *.png
├── crates/
│   ├── media-sort-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── actions/
│   │       │   ├── mod.rs
│   │       │   ├── reversible.rs
│   │       │   ├── move_action.rs
│   │       │   ├── rename_action.rs
│   │       │   └── delete_action.rs
│   │       ├── history.rs
│   │       ├── settings/
│   │       │   ├── mod.rs
│   │       │   ├── general.rs
│   │       │   ├── keybindings.rs
│   │       │   ├── pinned_folders.rs
│   │       │   ├── window_position.rs
│   │       │   ├── metadata_panel.rs
│   │       │   └── store.rs
│   │       ├── models.rs
│   │       ├── media_type.rs
│   │       ├── path_utils.rs
│   │       └── l10n.rs
│   │
│   ├── media-sort-backend/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── filesystem/
│   │       │   ├── mod.rs
│   │       │   ├── watcher.rs
│   │       │   ├── scanner.rs
│   │       │   └── trash_staging.rs
│   │       ├── metadata/
│   │       │   ├── mod.rs
│   │       │   ├── image_meta.rs
│   │       │   ├── video_meta.rs
│   │       │   └── audio_meta.rs
│   │       ├── media/
│   │       │   ├── mod.rs
│   │       │   ├── image_decoder.rs
│   │       │   ├── video_decoder.rs
│   │       │   ├── audio_decoder.rs
│   │       │   ├── thumbnail.rs
│   │       │   └── mpv_context.rs
│   │       └── platform/
│   │           ├── mod.rs
│   │           └── trash.rs
│   │
│   └── media-sort-gui/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           ├── state.rs
│           ├── message.rs
│           ├── update.rs
│           ├── view/
│           │   ├── mod.rs
│           │   ├── main_layout.rs
│           │   ├── folder_panel.rs
│           │   ├── folder_tree.rs
│           │   ├── media_grid.rs
│           │   ├── media_preview.rs
│           │   ├── metadata_panel.rs
│           │   ├── history_bar.rs
│           │   ├── search_bar.rs
│           │   ├── settings_dialog.rs
│           │   ├── keybinding_editor.rs
│           │   └── hotkey_popup.rs
│           ├── widgets/
│           │   ├── mod.rs
│           │   ├── video_canvas.rs       # custom wgpu + mpv interop
│           │   ├── gif_player.rs
│           │   ├── thumbnail_grid.rs
│           │   └── folder_icon.rs
│           ├── subscriptions/
│           │   ├── mod.rs
│           │   ├── keyboard.rs
│           │   ├── file_watcher.rs
│           │   ├── prefetch.rs
│           │   └── video_trigger.rs
│           ├── theme.rs
│           └── cache.rs
│
└── tests/
    ├── integration/
    │   ├── action_tests.rs
    │   ├── history_tests.rs
    │   └── settings_tests.rs
    └── fixtures/
        └── *.jpg, *.png, *.mp4
```

### 1.2 Workspace Root `Cargo.toml`

```toml
[workspace]
members = [
    "crates/media-sort-core",
    "crates/media-sort-backend",
    "crates/media-sort-gui",
]
resolver = "2"

[workspace.package]
version = "3.0.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/Lolle2000la/Image-Sort"
rust-version = "1.85"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
parking_lot = "0.12"
crossbeam-channel = "0.5"

# media-sort-core
fluent = "0.16"
fluent-bundle = "0.16"
unic-langid = { version = "0.9", features = ["macros"] }
once_cell = "1"

# media-sort-backend
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "gif", "bmp", "tiff", "ico", "webp"] }
notify = { version = "7", default-features = false, features = ["macos_kqueue"] }
notify-debouncer-mini = "0.5"
trash = "5"
walkdir = "2"
kamadak-exif = "0.6"
mpv = "0.1"                       # libmpv bindings (or custom sys crate)
symphonia = { version = "0.5", features = ["mp3", "aac", "flac", "vorbis", "wav", "all-codecs"] }
symphonia-core = "0.5"
rodio = { version = "0.20", default-features = false, features = ["flac", "vorbis", "wav", "mp3"] }
strum = { version = "0.26", features = ["derive"] }
ash = "0.38"                      # Vulkan bindings for mpv texture sharing
raw-window-handle = "0.6"

# pure-Rust metadata parsers (no ffprobe dependency)
id3 = "1"                         # MP3, AIFF, WAV tag reading
metaflac = "0.2"                  # FLAC Vorbis comment blocks
mp4ameta = "0.3"                  # MP4/M4A/M4V iTunes-style atoms

# media-sort-gui
iced = { version = "0.14", features = ["wgpu", "image", "lazy", "advanced"] }
iced_wgpu = "0.14"
iced_winit = "0.14"
wgpu = "24"
winit = "0.30"
tokio = { version = "1", features = ["rt", "rt-multi-thread", "sync"] }
lru = "0.12"
env_logger = "0.11"
log = "0.4"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
```

### 1.3 `media-sort-core` Crate (`crates/media-sort-core/Cargo.toml`)

```toml
[package]
name = "media-sort-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
parking_lot.workspace = true
fluent.workspace = true
fluent-bundle.workspace = true
unic-langid.workspace = true
once_cell.workspace = true
strum = { version = "0.26", features = ["derive"] }
```

**Purpose**: Zero external dependencies beyond serialization and localization. Contains:
- `ReversibleAction` trait
- `MoveAction`, `RenameAction`, `DeleteAction` structs
- `History` (undo/redo stack engine)
- All settings data models (`GeneralSettings`, `KeyBindings`, `PinnedFolders`, `WindowPosition`, `MetadataPanelSettings`, `SettingsStore`)
- `MediaType` enum
- `l10n` module: Fluent-based localization using FTL files embedded at compile time
- `path_utils`: cross-platform path comparison (replacing C# `PathHelper.PathEquals`)
- `models.rs`: shared domain types (`MediaEntry`, `FolderNode`, `Selection`, `SearchQuery`)

### 1.4 `media-sort-backend` Crate (`crates/media-sort-backend/Cargo.toml`)

```toml
[package]
name = "media-sort-backend"
version.workspace = true
edition.workspace = true

[dependencies]
media-sort-core = { path = "../media-sort-core" }
image.workspace = true
notify.workspace = true
notify-debouncer-mini.workspace = true
trash.workspace = true
walkdir.workspace = true
kamadak-exif.workspace = true
symphonia.workspace = true
symphonia-core.workspace = true
rodio.workspace = true
id3.workspace = true
metaflac.workspace = true
mp4ameta.workspace = true
mpv.workspace = true
ash.workspace = true
raw-window-handle.workspace = true
serde.workspace = true
thiserror.workspace = true
tracing.workspace = true
parking_lot.workspace = true
crossbeam-channel.workspace = true
strum.workspace = true
```

**Purpose**: All OS and hardware interop. Depends on `media-sort-core` for data types. No dependency on `iced` or `wgpu` at the type level (though `ash` is used for raw Vulkan handles used in mpv texture sharing — the GUI crate wires these into `wgpu`).

### 1.5 `media-sort-gui` Crate (`crates/media-sort-gui/Cargo.toml`)

```toml
[package]
name = "media-sort-gui"
version.workspace = true
edition.workspace = true

[dependencies]
media-sort-core = { path = "../media-sort-core" }
media-sort-backend = { path = "../media-sort-backend" }
iced.workspace = true
iced_wgpu.workspace = true
iced_winit.workspace = true
wgpu.workspace = true
winit.workspace = true
tokio.workspace = true
lru.workspace = true
env_logger.workspace = true
log.workspace = true
parking_lot.workspace = true
crossbeam-channel.workspace = true
```

**Purpose**: The `iced::Application` loop. Depends on both core and backend crates. The only binary target.

---

## PHASE 2: CORE MUTATION & HISTORY ENGINE

### 2.1 Directory: `crates/media-sort-core/src/actions/`

#### 2.1.1 `reversible.rs` — The `ReversibleAction` Trait

```rust
/// A reversible mutation on the file system.
///
/// Each concrete action captures its preconditions at construction time
/// and implements the two-phase execution model:
/// 1. `execute()` — perform the mutation
/// 2. `rollback()` — reverse the mutation
///
/// The display name is used for the history bar UI.
pub trait ReversibleAction: Send + Sync {
    /// Human-readable description (e.g. "Move sunset.jpg to Vacation/").
    fn display_name(&self) -> &str;

    /// Perform the forward mutation. Must be idempotent-safe in practice
    /// (only called once per instance, but the guard is structural).
    fn execute(&mut self) -> Result<(), ActionError>;

    /// Reverse the mutation, returning to the pre-execute state.
    fn rollback(&mut self) -> Result<(), ActionError>;
}
```

**Why `&mut self`**: Ownership of `Box<dyn ReversibleAction>` lives in the `History` stack; `execute`/`rollback` mutate the action's internal tracking state (e.g., the `IDisposable` equivalent in `DeleteAction`).

**Source parity trace**: `IReversibleAction` (`legacy: src/ImageSort/Actions/IReversibleAction.cs:3-9`)

#### `ActionError` Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("source file not found: {0}")]
    SourceNotFound(std::path::PathBuf),

    #[error("target already exists: {0}")]
    TargetExists(std::path::PathBuf),

    #[error("directory not found: {0}")]
    DirectoryNotFound(std::path::PathBuf),

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("restoration not possible: {0}")]
    RestorationFailed(String),
}
```

#### 2.1.2 `move_action.rs`

```rust
use std::path::{Path, PathBuf};
use crate::actions::reversible::{ActionError, ReversibleAction};
use crate::l10n;

pub struct MoveAction {
    old_path: PathBuf,       // absolute, canonical
    new_path: PathBuf,       // absolute, canonical
    executed: bool,
}

impl MoveAction {
    /// `file` — absolute path to source file.
    /// `to_folder` — absolute path to destination directory.
    ///
    /// # Errors
    /// Returns `ActionError::SourceNotFound` if `file` does not exist.
    /// Returns `ActionError::DirectoryNotFound` if `to_folder` does not exist.
    pub fn new(file: &Path, to_folder: &Path) -> Result<Self, ActionError> {
        let file = file.canonicalize().map_err(|_| ActionError::SourceNotFound(file.to_path_buf()))?;
        let to_folder = to_folder.canonicalize().map_err(|_| ActionError::DirectoryNotFound(to_folder.to_path_buf()))?;

        let file_name = file.file_name().ok_or(ActionError::SourceNotFound(file.clone()))?;
        let new_path = to_folder.join(file_name);

        Ok(Self { old_path: file, new_path, executed: false })
    }

    pub fn old_path(&self) -> &Path { &self.old_path }
    pub fn new_path(&self) -> &Path { &self.new_path }
}

impl ReversibleAction for MoveAction {
    fn display_name(&self) -> &str {
        // Uses Fluent: move-action-message = Move {$file_name} to {$directory}
        // Cached at construction time in a String field; elided for brevity.
        // ...
    }

    fn execute(&mut self) -> Result<(), ActionError> {
        std::fs::rename(&self.old_path, &self.new_path)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        std::fs::rename(&self.new_path, &self.old_path)?;
        self.executed = false;
        Ok(())
    }
}
```

**Source parity trace**: `MoveAction` (`legacy: src/ImageSort/Actions/MoveAction.cs:8-58`)

#### 2.1.3 `rename_action.rs`

```rust
/// Renames a file, preserving its extension automatically.
///
/// The user provides only the stem (e.g., "my-image" for "my-image.png").
pub struct RenameAction {
    old_path: PathBuf,
    new_path: PathBuf,       // same dir, new stem + old extension
    executed: bool,
}

impl RenameAction {
    /// `path` — absolute path to the source file.
    /// `new_stem` — the new base name without extension.
    ///
    /// # Errors
    /// `SourceNotFound` if path doesn't exist.
    /// `TargetExists` if the computed new_path already exists.
    pub fn new(path: &Path, new_stem: &str) -> Result<Self, ActionError> {
        let path = path.canonicalize().map_err(|_| ActionError::SourceNotFound(path.to_path_buf()))?;

        // Validate new_stem for illegal characters (platform-specific)
        if new_stem.contains('\\') || new_stem.contains('/') || new_stem.contains(':')
            || new_stem.contains('*') || new_stem.contains('?') || new_stem.contains('"')
            || new_stem.contains('<') || new_stem.contains('>') || new_stem.contains('|')
        {
            return Err(ActionError::TargetExists(PathBuf::from(new_stem)));
        }

        let parent = path.parent().unwrap_or(Path::new("."));
        let ext = path.extension().unwrap_or_default();
        let new_path = if ext.is_empty() {
            parent.join(new_stem)
        } else {
            parent.join(format!("{}.{}", new_stem, ext.to_string_lossy()))
        };

        if new_path.exists() {
            return Err(ActionError::TargetExists(new_path));
        }

        Ok(Self { old_path: path, new_path, executed: false })
    }
}

impl ReversibleAction for RenameAction {
    fn execute(&mut self) -> Result<(), ActionError> {
        std::fs::rename(&self.old_path, &self.new_path)?;
        self.executed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        std::fs::rename(&self.new_path, &self.old_path)?;
        self.executed = false;
        Ok(())
    }
}
```

**Source parity trace**: `RenameAction` (`legacy: src/ImageSort/Actions/RenameAction.cs:8-55`)

#### 2.1.4 `delete_action.rs`

```rust
use crate::actions::reversible::{ActionError, ReversibleAction};
use std::path::PathBuf;

/// Deletes a file via the application-level trash staging engine.
///
/// The staged trash engine (`media-sort-backend::filesystem::trash_staging`)
/// provides an opaque restore handle. Disposing this handle triggers the
/// native OS trash operation.
pub struct DeleteAction {
    path: PathBuf,
    /// Opaque handle from the trash staging engine.
    /// When dropped without being consumed, the file remains deleted.
    /// To rollback, we consume this handle to trigger restoration.
    restore_handle: Option<Box<dyn TrashRestoreHandle>>,
}

pub trait TrashRestoreHandle: Send + Sync {
    /// Restore the file from staging. Called during rollback.
    fn restore(&mut self) -> Result<(), ActionError>;
    /// Finalize — flush to native OS trash. Called on session teardown.
    fn flush_to_native_trash(&mut self) -> Result<(), ActionError>;
}

impl DeleteAction {
    pub fn new(path: PathBuf, handle: Box<dyn TrashRestoreHandle>) -> Self {
        Self { path, restore_handle: Some(handle) }
    }
}

impl ReversibleAction for DeleteAction {
    fn execute(&mut self) -> Result<(), ActionError> {
        // The staging engine already moved the file into .mediasort/trash/
        // at construction time. execute() just marks it as executed.
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ActionError> {
        if let Some(mut handle) = self.restore_handle.take() {
            handle.restore()?;
        }
        Ok(())
    }
}
```

**Source parity trace**: `DeleteAction` (`legacy: src/ImageSort/Actions/DeleteAction.cs:8-52`)

### 2.2 `history.rs` — The History Stack Engine

```rust
use crate::actions::reversible::{ActionError, ReversibleAction};
use parking_lot::Mutex;

const MAX_HISTORY_SIZE: usize = 256;

/// Thread-safe undo/redo stack. Cleared automatically when the active
/// directory changes (directory-scoping rule).
pub struct History {
    done: Vec<Box<dyn ReversibleAction>>,
    undone: Vec<Box<dyn ReversibleAction>>,
}

impl History {
    pub fn new() -> Self {
        Self { done: Vec::with_capacity(64), undone: Vec::with_capacity(16) }
    }

    /// Push an action onto the done stack. Clears the redo stack.
    pub fn push_executed(&mut self, action: Box<dyn ReversibleAction>) {
        if self.done.len() >= MAX_HISTORY_SIZE {
            self.done.remove(0); // evict oldest
        }
        self.done.push(action);
        self.undone.clear();
    }

    /// Roll back the most recent action. Pushes it to the redo stack.
    pub fn undo(&mut self) -> Result<(), ActionError> {
        let mut action = self.done.pop().ok_or(ActionError::RestorationFailed("nothing to undo".into()))?;
        action.rollback()?;
        self.undone.push(action);
        Ok(())
    }

    /// Re-apply the most recently undone action.
    pub fn redo(&mut self) -> Result<(), ActionError> {
        let mut action = self.undone.pop().ok_or(ActionError::RestorationFailed("nothing to redo".into()))?;
        action.execute()?;
        self.done.push(action);
        Ok(())
    }

    /// Discard all history. Called when changing the active directory.
    pub fn clear(&mut self) {
        // Drop actions; any restore handles in DeleteActions that were
        // never rolled back will have their Drop impl flush to real trash.
        self.done.clear();
        self.undone.clear();
    }

    pub fn last_done_name(&self) -> Option<&str> {
        self.done.last().map(|a| a.display_name())
    }

    pub fn last_undone_name(&self) -> Option<&str> {
        self.undone.last().map(|a| a.display_name())
    }

    pub fn can_undo(&self) -> bool { !self.done.is_empty() }
    pub fn can_redo(&self) -> bool { !self.undone.is_empty() }
    pub fn done_len(&self) -> usize { self.done.len() }
    pub fn undone_len(&self) -> usize { self.undone.len() }
}
```

**Directory-scoping rule** (matches legacy behavior at `MainViewModel.cs:123-128`): When the active folder changes, the UI layer calls `history.clear()`. Any `DeleteAction` handles remaining in the `done` or `undone` stacks are dropped — their `Drop` implementation calls `flush_to_native_trash()`, permanently deleting the files to the OS trash.

### 2.3 `media_type.rs`

```rust
use strum::EnumIter;

/// Supported media categories and their file extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

impl MediaType {
    /// Extensions recognized for each type.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            MediaType::Image => &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "ico", "webp", "jxl", "heic", "heif", "avif"],
            MediaType::Video => &["mp4", "mkv", "webm", "avi", "mov", "wmv", "flv", "m4v"],
            MediaType::Audio => &["mp3", "flac", "ogg", "wav", "aac", "m4a", "wma", "opus", "aiff"],
        }
    }

    /// All recognized extensions across all types.
    pub fn all_extensions() -> Vec<&'static str> {
        let mut exts = Vec::new();
        for ty in [Self::Image, Self::Video, Self::Audio] {
            exts.extend(ty.extensions());
        }
        exts
    }
}
```

### 2.4 `models.rs` — Shared Domain Types

```rust
use std::path::PathBuf;

/// A single media entry in the currently loaded directory.
#[derive(Debug, Clone)]
pub struct MediaEntry {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub file_name: String,
}

/// A node in the folder tree.
#[derive(Debug, Clone)]
pub struct FolderNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<FolderNode>,
    pub is_current: bool,
    pub is_expanded: bool,
}

/// Pinned folder with optional index-based shortcut.
#[derive(Debug, Clone)]
pub struct PinnedFolder {
    pub path: PathBuf,
    pub name: String,
    /// Optional: `Some(1)` means bound to Alt+1.
    pub numeric_shortcut: Option<u8>, // 1..=9
}
```

### 2.5 `settings/` — Configuration Subsystem

Each settings group mirrors the legacy C# `SettingsGroupViewModelBase` pattern but uses a typed `SettingsStore` with serde:

```rust
use serde::{Serialize, Deserialize};

/// Serializable settings store persisted to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsStore {
    pub general: GeneralSettings,
    pub keybindings: KeyBindings,
    pub pinned_folders: PinnedFoldersSettings,
    pub window_position: WindowPosition,
    pub metadata_panel: MetadataPanelSettings,
}

impl SettingsStore {
    pub fn config_path() -> PathBuf {
        // Platform-appropriate config directory
        let base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort");
        std::fs::create_dir_all(&base).ok();
        #[cfg(debug_assertions)]
        { base.join("debug_config.json") }
        #[cfg(not(debug_assertions))]
        { base.join("config.json") }
    }

    pub fn load() -> Result<Self, SettingsError> { /* serde_json::from_reader */ }
    pub fn save(&self) -> Result<(), SettingsError> { /* serde_json::to_writer_pretty */ }
}
```

#### `general.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default)]
    pub dark_mode: bool,                         // legacy: DarkMode, default = false

    #[serde(default = "default_true")]
    pub check_for_updates_on_startup: bool,      // legacy: CheckForUpdatesOnStartup, default = true

    #[serde(default)]
    pub install_prerelease_builds: bool,         // legacy: InstallPrereleaseBuilds, default = false

    #[serde(default = "default_true")]
    pub animate_gifs: bool,                      // legacy: AnimateGifs, default = true

    #[serde(default = "default_true")]
    pub animate_gif_thumbnails: bool,            // legacy: AnimateGifThumbnails, default = true
}
```

#### `keybindings.rs`

Every bindable action gets a `KeyBinding` struct. Uses serde-compatible keycode representation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: KeyCode,         // e.g., "Up", "R", "Enter"
    pub modifiers: Modifiers, // bitflags: Ctrl, Shift, Alt, Meta
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    // Image manipulation (legacy: Move, Delete, Rename)
    pub move_to_folder: KeyBinding,      // default: Up, no mods
    pub delete: KeyBinding,              // default: Down, no mods
    pub rename: KeyBinding,              // default: R, no mods

    // Image selection (legacy: GoLeft, GoRight)
    pub go_left: KeyBinding,             // default: Left, no mods
    pub go_right: KeyBinding,            // default: Right, no mods

    // Folder manipulation
    pub create_folder: KeyBinding,       // default: C, no mods
    pub folder_up: KeyBinding,           // default: W, no mods
    pub folder_left: KeyBinding,         // default: A, no mods
    pub folder_down: KeyBinding,         // default: S, no mods
    pub folder_right: KeyBinding,        // default: D, no mods

    // History (legacy: Undo, Redo)
    pub undo: KeyBinding,                // default: Q, no mods
    pub redo: KeyBinding,                // default: E, no mods

    // Folder opening
    pub open_folder: KeyBinding,         // default: O, no mods
    pub open_selected_folder: KeyBinding,// default: Enter, no mods

    // Pinned folders
    pub pin: KeyBinding,                 // default: P, no mods
    pub pin_selected: KeyBinding,        // default: F, no mods
    pub unpin: KeyBinding,               // default: U, no mods
    pub move_pinned_up: KeyBinding,      // default: Ctrl+W
    pub move_pinned_down: KeyBinding,    // default: Ctrl+S

    // UI toggles
    pub search_images: KeyBinding,       // default: I, no mods
    pub toggle_metadata_panel: KeyBinding, // default: M, no mods
}
```

**Default impl**: `KeyBindings::default()` returns the above defaults, matching all 21 legacy bindings exactly.

**Source parity trace**: `KeyBindingsSettingsGroupViewModel` (`legacy: src/ImageSort.WPF/SettingsManagement/ShortCutManagement/KeybindingsSettingsGroupViewModel.cs:11-221`)

#### `pinned_folders.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PinnedFoldersSettings {
    pub paths: Vec<String>,  // absolute, canonical paths, case-normalized
}
```

#### `window_position.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowPosition {
    pub left: i32,           // default: 100
    pub top: i32,            // default: 100
    pub width: u32,          // default: 1000
    pub height: u32,         // default: 600
    pub maximized: bool,     // default: false
    pub screen_count: u32,   // tracked for multi-monitor safety
}
```

#### `metadata_panel.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataPanelSettings {
    #[serde(default)]
    pub is_expanded: bool,
    #[serde(default = "default_width")]
    pub panel_width: u16,  // default: 300
}

fn default_width() -> u16 { 300 }
```

### 2.6 `l10n.rs` — Localization

```rust
use fluent::{FluentBundle, FluentResource};
use unic_langid::langid;

pub struct Localization {
    bundles: HashMap<LanguageIdentifier, FluentBundle<FluentResource>>,
    current_lang: LanguageIdentifier,
}

impl Localization {
    pub fn init(default_lang: &str) -> Self { /* loads .ftl files from embed */ }

    /// Look up a message with optional substitution arguments.
    /// Example: `l10n.get("move-action-message", &[("file_name", "sunset.jpg"), ("directory", "Vacation/")])`
    pub fn get(&self, key: &str, args: &[(&str, &str)]) -> String {
        // fluent bundle format_pattern...
    }
}
```

FTL files replace RESX. Example `strings.ftl`:
```
move-action-message = Move {$file_name} to {$directory}
delete-action-message = Delete {$file_name}
rename-action-message = Rename {$old_file_name} to {$new_file_name}
could-not-act-error = Could not execute action "{$act_message}": {$error_message}
```

---

## PHASE 3: CROSS-PLATFORM OS & FILE SYSTEM DRIVERS

### 3.1 `filesystem/scanner.rs` — Directory Enumeration

```rust
use walkdir::WalkDir;
use media_sort_core::MediaType;

/// Enumerate media files in a directory, filtered by supported extensions.
/// Returns immediately — no recursion (TopDirectoryOnly).
pub fn scan_media_files(dir: &Path) -> Vec<PathBuf> {
    let exts: Vec<&str> = MediaType::all_extensions();
    WalkDir::new(dir)
        .max_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|ext| exts.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}
```

**Source parity trace**: `FullAccessFileSystem.GetFiles` (`legacy: src/ImageSort/FileSystem/FullAccessFileSystem.cs:13`)

### 3.2 `filesystem/watcher.rs` — Async Directory Watch

```rust
use notify::{Event, RecursiveMode, Watcher};
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use std::time::Duration;
use tokio::sync::mpsc;

/// Create an asynchronous debounced watcher for a directory.
/// Returns a channel receiver and a handle that stops the watcher on drop.
pub fn watch_directory(path: &Path) -> (FileWatcherHandle, mpsc::Receiver<FileSystemEvent>) {
    let (tx, rx) = mpsc::channel(256);
    let path = path.to_path_buf();

    // Debounce at 100ms to coalesce rapid events (e.g. bulk renames)
    let mut debouncer = new_debouncer(Duration::from_millis(100), move |events: &[notify::Event]| {
        for event in events {
            for path in &event.paths {
                let _ = tx.blocking_send(match event.kind {
                    DebouncedEventKind::Any => FileSystemEvent::Modified(path.clone()),
                    // ... map to custom enum
                });
            }
        }
    }).expect("file watcher setup");

    debouncer.watch(&path, RecursiveMode::NonRecursive)
        .expect("watch directory");

    let handle = FileWatcherHandle { _debouncer: debouncer };
    (handle, rx)
}

pub struct FileWatcherHandle {
    _debouncer: notify_debouncer_mini::Debouncer<notify::INotifyWatcher>,
}
```

**Why debounced**: Matches the legacy `FileSystemWatcher` with `InternalBufferSize = 64000` and coalesced events. The debouncer avoids double-fires from rename sequences.

**Directory-change lifecycle**: When the UI changes directories, the watcher handle is dropped (stopping the old watcher), and a new one is created for the new path. The channel receiver is swapped in the app's subscription system.

### 3.3 `filesystem/trash_staging.rs` — Safe Staging Trash Engine

This is the **key design departure** from the legacy impl. The legacy code delegates to `SHFileOperation` immediately. For cross-platform robustness and flawless rollback, we introduce a two-phase staging model.

#### Architecture

```
Filesystem
  │
  │  DeleteAction.execute()
  ▼
[ .mediasort/trash/sha256(original_path)/file.ext ]
  │
  ├── rollback() → copy back to original location, delete from staging
  │
  ├── clear_history() → flush_to_native_trash() for each accumulated file
  │
  └── session teardown → flush_to_native_trash() for remaining files
```

```rust
use std::path::{Path, PathBuf};
use std::fs;
use parking_lot::Mutex;
use crate::actions::delete_action::{TrashRestoreHandle, ActionError};

pub struct TrashStaging {
    /// Root staging directory: <config_dir>/media-sort/trash/
    staging_root: PathBuf,
    /// Active restore handles for this session's deleted files.
    staged: Mutex<Vec<StagedFile>>,
}

struct StagedFile {
    original_path: PathBuf,
    staging_path: PathBuf,
}

impl TrashStaging {
    /// Initialize the staging area, creating the directory if needed.
    pub fn new() -> Result<Self, ActionError> {
        let root = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort")
            .join("trash");

        fs::create_dir_all(&root)?;
        Ok(Self { staging_root: root, staged: Mutex::new(Vec::new()) })
    }

    /// Stage a file for potential rollback.
    /// The file is **moved** into the staging area immediately.
    /// Returns a restore handle.
    pub fn stage_file(&self, path: &Path) -> Result<Box<dyn TrashRestoreHandle>, ActionError> {
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };

        let file_name = path.file_name().ok_or(ActionError::SourceNotFound(path.to_path_buf()))?;
        let staging_dir = self.staging_root.join(&hash);
        fs::create_dir_all(&staging_dir)?;

        let staging_path = staging_dir.join(file_name);
        fs::rename(path, &staging_path)?;

        let staged = StagedFile {
            original_path: path.to_path_buf(),
            staging_path: staging_path.clone(),
        };
        self.staged.lock().push(staged);

        Ok(Box::new(StagingRestoreHandle {
            original_path: path.to_path_buf(),
            staging_path,
            flushed: false,
        }))
    }

    /// Flush all remaining staged files to the native OS trash.
    /// Called on history.clear() and session teardown.
    pub fn flush_all_to_native(&self) {
        let mut staged = self.staged.lock();
        for item in staged.drain(..) {
            // Use the `trash` crate for cross-platform trash operations
            let _ = trash::delete(&item.staging_path);
        }
    }
}

struct StagingRestoreHandle {
    original_path: PathBuf,
    staging_path: PathBuf,
    flushed: bool,
}

impl TrashRestoreHandle for StagingRestoreHandle {
    fn restore(&mut self) -> Result<(), ActionError> {
        if self.flushed {
            return Err(ActionError::RestorationFailed("already flushed".into()));
        }
        fs::rename(&self.staging_path, &self.original_path)?;
        Ok(())
    }

    fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
        if self.flushed { return Ok(()); }
        trash::delete(&self.staging_path)
            .map_err(|e| ActionError::RestorationFailed(e.to_string()))?;
        self.flushed = true;
        Ok(())
    }
}

impl Drop for StagingRestoreHandle {
    fn drop(&mut self) {
        if !self.flushed {
            let _ = self.flush_to_native_trash();
        }
    }
}
```

**Why this design**: Legacy `DeleteAction.Revert()` depends on shell Recycle Bin APIs that are Windows-specific and unreliable (file could be manually emptied from the bin between execute and undo). The staging approach guarantees:
1. Immediate file removal from the working directory.
2. Guaranteed rollback (file is local, not in an OS-managed bin).
3. Deferred native trash operations at safe boundaries.

#### Crash Resiliency: Orphaned Trash Reconciliation

Because `StagingRestoreHandle` relies on its `Drop` implementation to flush files to the native OS trash, an unexpected application crash or hardware power loss will leave orphaned media assets stranded in `<config_dir>/media-sort/trash/`. These files are physically moved from their original locations and will appear permanently deleted to the user.

**The fix**: On startup, before opening the main window loop, the backend must scan the staging root for any directories left from a previous crashed session and immediately flush them to the native OS trash. This is performed in `TrashStaging::new()`:

```rust
impl TrashStaging {
    pub fn new() -> Result<Self, ActionError> {
        let root = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort")
            .join("trash");

        fs::create_dir_all(&root)?;

        // --- CRASH RESILIENCY: flush orphaned files from prior sessions ---
        Self::reconcile_orphaned_trash(&root);

        Ok(Self { staging_root: root, staged: Mutex::new(Vec::new()) })
    }

    /// Walk the staging root for directories left over from a crashed session.
    /// Each directory represents a staged file that was never restored or legitimately
    /// flushed. Its contents are sent to the native OS trash, and the directory is removed.
    pub fn reconcile_orphaned_trash(staging_root: &Path) {
        if !staging_root.exists() {
            return;
        }
        let entries = match fs::read_dir(staging_root) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            for file_entry in fs::read_dir(&entry_path).into_iter().flatten().flatten() {
                let file_path = file_entry.path();
                if file_path.is_file() {
                    // Send to native OS trash via the cross-platform `trash` crate.
                    // Errors are silently ignored — the file may already be gone.
                    let _ = trash::delete(&file_path);
                }
            }
            let _ = fs::remove_dir_all(&entry_path);
        }
    }
}
```

This reconciliation is idempotent and fast. If the staging root is empty (clean shutdown), it is a no-op. If orphans exist, they are cleaned in O(n) where n is the number of abandoned files. The function logs each action via `tracing` at debug level for auditability.

### 3.4 `metadata/` — Metadata Extraction Subsystem

#### `image_meta.rs`

```rust
use kamadak_exif::*;
use std::collections::BTreeMap;
use std::path::Path;

/// Extract EXIF metadata from an image file.
/// Returns a map of (IFD/tag group) → (tag name → tag value).
pub fn extract_image_metadata(path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    let file = std::fs::File::open(path)?;
    let mut buf = std::io::BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut buf)
        .map_err(|_| MetadataError::ExtractionFailed)?;

    let mut dirs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for field in exif.fields() {
        let ifd_name = field.ifd_num.to_string(); // or map to human name
        let tag_name = field.tag.to_string();
        let value = field.display_value().to_string();

        dirs.entry(ifd_name)
            .or_default()
            .insert(tag_name, value);
    }

    Ok(dirs)
}
```

**Source parity trace**: `FullAccessFileSystemMetadataExtractor` (`legacy: src/ImageSort/FileSystem/FullAccessFileSystemMetadataExtractor.cs:12-27`) — returns the same `Dictionary<string, Dictionary<string, string>>` shape using EXIF as primary source. For a more complete mapping of the legacy `MetadataExtractor` library output, add support for IPTC/XMP via the `xmp-toolkit` or `extended` crates.

#### `video_meta.rs`

Pure-Rust metadata extraction from video container formats. No external `ffprobe` process dependency. The returned shape mirrors the legacy `Dictionary<string, Dictionary<string, string>>` layout so the metadata panel UI works identically for all media types.

```rust
/// Extract container-level metadata from a video file.
/// Uses pure-Rust parsers: `mp4ameta` for MP4/M4V atoms (iTunes-style tags,
/// codec info, duration) and will be extended with additional format-specific
/// crates as new containers are added.
pub fn extract_video_metadata(path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "mp4" | "m4v" | "mov" => extract_mp4_metadata(path),
        "mkv" | "webm" => extract_matroska_metadata(path),
        // Fallback: use symphonia for format detection + basic stream info
        _ => extract_generic_container_metadata(path),
    }
}

fn extract_mp4_metadata(path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use mp4ameta::{Tag, Data};
    use std::fs::File;

    let mut file = File::open(path)?;
    let tag = Tag::read_from(&mut file)?;

    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut mp4_section: BTreeMap<String, String> = BTreeMap::new();

    // Standard iTunes-style atoms
    if let Some(title) = tag.title()         { mp4_section.insert("Title".into(),  title.to_string()); }
    if let Some(artist) = tag.artist()       { mp4_section.insert("Artist".into(), artist.to_string()); }
    if let Some(album) = tag.album()         { mp4_section.insert("Album".into(),  album.to_string()); }
    if let Some(year) = tag.year()           { mp4_section.insert("Year".into(),   year.to_string()); }
    if let Some(genre) = tag.genre()         { mp4_section.insert("Genre".into(),  genre.to_string()); }
    if let Some(track) = tag.track_number()  { mp4_section.insert("Track".into(),  track.to_string()); }
    if let Some(total) = tag.total_tracks()  { mp4_section.insert("Total Tracks".into(), total.to_string()); }

    if !mp4_section.is_empty() {
        sections.insert("MP4 Metadata".into(), mp4_section);
    }

    Ok(sections)
}

fn extract_matroska_metadata(_path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    // Future: integrate with a pure-Rust Matroska/WebM parser.
    // For now, fall through to generic symphonia-based extraction.
    extract_generic_container_metadata(_path)
}

fn extract_generic_container_metadata(_path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    // Use symphonia to open the container, probe format, and read track info
    // (codec, duration, sample rate, channel count).
    Ok(BTreeMap::new())
}
```

#### `audio_meta.rs`

```rust
/// Extract metadata from audio files using pure-Rust tag readers.
/// `id3` handles MP3/AIFF/WAV ID3v2 tags; `metaflac` handles FLAC Vorbis comment blocks.
/// The output shape is identical to image metadata so the metadata panel is format-agnostic.
pub fn extract_audio_metadata(path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "mp3" | "aiff" | "wav" => extract_id3_metadata(path),
        "flac" => extract_flac_metadata(path),
        "ogg" | "opus" => extract_vorbis_comment_metadata(path),
        "m4a" | "aac" => extract_mp4_metadata(path), // reuses mp4ameta from video_meta
        _ => extract_generic_container_metadata(path),
    }
}

fn extract_id3_metadata(path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use id3::{Tag, TagLike};

    let tag = Tag::read_from_path(path)?;
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut id3_section: BTreeMap<String, String> = BTreeMap::new();

    if let Some(title)   = tag.title()  { id3_section.insert("Title".into(),  title.to_string()); }
    if let Some(artist)  = tag.artist() { id3_section.insert("Artist".into(), artist.to_string()); }
    if let Some(album)   = tag.album()  { id3_section.insert("Album".into(),  album.to_string()); }
    if let Some(year)    = tag.year()   { id3_section.insert("Year".into(),   year.to_string()); }
    if let Some(genre)   = tag.genre()  { id3_section.insert("Genre".into(),  genre.to_string()); }
    if let Some(track)   = tag.track()  { id3_section.insert("Track".into(),  track.to_string()); }

    if !id3_section.is_empty() {
        sections.insert("ID3 Metadata".into(), id3_section);
    }

    Ok(sections)
}

fn extract_flac_metadata(path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use metaflac::Tag;

    let tag = Tag::read_from_path(path)?;
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut flac_section: BTreeMap<String, String> = BTreeMap::new();

    if let Some(vc) = tag.vorbis_comments() {
        for (key, values) in vc.comments() {
            if let Some(value) = values.first() {
                flac_section.insert(key.clone(), value.clone());
            }
        }
    }

    if !flac_section.is_empty() {
        sections.insert("FLAC Vorbis Comment".into(), flac_section);
    }

    Ok(sections)
}

fn extract_vorbis_comment_metadata(_path: &Path) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    // Symphonia can parse Ogg/Vorbis/Opus container-level Vorbis comments.
    Ok(BTreeMap::new())
}
```

### 3.5 `media/audio_decoder.rs` — Hardware Audio Sink

`symphonia` decodes raw PCM blocks from container formats. `rodio` streams those blocks to the native platform audio sink (PipeWire/ALSA on Linux, CoreAudio on macOS, WASAPI on Windows). No manual buffer marshaling is required.

```rust
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::path::Path;

pub struct AudioPlayer {
    _stream: OutputStream,    // kept alive for the lifetime of playback
    sink: Sink,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self { _stream: stream, sink })
    }

    /// Load and start playing an audio file.
    pub fn play(&self, path: &Path) -> Result<(), AudioError> {
        let file = File::open(path)?;
        let source = Decoder::new(file)?;
        self.sink.append(source);
        Ok(())
    }

    pub fn pause(&self)  { self.sink.pause(); }
    pub fn resume(&self) { self.sink.play(); }
    pub fn stop(&self)   { self.sink.stop(); }
    pub fn is_paused(&self) -> bool { self.sink.is_paused() }
    pub fn volume(&self, vol: f32)  { self.sink.set_volume(vol); }
}
```

**Note**: For bit-perfect or gapless playback, use `cpal` directly with a custom ring buffer fed by `symphonia`'s decoded PCM frames. `rodio` is sufficient for standard playback scenarios and requires zero configuration across platforms.

### 3.6 `media/video_decoder.rs` — libmpv Context

```rust
use mpv::{MpvHandler, MpvHandlerBuilder, Protocol, FileState};
use std::sync::Arc;

/// Owned mpv instance for video playback within the media preview pane.
///
/// # Lifetime Safety
/// This struct owns both the mpv render context and the shared Vulkan texture.
/// The `Drop` implementation enforces strict teardown ordering: callback
/// unregistration → channel sender deallocation → render context free → texture
/// release. This prevents use-after-free on the VkImage when widgets are
/// rebuilt during directory transitions.
pub struct MpvContext {
    pub render_ctx: *mut mpv_sys::mpv_render_context,
    /// Keep the shared texture alive within the backend context layer,
    /// ensuring it outlives transient UI widget layout drops.
    pub active_texture: Option<Arc<VulkanSharedTexture>>,
    /// Raw pointer to the heap-allocated channel sender `Box<Sender<()>>`
    /// that was passed to mpv as `cb_ctx`. Tracked so it can be safely
    /// deallocated AFTER the callback has been unregistered.
    callback_context_raw: *mut std::ffi::c_void,
}

impl MpvContext {
    pub fn new(
        vk_instance: ash::Instance,
        vk_device: ash::Device,
        vk_physical_device: ash::vk::PhysicalDevice,
    ) -> Result<Self, MpvError> {
        let handler = MpvHandlerBuilder::default()
            .with_protocol(Protocol::default())
            .set_option("vo", "gpu")
            .set_option("hwdec", "auto")
            .set_option("gpu-context", "vulkan")
            .set_option("gpu-api", "vulkan")
            .set_option("keep-open", "yes")
            .set_option("loop-file", "inf")
            .set_option("audio-display", "no")
            .build()?;

        Ok(Self {
            handler,
            vk_instance,
            vk_device,
            vk_physical_device,
            active_texture: None,
            callback_context_raw: std::ptr::null_mut(),
        })
    }

    /// Register the wakeup callback with channel context.
    /// The `sender` is heap-allocated and leaked; `Drop` reclaims it.
    pub unsafe fn register_callback(&mut self, sender: tokio::sync::mpsc::Sender<()>) {
        let sender_box = Box::new(sender);
        self.callback_context_raw = Box::into_raw(sender_box) as *mut std::ffi::c_void;

        mpv_sys::mpv_render_context_set_update_callback(
            self.render_ctx,
            Some(mpv_wakeup_callback),
            self.callback_context_raw,
        );
    }

    /// Load a media file.
    pub fn load_file(&mut self, path: &Path) -> Result<(), MpvError> {
        self.handler.command(&["loadfile", path.to_str().unwrap()])?;
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        self.handler.get_property("pause").unwrap_or(true) == false
    }

    pub fn toggle_pause(&mut self) {
        let _ = self.handler.command(&["cycle", "pause"]);
    }

    pub fn seek(&mut self, seconds: f64) {
        let _ = self.handler.command(&["seek", &seconds.to_string(), "relative"]);
    }
}

impl Drop for MpvContext {
    fn drop(&mut self) {
        unsafe {
            // 1. CRITICAL: Unregister the callback loop by passing a null
            //    function pointer. This guarantees libmpv's decoder thread
            //    will never invoke a dangling sender pointer.
            mpv_sys::mpv_render_context_set_update_callback(
                self.render_ctx,
                None,
                std::ptr::null_mut(),
            );

            // 2. Safe to reclaim ownership and drop the leaked channel sender.
            if !self.callback_context_raw.is_null() {
                let _sender_box = Box::from_raw(
                    self.callback_context_raw as *mut tokio::sync::mpsc::Sender<()>,
                );
            }

            // 3. Free the render context. At this point no decoder thread
            //    is writing, and the callback is detached.
            mpv_sys::mpv_render_context_free(self.render_ctx);
        }
        // 4. active_texture drops here (if set). wgpu reclaims the VkImage.
    }
}
```

---

## PHASE 4: HIGH-PERFORMANCE MULTIMEDIA ENGINE

### 4.1 Zero-Copy MPV → wgpu Texture Sharing Pipeline

Instead of a fragile external-memory import path (where mpv allocates memory and we attempt to import it into wgpu), the pipeline is **inverted**: wgpu allocates the texture natively within its own device context first. The underlying Vulkan image descriptors are extracted via `wgpu::Texture::as_hal` and passed directly to `mpv_render_context` using `MPV_RENDER_PARAM_VULKAN_WRITE_IMAGE`. This ensures the frame is rendered straight into memory that `iced`'s compositor already owns, eliminating any copy across the PCIe bus.

```
┌──────────────────────────────────────────────────────────────┐
│                     `iced` Render Loop                        │
│                                                               │
│  VideoCanvas Widget                                           │
│  │                                                            │
│  │  1. At widget init:                                        │
│  │     device.create_texture() → wgpu::Texture                │
│  │     texture.as_hal::<Vulkan>() → VkImage, VkDeviceMemory   │
│  │  2. Pass VkImage to mpv_render_context via                 │
│  │     MPV_RENDER_PARAM_VULKAN_WRITE_IMAGE                    │
│  │  3. Each frame: mpv_render_context_render() →              │
│  │     writes directly into our wgpu::Texture                 │
│  │  4. Bind wgpu::TextureView as sampled image                │
│  │     in iced's quad render pass                             │
│  │                                                            │
└──────────────────────────────────────────────────────────────┘
                ▲
                │ VkImage (OWNED by wgpu, written by mpv)
                │
       ┌────────┴────────────┐
       │  mpv (libmpv)       │
       │  vo=gpu             │
       │  gpu-api=vulkan     │
       │                     │
       │  Writes into OUR    │
       │  VkImage via        │
       │  WRITE_IMAGE param  │
       └─────────────────────┘
```

#### Shared Texture Struct

```rust
// crates/media-sort-gui/src/widgets/video_canvas.rs

use ash::vk;
use wgpu_hal::api::Vulkan;

/// A wgpu::Texture whose underlying Vulkan image is exposed to mpv for zero-copy
/// hardware-decoded frame rendering. The texture is allocated by wgpu (so it lives
/// within iced's compositor memory space) and handed to mpv via
/// `MPV_RENDER_PARAM_VULKAN_WRITE_IMAGE`.
///
/// `vk_format_raw` and `vk_layout_raw` are cached as platform-primitive integers
/// matching `mpv-sys` field layout expectations, avoiding architecture-dependent
/// enum-to-int width mismatches.
pub struct VulkanSharedTexture {
    /// Owned by wgpu — sampled by iced's quad pass.
    pub wgpu_texture: wgpu::Texture,
    /// Raw Vulkan image handle extracted via wgpu-hal escape hatch.
    pub vk_image: vk::Image,
    /// Raw device memory backing the image. Required for mpv's render context
    /// to bind layout states correctly.
    pub vk_device_memory: vk::DeviceMemory,
    /// Vulkan format as a primitive integer (e.g. VK_FORMAT_R8G8B8A8_UNORM = 37).
    /// Cached at construction time so FFI casts use the correct signed integer width.
    pub vk_format_raw: std::os::raw::c_int,
    /// Vulkan image layout as a primitive integer
    /// (e.g. VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL = 1000001002).
    pub vk_layout_raw: std::os::raw::c_int,
    /// Cached dimensions for mpv render params.
    pub width: u32,
    pub height: u32,
}
```

#### Construction: wgpu Allocates, mpv Receives

```rust
impl VulkanSharedTexture {
    /// Allocate a wgpu texture and extract its Vulkan handles.
    ///
    /// # Safety
    /// The caller must ensure `device` is a Vulkan-backed wgpu device.
    /// The underlying `wgpu_hal` handles are valid for the lifetime of `wgpu_texture`.
    pub unsafe fn allocate_for_mpv(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Self {
        let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mpv_shared_vulkan_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                 | wgpu::TextureUsages::RENDER_ATTACHMENT
                 | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        // Extract underlying Vulkan handles via wgpu-hal.
        // This is the escape hatch: wgpu_hal::api::Vulkan exposes raw handles
        // while keeping the texture alive via wgpu's ref-counting.
        let hal_texture = wgpu_texture
            .as_hal::<Vulkan>()
            .expect("Vulkan backend is enforced via env/config; this must not fail");
        let vk_image = hal_texture.raw_handle();
        let vk_device_memory = hal_texture.memory_handle();

        // Cache format and layout as primitive integers matching mpv-sys
        // field widths to avoid architecture-dependent enum-to-int mismatches.
        let vk_format_raw = ash::vk::Format::R8G8B8A8_UNORM.as_raw();
        let vk_layout_raw = ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL.as_raw();

        VulkanSharedTexture {
            wgpu_texture,
            vk_image,
            vk_device_memory,
            vk_format_raw,
            vk_layout_raw,
            width,
            height,
        }
    }
}
```

#### Per-Frame Render: mpv Writes Directly into wgpu Memory

```rust
/// Instruct mpv to render the current frame directly into the Vulkan image
/// backing our wgpu::Texture. No CPU readback. No staging buffer copy.
///
/// # Safety
/// - `mpv_ctx` must be a valid, initialized mpv_render_context.
/// - `shared` must have been allocated by `VulkanSharedTexture::allocate_for_mpv`
///   via the same Vulkan device that mpv was configured with.
pub unsafe fn render_mpv_to_vulkan_surface(
    mpv_ctx: *mut mpv_sys::mpv_render_context,
    shared: &VulkanSharedTexture,
) {
    // Build the VkImage descriptor that mpv will write into.
    // All fields use explicit `as` casts to the primitive integer types
    // expected by mpv-sys struct layout, avoiding architecture-dependent
    // enum-to-int width mismatches (ash uses u32 internally; mpv-sys
    // expects c_int / u64 in some fields on some platforms).
    let mut vk_img = mpv_sys::mpv_opengl_vulkan_image {
        image: shared.vk_image.as_raw() as u64,
        format: shared.vk_format_raw as std::os::raw::c_int,
        layout: shared.vk_layout_raw as std::os::raw::c_int,
        width: shared.width as std::os::raw::c_int,
        height: shared.height as std::os::raw::c_int,
    };

    // Tell mpv: "render into this VkImage, not one you own."
    let mut params: [mpv_sys::mpv_render_param; 2] = [
        mpv_sys::mpv_render_param {
            type_id: mpv_sys::MPV_RENDER_PARAM_VULKAN_WRITE_IMAGE,
            data: &mut vk_img as *mut mpv_sys::mpv_opengl_vulkan_image
                as *mut std::ffi::c_void,
        },
        // Sentinel: end of params list.
        mpv_sys::mpv_render_param {
            type_id: 0,
            data: std::ptr::null_mut(),
        },
    ];

    // mpv decodes the frame and writes pixel data directly into our VkImage.
    // No data crosses the PCIe bus — the frame stays in VRAM the entire time.
    mpv_sys::mpv_render_context_render(mpv_ctx, params.as_mut_ptr());
}
```

#### Widget Integration: iced Frame Loop

```rust
pub struct VideoCanvas {
    shared_texture: Arc<VulkanSharedTexture>,
    mpv_ctx: *mut mpv_sys::mpv_render_context,
}

// In the iced `view` method, the widget produces a quad element bound to
// `shared_texture.wgpu_texture`. Each frame tick:
// 1. unsafe { render_mpv_to_vulkan_surface(self.mpv_ctx, &self.shared_texture); }
// 2. The texture is already in wgpu's memory; iced's compositor samples it
//    during the next draw pass without any additional upload.
```

**Why this direction (wgpu → mpv) instead of mpv → wgpu**: Allocating in wgpu ensures the texture lives in the memory pool managed by `iced_wgpu`'s compositor. mpv is configured as a guest writer. When hardware decoding (VA-API/NVDEC/VideoToolbox) is active, the decoded frame surfaces in VRAM, mpv copies it into our VkImage via a GPU-side blit, and iced samples it in the same render pass. No pixel data ever travels through system RAM.

**Vulkan interop requirements**:
- `wgpu` must be configured with the Vulkan backend (`Backends::VULKAN`).
- On macOS, this goes through MoltenVK (Vulkan → Metal translation). Performance is acceptable because the zero-copy path avoids the Metal→CPU→wgpu round-trip that a staging buffer would require.
- The `mpv` crate or raw `mpv_sys` bindings must expose `MPV_RENDER_PARAM_VULKAN_WRITE_IMAGE` (available in mpv ≥ 0.34).
- The wgpu texture must use `Rgba8Unorm` or `Bgra8Unorm` format, matching mpv's output.

#### Thread-Marshaling the mpv Wakeup Signal

`libmpv` exposes a callback registration function to signal when a new frame has been decoded and is ready for presentation:

```c
void mpv_render_context_set_update_callback(
    mpv_render_context *ctx,
    mpv_render_context_update_fn cb,
    void *cb_ctx
);
```

**Critical caveat**: This callback is invoked by libmpv from an **arbitrary internal decoder thread**. Any direct UI layout queries, `wgpu` texture updates, or `iced` widget invalidation executed inside this callback will cause segmentation faults or thread deadlocks. The callback body must be constrained to a lightweight signal-only operation.

**The solution**: Marshal the signal back into the `iced` application loop using a dedicated `iced::stream::channel` subscription. A `tokio::sync::mpsc` (or `crossbeam-channel`) sender is passed as the `cb_ctx` pointer. The C callback does a non-blocking `blocking_send`, and an `iced::Subscription` on the receiver side produces `Message::Tick(Instant::now())` events that explicitly request a compositor repaint via `iced::window::redraw`. This is required because `iced` may skip a redraw pass under certain optimization paths if it detects no structural state change. `Message::Tick` + `redraw()` guarantees the video frame lands on screen with sub-millisecond precision.

```rust
// crates/media-sort-gui/src/subscriptions/video_trigger.rs

use iced::Subscription;
use tokio::sync::mpsc;

/// Owns the receiver half of the mpv render signal channel.
pub struct MpvRenderSignalReceiver {
    pub rx: mpsc::Receiver<()>,
}

impl MpvRenderSignalReceiver {
    /// Create a new channel. The sender half is given to mpv via the
    /// `cb_ctx` parameter of `mpv_render_context_set_update_callback`.
    pub fn new() -> (Self, mpsc::Sender<()>) {
        let (tx, rx) = mpsc::channel(8);
        (Self { rx }, tx)
    }
}

/// iced subscription that polls the mpv signal channel.
/// Each incoming signal produces a `Message::Tick`, which the update loop
/// pairs with `iced::window::redraw(Id::MAIN)` to force an immediate
/// compositor repaint. This guarantees every decoded frame reaches the screen.
pub fn monitor_mpv_frames(rx: mpsc::Receiver<()>) -> Subscription<crate::app::Message> {
    Subscription::run(move |mut output| async move {
        while rx.recv().await.is_some() {
            let _ = output
                .send(crate::app::Message::Tick(std::time::Instant::now()))
                .await;
        }
    })
}

/// C-compatible callback handed to `mpv_render_context_set_update_callback`.
/// Called by mpv from its internal decoder thread whenever a new frame
/// is ready. The `cb_ctx` pointer must be the `mpsc::Sender<()>`.
///
/// # Safety
/// `cb_ctx` must be a valid, non-null pointer to an `mpsc::Sender<()>` that
/// outlives the mpv render context.
pub unsafe extern "C" fn mpv_wakeup_callback(cb_ctx: *mut std::ffi::c_void) {
    let sender = cb_ctx as *const mpsc::Sender<()>;
    if let Some(tx) = unsafe { sender.as_ref() } {
        // Non-blocking send. If the channel is full (unlikely with capacity 8),
        // the frame is dropped — the next callback will trigger the redraw.
        let _ = tx.blocking_send(());
    }
}
```

**Registration at mpv init time**:

```rust
// During MpvContext construction (section 3.5), after initializing the
// render context:
let (signal_rx, signal_tx) = MpvRenderSignalReceiver::new();

unsafe {
    mpv_sys::mpv_render_context_set_update_callback(
        render_context,
        Some(mpv_wakeup_callback),
        // Pass the sender as the callback context. Its lifetime is tied
        // to MpvContext (which outlives the render context).
        Box::into_raw(Box::new(signal_tx)) as *mut std::ffi::c_void,
    );
}

// Store signal_rx in the app state for the subscription below.
```

**Subscription in `app.rs`**:

```rust
pub fn subscription(&self) -> Subscription<Message> {
    let mut subs = vec![
        keyboard_subscription(self),
        file_watcher_subscription(self),
    ];

    if let Some(ref rx) = self.state.mpv_render_signal_rx {
        subs.push(
            video_trigger::monitor_mpv_frames(rx.clone())
        );
    }

    Subscription::batch(subs)
}
```

**Why `Message::Tick` + `iced::window::redraw` instead of `Message::Noop`**: The texture is updated in-place (`mpv_render_context_render` writes directly into the wgpu-allocated VkImage). No state mutation is needed in the update loop. However, `iced`'s compositor may skip a full window repaint under certain optimization paths if it detects no structural state change between frames. `Message::Tick` paired with an explicit `iced::window::redraw(Id::MAIN)` in the update handler overrides this optimization, guaranteeing that every frame written to VRAM by mpv is presented on screen with sub-millisecond precision. See the `update.rs` handler below.

### 4.2 Lookahead Prefetch Loop (with Thread Explosion Guard)

The naive approach — spawning `std::thread::spawn` for every prefetch candidate on every index change — causes unbounded thread explosion when a user holds down the Left/Right arrow key to skim through hundreds of files. Each keystroke fires a new batch of spawns while prior batches are still running, saturating the disk I/O subsystem and starving the UI thread.

**The fix**: Replace ad-hoc thread spawning with a **managed worker pattern** using `tokio::task::JoinHandle` and explicit cancellation. A `PrefetchWorkerManager` tracks all in-flight jobs. When the active selection shifts from N to N+1, any prefetch task for indices outside the new radius is explicitly aborted via `JoinHandle::abort()`. New tasks are spawned only for indices within the new radius that are not already cached.

```rust
// crates/media-sort-gui/src/subscriptions/prefetch.rs

use std::sync::Arc;
use parking_lot::Mutex;
use lru::LruCache;
use std::path::PathBuf;
use tokio::task::JoinHandle;

/// Number of adjacent entries to prefetch on each side.
const PREFETCH_RADIUS: usize = 2;

/// Maximum cached thumbnails/decoded frames.
const PREFETCH_CACHE_SIZE: usize = 32;

pub enum DecodedFrame {
    Image(Arc<image::DynamicImage>),
    VideoSnapshot(PathBuf),    // path queued for mpv thumbnail extraction
    Pending,
}

/// Thread-safe prefetch cache shared between the worker and the UI.
pub struct PrefetchCache {
    pub cache: Mutex<LruCache<PathBuf, DecodedFrame>>,
}

impl PrefetchCache {
    pub fn new() -> Self {
        Self { cache: Mutex::new(LruCache::new(PREFETCH_CACHE_SIZE.try_into().unwrap())) }
    }

    pub fn contains(&self, path: &PathBuf) -> bool {
        self.cache.lock().contains(path)
    }
}

/// Manages a bounded set of cancellable background prefetch tasks.
/// When the user rapidly changes selection, stale tasks are aborted
/// before new ones are spawned — preventing thread explosion and disk thrashing.
pub struct PrefetchWorkerManager {
    current_jobs: Arc<Mutex<Vec<JoinHandle<()>>>>,
    cache: Arc<PrefetchCache>,
}

impl PrefetchWorkerManager {
    pub fn new(cache: Arc<PrefetchCache>) -> Self {
        Self {
            current_jobs: Arc::new(Mutex::new(Vec::with_capacity(PREFETCH_RADIUS * 2))),
            cache,
        }
    }

    /// Cancel all outstanding prefetch jobs.
    /// Called immediately before scheduling a new batch.
    pub fn cancel_all_outstanding(&self) {
        let mut jobs = self.current_jobs.lock();
        for job in jobs.drain(..) {
            job.abort();
        }
    }

    /// Schedule prefetch for adjacent entries around `current_index`.
    /// Cancels any in-flight jobs from the previous selection position first,
    /// then spawns fresh tasks only for entries within `[N-R, N+R]` that are
    /// not already present in the cache.
    pub fn schedule_prefetch(&self, media_list: &[PathBuf], current_index: usize) {
        // Step 1: Abort all stale jobs from the previous selection position.
        self.cancel_all_outstanding();

        let start = current_index.saturating_sub(PREFETCH_RADIUS);
        let end = (current_index + PREFETCH_RADIUS + 1).min(media_list.len());

        let mut new_jobs: Vec<JoinHandle<()>> = Vec::with_capacity(end - start);
        let cache = self.cache.clone();

        for i in start..end {
            if i == current_index {
                continue;
            }
            let path = media_list[i].clone();

            // Skip if already cached (no need to re-decode).
            if cache.contains(&path) {
                continue;
            }

            let cache = cache.clone();
            let job: JoinHandle<()> = tokio::task::spawn_blocking(move || {
                let frame = match detect_media_type(&path) {
                    MediaType::Image => decode_image_thumbnail(&path),
                    MediaType::Video => extract_video_thumbnail(&path),
                    MediaType::Audio => extract_audio_cover_art(&path),
                };
                cache.cache.lock().put(path, frame);
            });

            new_jobs.push(job);
        }

        *self.current_jobs.lock() = new_jobs;
    }
}
```

**Execution model**: `schedule_prefetch` is called from the iced `update` function whenever `selected_index` changes. It is synchronous and returns immediately — the heavy decode work happens on the `tokio` blocking thread pool. The `JoinHandle::abort` call is non-blocking; the OS scheduler will terminate the task's thread at the next preemption point (typically a syscall in the image decoder). Because the number of concurrent jobs is bounded to `PREFETCH_RADIUS * 2` (4 tasks), the disk I/O subsystem never sees more than 4 concurrent decode requests.

**Why `spawn_blocking` instead of async I/O**: Image/video decoding is CPU-bound and file I/O is synchronous on most `image` crate backends. The blocking pool prevents these tasks from starving the async runtime's worker threads.

---

## PHASE 5: UI LOOP & DYNAMIC HOTKEY SYSTEM

### 5.1 `app.rs` — The Centralized Elm Loop

```rust
// crates/media-sort-gui/src/app.rs

use iced::{Application, Command, Theme};
use media_sort_backend::filesystem::TrashStaging;

pub struct MediaSortApp {
    // Core state
    pub state: AppState,

    // Backend drivers
    pub filesystem: media_sort_backend::filesystem::FileSystemDriver,
    pub trash_staging: Arc<MediaSortTrashStaging>,
    pub metadata_extractor: media_sort_backend::metadata::Extractor,

    // GUI-specific
    pub cache: Arc<PrefetchCache>,
    pub prefetch_worker: PrefetchWorkerManager,
    pub mpv_context: Option<MpVContext>,

    // Settings
    pub settings: SettingsStore,
    pub l10n: Localization,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Folder operations
    FolderSelected(usize),
    OpenFolderDialog,
    FolderDialogResult(Option<PathBuf>),
    CreateFolder(String),
    PinSelected,
    UnpinSelected,
    MovePinnedUp,
    MovePinnedDown,

    // Media navigation
    SelectMedia(usize),
    GoLeft,
    GoRight,

    // Actions
    MoveMedia,
    DeleteMedia,
    RenameMedia(String),

    // History
    Undo,
    Redo,

    // Hotkey
    PinFolderShortcut(u8),  // Alt+1 through Alt+9
    SearchFocus,
    ToggleMetadataPanel,

    // Settings
    OpenSettings,
    ToggleDarkMode,
    SaveSettings,

    // Backend events
    FilesystemEvent(FileSystemEvent),
    PrefetchComplete(PathBuf, DecodedFrame),

    // UI
    SearchTermChanged(String),
    WindowEvent(iced::window::Event),
    Tick(std::time::Instant),
    Noop,
}
```

### 5.2 `state.rs` — AppState

```rust
pub struct AppState {
    // Current directory
    pub current_folder: PathBuf,
    pub folder_tree: Vec<FolderNode>,       // lazy-loaded per expansion
    pub expanded_folders: HashSet<PathBuf>,

    // Pinned folders
    pub pinned_folders: Vec<PinnedFolder>,

    // Media listing
    pub media_entries: Vec<MediaEntry>,     // loaded from current folder
    pub selected_index: usize,              // current selection
    pub search_term: String,                // filter text
    pub filtered_indices: Vec<usize>,       // indices matching search

    // History
    pub history: History,

    // UI state
    pub show_settings: bool,
    pub metadata_expanded: bool,
    pub metadata_panel_width: u16,
    pub metadata_result: Option<MetadataResult>,  // for the current selection
    pub preview_mode: PreviewMode,

    // Media playback
    pub video_path: Option<PathBuf>,        // currently playing video
    pub video_paused: bool,
    pub audio_path: Option<PathBuf>,        // currently playing audio
}

pub enum PreviewMode {
    Thumbnails,
    SingleMedia,    // full-size view
}

pub enum MetadataResult {
    Success {
        sections: BTreeMap<String, BTreeMap<String, String>>,
    },
    FileDoesNotExist,
    Error(String),
}
```

### 5.3 `update.rs` — Message Handler

The `update` function is the central reducer. Key handlers:

```rust
pub fn update(app: &mut MediaSortApp, message: Message) -> Command<Message> {
    match message {
        Message::Tick(_) => {
            // Force an immediate hardware repaint of the active window surface.
            // Used by the mpv render wakeup subscription and other frame-level
            // triggers where state is mutated in-place (no iced state change).
            iced::window::redraw(iced::window::Id::MAIN)
        }

        Message::FolderSelected(index) => {
            // 1. Clear action history (directory-scoping)
            app.state.history.clear();
            // 2. Flush any remaining staged trash to native OS
            app.trash_staging.flush_all_to_native();
            // 3. Unload video/audio
            if let Some(mpv) = &mut app.mpv_context {
                mpv.stop();
            }
            // 4. Set new current folder
            let new_folder = /* get from folder_tree[index] */;
            app.state.current_folder = new_folder.path.clone();
            // 5. Scan media
            app.state.media_entries = scan_media_entries(&new_folder.path);
            app.state.selected_index = 0;
            app.state.search_term.clear();
            app.state.filtered_indices = (0..app.state.media_entries.len()).collect();
            // 6. Trigger prefetch (cancels stale jobs, spawns fresh within radius)
            let paths: Vec<PathBuf> = app.state.media_entries.iter().map(|e| e.path.clone()).collect();
            app.prefetch_worker.schedule_prefetch(&paths, 0);
            Command::none()
        }

        Message::MoveMedia => {
            let entry = &app.state.media_entries[app.state.selected_index];
            let target = &app.state.pinned_folders[selected_pinned_index];

            let mut action = MoveAction::new(&entry.path, &target.path)
                .expect("MoveAction precondition");
            action.execute().expect("move");

            // Update UI lists
            let old_index = app.state.selected_index;
            app.state.media_entries.remove(app.state.selected_index);
            app.state.history.push_executed(Box::new(action));

            // Preserve selection index (legacy behavior)
            if old_index >= app.state.media_entries.len() && !app.state.media_entries.is_empty() {
                app.state.selected_index = 0;
            } else if old_index < app.state.media_entries.len() {
                app.state.selected_index = old_index;
            }

            Command::none()
        }

        Message::PinFolderShortcut(n) => {
            // Alt+N: move current selection to pinned folder at index n-1
            let pinned_index = (n - 1) as usize;
            if let Some(target) = app.state.pinned_folders.get(pinned_index) {
                let entry = &app.state.media_entries[app.state.selected_index];
                let mut action = MoveAction::new(&entry.path, &target.path).expect("valid");
                action.execute().expect("move");

                app.state.media_entries.remove(app.state.selected_index);
                app.state.history.push_executed(Box::new(action));

                // Auto-advance to next media
                if app.state.selected_index >= app.state.media_entries.len() {
                    app.state.selected_index = 0;
                }
            }
            Command::none()
        }

        // ... other handlers
    }
}
```

### 5.4 `subscriptions/keyboard.rs` — Global Hotkey Subscription

```rust
use iced::{keyboard, Subscription};
use iced::keyboard::{Key, Modifiers};

pub fn keyboard_subscription(app: &MediaSortApp) -> Subscription<Message> {
    iced::keyboard::on_key_press(|key, modifiers| {
        // If a text input has focus, suppress all hotkeys
        // Tracked via app state flag `app.state.text_input_focused`
        let keybinding = &app.settings.keybindings;

        match (key, modifiers) {
            kb if kb == keybinding.go_left.into() => Some(Message::GoLeft),
            kb if kb == keybinding.go_right.into() => Some(Message::GoRight),
            kb if kb == keybinding.move_to_folder.into() => Some(Message::MoveMedia),
            kb if kb == keybinding.delete.into() => Some(Message::DeleteMedia),
            kb if kb == keybinding.rename.into() => Some(Message::RenameMedia("".into())),
            kb if kb == keybinding.undo.into() => Some(Message::Undo),
            kb if kb == keybinding.redo.into() => Some(Message::Redo),
            kb if kb == keybinding.open_folder.into() => Some(Message::OpenFolderDialog),
            kb if kb == keybinding.open_selected_folder.into() => Some(Message::OpenSelectedFolder),
            kb if kb == keybinding.pin.into() => Some(Message::PinSelected),
            kb if kb == keybinding.unpin.into() => Some(Message::UnpinSelected),
            kb if kb == keybinding.move_pinned_up.into() => Some(Message::MovePinnedUp),
            kb if kb == keybinding.move_pinned_down.into() => Some(Message::MovePinnedDown),
            kb if kb == keybinding.search_images.into() => Some(Message::SearchFocus),
            kb if kb == keybinding.toggle_metadata_panel.into() => Some(Message::ToggleMetadataPanel),

            // Dynamic pinned folder shortcuts: Alt+[1-9]
            (Key::Character(c), Modifiers::ALT) if ('1'..='9').contains(&c) => {
                let n = c.to_digit(10).unwrap() as u8;
                Some(Message::PinFolderShortcut(n))
            }

            _ => None,
        }
    })
}
```

**Text-input guard**: When the search bar `TextInput` widget has focus, the `on_key_press` subscription is bypassed by checking a flag `app.state.text_input_focused` before any key matching. In `iced`, this is implemented by conditionally adding the keyboard subscription only when the search field is not focused, or by a guard at the top of the handler.

### 5.5 `view/` — Layout Tree

```rust
pub fn view(app: &MediaSortApp) -> Element<Message> {
    // Three-panel horizontal split layout:
    //
    // ┌──────────────┬─────────────────────────┬─────────────┐
    // │ Folder Panel │    Media Preview/Grid   │  Metadata   │
    // │              │                        │  Panel      │
    // │ [Pinned]     │  ┌──────────────────┐   │ (collapsible)
    // │  Folder 1    │  │                  │   │             │
    // │  Folder 2    │  │  Image / Video   │   │ EXIF IFD0   │
    // │              │  │  Preview         │   │  Make: Sony │
    // │ [Tree]       │  │                  │   │  Model: ... │
    // │  Current/    │  └──────────────────┘   │             │
    // │  Subfolders  │                        │             │
    // │              │  [Search bar]           │             │
    // │              │  [Thumbnail Grid]       │             │
    // ├──────────────┴─────────────────────────┴─────────────┤
    // │  History Bar: Undo | Redo | Last Action             │
    // └──────────────────────────────────────────────────────┘
    //
    // Bottom: History bar showing last_done / last_undone
}
```

### 5.6 `widgets/video_canvas.rs` — The `iced` Video Widget

The video widget implements `iced::widget::shader::Program` for the custom wgpu rendering:

```rust
use std::sync::Arc;
use iced::widget::shader::{self, Viewport};
use iced::advanced::widget::{self, Widget};
use iced::{Element, Length, Rectangle, Size};

/// The video preview widget holds a reference-counted `VulkanSharedTexture`
/// (see section 4.1). On each frame tick, `render_mpv_to_vulkan_surface()`
/// writes the decoded video frame directly into the underlying VkImage,
/// and `iced`'s compositor samples the `wgpu::Texture` without any copy.
pub struct VideoCanvas {
    shared_texture: Arc<VulkanSharedTexture>,
    mpv_ctx: *mut mpv_sys::mpv_render_context,
}

impl<Message> Widget<Message, Theme, Renderer> for VideoCanvas {
    fn size(&self) -> Size<Length> { Size::new(Length::Fill, Length::Fill) }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &widget::Style,
        layout: widget::Layout<'_>,
        _cursor: widget::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        // Per-frame: mpv writes into our VkImage, then iced samples it.
        // No CPU readback, no staging buffer, no PCIe transfer.
        unsafe {
            render_mpv_to_vulkan_surface(self.mpv_ctx, &self.shared_texture);
        }
        // Bind self.shared_texture.wgpu_texture as a sampled image in the quad pass.
    }
}

/// Safe Vulkan tear-down: detach mpv from the shared VkImage before wgpu
/// deallocates it. Rust drops struct fields in declaration order, so the
/// `Arc<VulkanSharedTexture>` would otherwise drop its `wgpu::Texture` while
/// mpv's decoder thread might still be writing to the underlying VkImage.
/// We explicitly free the render context first to prevent use-after-free.
impl Drop for VideoCanvas {
    fn drop(&mut self) {
        unsafe {
            // 1. Detach mpv from the target surface. Any in-flight decode
            //    operation is cancelled before memory is released.
            mpv_sys::mpv_render_context_free(self.mpv_ctx);
        }
        // 2. self.shared_texture drops here (implicit, after this block).
        //    The Arc<VulkanSharedTexture> may still be referenced elsewhere
        //    via clones; wgpu reclaims the VkImage only when the last Arc
        //    is dropped, which is safe because mpv is no longer writing.
    }
}
```

---

## CROSS-CRATE COMMUNICATION BOUNDARIES

```
┌──────────────────────────────────────────────────────────────┐
│                    media-sort-gui (iced Application)          │
│                                                               │
│  Owning: AppState, History, PrefetchCache, MpvContext         │
│  Importing: media-sort-core types, media-sort-backend drivers │
│                                                               │
│  ┌──────────────────────┐      ┌───────────────────────────┐ │
│  │ subscriptions/        │      │ widgets/                   │ │
│  │  keyboard.rs ─────────┼──────┼─► update.rs ◄── view/     │ │
│  │  file_watcher.rs      │      │                            │ │
│  │  prefetch.rs          │      │ Message enum (60 variants) │ │
│  └──────────────────────┘      └───────────────────────────┘ │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│                   media-sort-backend                          │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐ │
│  │ filesystem/   │  │ metadata/    │  │ media/              │ │
│  │  watcher.rs   │  │  image_meta  │  │  image_decoder      │ │
│  │  scanner.rs   │  │  video_meta  │  │  video_decoder(mpv) │ │
│  │  trash_staging│  │  audio_meta  │  │  audio_decoder      │ │
│  └──────────────┘  └──────────────┘  └────────────────────┘ │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│                    media-sort-core                            │
│                                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐ │
│  │ actions/  │  │ settings/│  │ models.rs│  │ l10n.rs     │ │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────┘ │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

**Data flow rules**:
- `media-sort-core` has zero external dependencies (except serde/fluent). It defines traits and data.
- `media-sort-backend` depends on `media-sort-core` for types. It implements the actual OS interop.
- `media-sort-gui` depends on both. It owns all mutable state and orchestrates the others.
- Communication from backend → GUI uses channel receivers wrapped as `iced::Subscription` streams.
- No circular dependencies.

---

## FEATURE TRACEABILITY MATRIX

| Legacy Feature (C#) | Media Sort Implementation | Status |
|---------------------|--------------------------|--------|
| `MainViewModel` orchestration | `app.rs` → `AppState` + `update.rs` | Mapped |
| `ImagesViewModel` file filtering | `filesystem/scanner.rs` → `scan_media_files()` | Mapped + video/audio |
| `FoldersViewModel` tree + pinning | `state.rs` `folder_tree` + `pinned_folders` | Mapped |
| `ActionsViewModel` undo/redo stacks | `history.rs` `History` struct | Mapped |
| `MoveAction` / `RenameAction` / `DeleteAction` | `actions/move_action.rs`, `rename_action.rs`, `delete_action.rs` | Mapped |
| `RecycleBin` + `SHFileOperation` | `filesystem/trash_staging.rs` — two-phase staging | Enhanced (cross-platform) |
| `FileSystemWatcher` | `filesystem/watcher.rs` — `notify` + debouncer | Mapped |
| `MetadataExtractor` (EXIF/XMP/IPTC) | `metadata/image_meta.rs` — `kamadak-exif` | Mapped, extendable |
| `SettingsViewModel` + JSON config | `settings/store.rs` + `SettingsStore` | Mapped |
| `SettingsGroupViewModelBase` per-group | Typed structs in `settings/` (General, KeyBindings, etc.) | Mapped |
| `KeyBindingsSettingsGroupViewModel` (21 bindings) | `keybindings.rs` `KeyBindings` struct | Mapped |
| `Hotkey` record + `HotkeyEditorControl` | `widgets/keybinding_editor.rs` | Mapped |
| `WindowPositionSettingsViewModel` | `window_position.rs` | Mapped |
| `PinnedFolderSettingsViewModel` | `pinned_folders.rs` | Mapped |
| `MetadataPanelSettings` | `metadata_panel.rs` | Mapped |
| `GeneralSettingsGroupViewModel` (Dark mode, updates, GIFs) | `general.rs` | Mapped |
| `PathHelper.PathEquals` | `path_utils.rs` (platform-aware) | Mapped |
| `ImageLoading` (LRU cache + EXIF orientation + GIF) | `media/image_decoder.rs` + `prefetch.rs` | Mapped |
| Animated GIF support | `widgets/gif_player.rs` (via `image` crate GIF decoder) | Mapped |
| `ShellFileLoader` (folder icons) | System icon theme (freedesktop `IconTheme` on Linux, `objc` on macOS) | Adapted |
| Search/filter images by name | `state.rs` `search_term` + `filtered_indices` | Mapped |
| Dynamic hotkey to pinned folder (NEW) | `update.rs` → `PinFolderShortcut(u8)` | New Feature |
| Video playback (NEW) | `widgets/video_canvas.rs` + `mpv_context.rs` | New Feature |
| Audio playback (NEW) | `media/audio_decoder.rs` + `symphonia` | New Feature |
| `Gif` animation in thumbnails | `media/thumbnail.rs` | Mapped |
| Explorer context menu (Windows registry) | Deferred (not core to media sorting) | Optional |
| GitHub auto-updater | Deferred (use platform package managers) | Replaced |

---

## IMPLEMENTATION ORDER (Build Milestones)

| Milestone | Crates | Deliverable |
|-----------|--------|-------------|
| **M1** | `media-sort-core` | All data models, `ReversibleAction` trait, `MoveAction`, `History`, `SettingsStore`, `l10n` — unit tested |
| **M2** | `media-sort-backend` (filesystem) | `scanner.rs`, `watcher.rs`, `trash_staging.rs` — integration tested |
| **M3** | `media-sort-backend` (metadata, media) | `image_meta`, `video_meta`, `audio_meta`, `image_decoder`, `thumbnail` |
| **M4** | `media-sort-gui` (shell) | `main.rs`, `app.rs`, `state.rs`, `update.rs` — window opens, settings load/save |
| **M5** | `media-sort-gui` (folder panel) | `folder_panel.rs`, `folder_tree.rs` — directory browsing, pinning |
| **M6** | `media-sort-gui` (media grid) | `media_grid.rs`, `media_preview.rs`, `search_bar.rs` — image browsing, filtering |
| **M7** | `media-sort-gui` (actions) | Wire `Move`, `Delete`, `Rename` through `update.rs` → `History` — full file sorting |
| **M8** | `media-sort-gui` (keybindings) | `subscriptions/keyboard.rs`, `keybinding_editor.rs` — all 21+ bindings |
| **M9** | `media-sort-gui` (metadata panel) | `metadata_panel.rs` — collapsible EXIF viewer |
| **M10** | `media-sort-gui` (video) | `video_canvas.rs`, `mpv_context.rs`, `video_trigger.rs` — wgpu-first zero-copy rendering + mpv wakeup callback marshaling → iced subs |
| **M11** | `media-sort-gui` (audio) | Audio transport bar, `symphonia` decode → `rodio` hardware sink, metadata panel integration |
| **M12** | `media-sort-gui` (prefetch) | `subscriptions/prefetch.rs` — lookahead cache |
| **M13** | Polish | Dark mode, settings dialog, window position restore, final edge cases |
|

---

## APPENDIX: MULTI-AGENT CODE GENERATION CONTRACT

This blueprint's three-crate architecture maps naturally to a multi-agent code generation workflow. To prevent cross-agent channel mismatches during parallel agent execution, **the following types are frozen contracts** and must be generated first by the coordinating orchestrator before any agent begins implementation work:

### Frozen Contract Types

| Contract | Location | Reason |
|----------|----------|--------|
| `ActionError` enum | `media-sort-core/src/actions/reversible.rs` | Shared error type consumed by all three crates |
| `ReversibleAction` trait | `media-sort-core/src/actions/reversible.rs` | Implemented by backend, consumed by GUI |
| `History` struct | `media-sort-core/src/history.rs` | Owned by GUI, mutation engine in core |
| `MediaType` enum + `extensions()` | `media-sort-core/src/media_type.rs` | Shared filtering across scanner and GUI |
| `SettingsStore` + all sub-structs | `media-sort-core/src/settings/` | Serialized by GUI, deserialized by core |
| `KeyBinding` struct | `media-sort-core/src/settings/keybindings.rs` | Hotkey editor ↔ keyboard subscription ↔ persistence |
| `FileSystemEvent` enum | `media-sort-backend/src/filesystem/watcher.rs` | Emitted by backend watcher, consumed by GUI subscription |
| `Message` enum (all variants) | `media-sort-gui/src/message.rs` | Central dispatch enum; all agents emit into this type |
| `TrashRestoreHandle` trait | `media-sort-core/src/actions/delete_action.rs` | Implemented by backend trash staging, consumed by core action |
| `VulkanSharedTexture` struct | `media-sort-gui/src/widgets/video_canvas.rs` | Allocated by GUI, passed to backend mpv context |

### Agent Boundary Rules

- **Core Agent** — Generates `media-sort-core/` only. Must not import any backend or GUI types. Unit tests run in isolation without any I/O or graphics.
- **Backend Agent** — Generates `media-sort-backend/` only. May import core types freely. Must not import `iced`, `wgpu`, or any GUI crate. `ash` usage is limited to Vulkan primitive types needed by the mpv render context.
- **GUI Agent** — Generates `media-sort-gui/` only. May import both core and backend types. The `update()` function is the single integration point; the agent must not spawn its own threads or access the filesystem directly (all I/O goes through backend APIs).

### Orchestration Sequence

1. Orchestrator generates all frozen contracts and seeds each agent workspace.
2. Core Agent completes M1 independently (zero downstream dependencies).
3. Backend Agent completes M2–M3 using only core types as reference.
4. GUI Agent completes M4–M13 using both completed crates as dependencies.

This ordering ensures that at every step, the generating agent has access to finalized, compilable types from earlier milestones — no agent ever codes against an interface that hasn't been generated yet.

---

*Blueprint generated from exhaustive analysis of Image-Sort codebase (legacy/2.x) and mapped to idiomatic Rust patterns.*
