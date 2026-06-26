# Media Sort — Technical Specification & Requirements Traceability

> Extracted from [Image-Sort](https://github.com/Lolle2000la/Image-Sort) (C# WPF, .NET 8.0, ReactiveUI, AdonisUI)  
> Target: Rust + `iced` + `wgpu` ground-up rewrite.  
> Scope extension: images → images, video, audio.

---

## 1. ARCHITECTURAL PATTERN & CORE EXECUTION FLOW

### 1.1 MVVM via ReactiveUI — Anatomy

The application uses **ReactiveUI** (a Reactive Extensions–based MVVM framework). All ViewModels inherit from `ReactiveObject`. Properties use `this.RaiseAndSetIfChanged(ref field, value)` which is equivalent to `INotifyPropertyChanged`. Commands are `ReactiveCommand<TIn, TOut>` with `canExecute` observables. Reactive pipelines are built with `this.WhenAnyValue(...)`, `ObservableAsPropertyHelper<T>`, and `SourceList<T>` (from DynamicData) for reactive collections.

| Layer | Project | Role |
|-------|---------|------|
| **Core ViewModels** | `ImageSort` | All business logic, zero WPF dependency |
| **WPF UI** | `ImageSort.WPF` | XAML views, platform interop, converters, styles |
| **Localization** | `ImageSort.Localization` | RESX-based string resources |
| **Dependency Injection** | `Splat` (via `Locator.Current`) | Service locator pattern |

**Source:** `src/ImageSort/ViewModels/MainViewModel.cs:13`  
**Source:** `src/ImageSort.WPF/App.xaml.cs:43-53` (DI registration)

### 1.2 MainViewModel State Lifecycle

`MainViewModel` is the **top-level orchestrator**. It composes four child ViewModels:

| Property | Type | Source |
|----------|------|--------|
| `Folders` | `FoldersViewModel` | `MainViewModel.cs:23-28` |
| `Images` | `ImagesViewModel` | `MainViewModel.cs:31-36` |
| `Actions` | `ActionsViewModel` | `MainViewModel.cs:15-20` |
| `PickFolder` | `Interaction<Unit, string>` | `MainViewModel.cs:39` |

**Startup** (`MainWindow.xaml.cs:40-53`):
1. `MainWindow` constructor instantiates `MainViewModel` directly.
2. `FoldersViewModel.CurrentFolder` is seeded from `args[1]` or `Environment.SpecialFolder.MyPictures`.
3. `ImagesViewModel` and `ActionsViewModel` are created empty.
4. On `Activated`, the view registers all key-binding handlers and command bindings.

**Lifecycle events:**
- `Closed` → `settings.SaveAsync()` (persists config to JSON).
- `OnStartup` → settings restores, then (if enabled) checks GitHub for updates via `Octokit`.
- `MainViewModel` has no explicit dispose — child ViewModels use `~Finalizer` (destructor) for cleanup of `FileSystemWatcher` and `SourceList`.

**Source:** `src/ImageSort.WPF/App.xaml.cs:36-63`  
**Source:** `src/ImageSort.WPF/MainWindow.xaml.cs:37-59`

### 1.3 Cross-ViewModel Reactive Synchronization

**Channel 1: CurrentFolder → Images list**  
`MainViewModel.cs:54-65`:
```
Images (when not null) subscribes to Folders.CurrentFolder.Path
  → sets Images.CurrentFolder
  → Images loads files from that path (filtered by extension)
```

**Channel 2: CurrentFolder change → Clear action history**  
`MainViewModel.cs:123-128`:
```
When Folders.CurrentFolder changes → Actions.Clear.Execute()
```
Rationale: Undo/redo stacks are directory-scoped. Moving to a new folder invalidates old actions.

**Channel 3: Images.RenameImage → Actions.Execute**  
`MainViewModel.cs:129-133`:
```
Images.RenameImage (whose output is IReversibleAction?)
  → where not null
  → Actions.Execute.Execute(action)
```
Rename produces an `IReversibleAction` that gets pushed onto the undo stack.

**Channel 4: Folder tree IsCurrentFolder tracking**  
`FoldersViewModel.cs:159-171`:
```
When CurrentFolder changes:
  → oldFolder.IsCurrentFolder = false
  → newFolder.IsCurrentFolder = true
```

**Channel 5: Metadata — ImagePath binding**  
In the XAML view layer, `Images.SelectedImage` is bound to `MetadataViewModel.ImagePath`. When it changes, metadata is extracted reactively.

**Source:** `src/ImageSort/ViewModels/MainViewModel.cs:54-133`

### 1.4 Pipeline: User Changes Active Folder

1. **Trigger**: `OpenCurrentlySelectedFolder.Execute()` or `OpenFolder.Execute()` (“O” key, or Enter on folder tree).
2. `FoldersViewModel.CurrentFolder` is set to a new `FolderTreeItemViewModel`.
3. **Reactive cascade**:
   a. `MainViewModel` sets `Images.CurrentFolder` to the new path.
   b. `ImagesViewModel` calls `fileSystem.GetFiles(folder)`, filters by supported extensions, then `images.Clear()` + `images.AddRange(...)`.
   c. The `SourceList<string> images` is filtered by `SearchTerm` and sorted ascending, bound to `ReadOnlyObservableCollection<string> Images`.
   d. `SelectedIndex` is reset to 0 (via a hack: `if (SelectedIndex == 0) SelectedIndex = -1; … = 0`).
   e. `ImagesView` re-renders the image list and preloads the selected image.
   f. `ActionsViewModel.Clear` is called, flushing undo/redo stacks.
4. **FileSystemWatcher** is recreated for the new folder (`ImagesViewModel.cs:166-180`).

**Source:** `src/ImageSort/ViewModels/ImagesViewModel.cs:80-104`

### 1.5 Pipeline: Advancing to Next Image (GoLeft/GoRight)

1. **Trigger**: Left/Right arrow key (or configured hotkey).  
   `ImagesViewModel.GoRight` → `SelectedIndex++` (or `GoLeft` → `SelectedIndex--`).
2. `SelectedIndex` change is observed via `WhenAnyValue(x => x.SelectedIndex)` → `Images.ElementAtOrDefault(i)` is projected to `SelectedImage` (OAPH).
3. The view's image control binds to `SelectedImage` and loads the image via `PathToBitmapImageConverter` → `ImageLoading.GetImageFromPath()`.
4. `GetImageFromPath` uses `LazyCache` (size-limited to 20 entries, each entry = 1 unit) with `BitmapCacheOption.OnLoad`. On cache miss, it creates a `BitmapImage` from the URI, checks for EXIF orientation via `System.Photo.Orientation` WIC query, and rotates accordingly. On failure, it renders a `DrawingImage` with the error text.

**Source:** `src/ImageSort/ViewModels/ImagesViewModel.cs:106-119`  
**Source:** `src/ImageSort.WPF/FileSystem/ImageLoading.cs:30-79`

### 1.6 Error Handling Patterns

- **Inaccessible directories**: Caught with `catch {}` (general exception) in `FolderTreeItemViewModel.cs:101-107` — silently skipped, returns `null`, filtered out.
- **Inaccessible files**: Same pattern — `UnauthorizedAccessException` caught during folder tree construction (`FolderTreeItemViewModel.cs:119`).
- **Corrupt images**: `ImageLoading.cs:52-77` catches all exceptions in image loading, renders error text in place of the image.
- **Action failures**: `ActionsViewModel.cs:31-44` catches exceptions in `Act()` and notifies user via `NotifyUserOfError` Interaction.
- **FileSystemWatcher errors**: Caught with empty `catch {}` — the watcher simply doesn't work; the app doesn't crash (`FolderTreeItemViewModel.cs:157-162`).
- **UnhandledInteractionException**: When user cancels a folder picker dialog, `UnhandledInteractionException<Unit, string>` is caught silently (`MainViewModel.cs:84`, `FoldersViewModel.cs:94-98`).

**Source:** `src/ImageSort/ViewModels/FolderTreeItemViewModel.cs:96-124`  
**Source:** `src/ImageSort/ViewModels/ActionsViewModel.cs:31-49`

---

## 2. COMPREHENSIVE FUNCTIONAL REQUIREMENTS LIST

### 2.1 Folder Browsing & Navigation (`FoldersViewModel` + `FolderTreeItemViewModel`)

| ID | Requirement | Source |
|----|-------------|--------|
| FR-F-01 | Display a tree of sub-folders under the current path, sorted alphabetically | `FolderTreeItemViewModel.cs:70-77` |
| FR-F-02 | Load sub-folders lazily — only when `IsVisible` is true (virtualized tree) | `FolderTreeItemViewModel.cs:89-92` |
| FR-F-03 | Handle inaccessible folders gracefully (silently skip) | `FolderTreeItemViewModel.cs:101-107` |
| FR-F-04 | Show folder name (last path segment) for each tree node; for drive roots (e.g. `C:\`), show the full path | `FolderTreeItemViewModel.cs:80-87` |
| FR-F-05 | Mark the currently open folder with `IsCurrentFolder = true` | `FoldersViewModel.cs:159-171` |
| FR-F-06 | Create a new sub-folder under the selected folder; check for name collision first | `FolderTreeItemViewModel.cs:127-138` |
| FR-F-07 | Prompt user for folder name via `Interaction<Unit, string>` | `FoldersViewModel.cs:150-157` |
| FR-F-08 | Track `AllFoldersTracked` = `CurrentFolder` ∪ `PinnedFolders` (used for hotkey conflict detection?) | `FoldersViewModel.cs:75-78` |
| FR-F-09 | Open a folder picked via native OS dialog | `FoldersViewModel.cs:78-99` |
| FR-F-10 | Open the currently selected folder as the active folder | `FoldersViewModel.cs:73-76` (command), `MainViewModel.cs:73-76` |
| FR-F-11 | WASD-style folder tree navigation (Up/Left/Down/Right arrow key simulation) | `MainWindow.xaml.cs:162-174` |

### 2.2 Pinned Folders Management (`FoldersViewModel`)

| ID | Requirement | Source |
|----|-------------|--------|
| FR-PF-01 | Pin a folder by picking from native OS dialog; prevent duplicates | `FoldersViewModel.cs:80-98` |
| FR-PF-02 | Pin the currently selected folder | `FoldersViewModel.cs:106-109` |
| FR-PF-03 | Unpin a selected pinned folder (remove from list, not delete on disk) | `FoldersViewModel.cs:116-121` |
| FR-PF-04 | Reorder pinned folders up/down by one position | `FoldersViewModel.cs:127-144` |
| FR-PF-05 | Persist pinned folder paths to settings JSON; restore on startup | `PinnedFolderSettingsViewModel.cs:8-22`, `App.xaml.cs:48-49` |
| FR-PF-06 | Restore pinned folders from persisted paths; validate directory exists; skip invalid | `FoldersViewModel.cs:174-186` |
| FR-PF-07 | Guard against pinning the same folder twice | `FoldersViewModel.cs:87` |

### 2.3 Image Browsing & Selection (`ImagesViewModel`)

| ID | Requirement | Source |
|----|-------------|--------|
| FR-I-01 | Enumerate image files in current folder, filtered by extension: `.png`, `.jpg`, `.jpeg`, `.gif`, `.bmp`, `.tiff`, `.tif`, `.ico`, `.webp` | `ImagesViewModel.cs:21` |
| FR-I-02 | Display images as a sorted list (ascending by path) | `ImagesViewModel.cs:73-78` |
| FR-I-03 | Select an image by index; expose selected image path via OAPH | `ImagesViewModel.cs:37-46` |
| FR-I-04 | Navigate selection left (decrement) and right (increment) with boundary guards | `ImagesViewModel.cs:106-120` |
| FR-I-05 | After a move/delete, preserve selection index if possible; otherwise reset to 0 | `MainViewModel.cs:97-103`, `MainViewModel.cs:119-121` |
| FR-I-06 | Search/filter images by name substring (case-insensitive, `OrdinalIgnoreCase`) | `ImagesViewModel.cs:73-76` |
| FR-I-07 | `SearchTerm` watermark: "Search images... (press 'Tab' to leave)" | `Text.resx` → `SearchTermWatermark` |
| FR-I-08 | Focus the search box via hotkey 'i' | `MainWindow.xaml.cs:207-209` |

### 2.4 Image Display & Caching

| ID | Requirement | Source |
|----|-------------|--------|
| FR-D-01 | Load and display images from file path; cache up to 20 images in memory (size-limited LRU via `LazyCache` + `MemoryCache`) | `ImageLoading.cs:17-28` |
| FR-D-02 | Respect EXIF orientation tag (`System.Photo.Orientation`): rotate 0°, 90°, 180°, 270° accordingly | `ImageLoading.cs:83-121` |
| FR-D-03 | Show error text in place of corrupted/unreadable images | `ImageLoading.cs:52-77` |
| FR-D-04 | Reject images with 0 width or height (throw `BadImageFormatException`) | `ImageLoading.cs:47-48` |
| FR-D-05 | Display file name only (not full path) in the image list via `PathToFilenameConverter` | `PathToFilenameConverter.cs:11-18` |
| FR-D-06 | Support animated GIFs; configurable via settings (`AnimateGifs`, `AnimateGifThumbnails`) | `GeneralSettingsGroupViewModel.cs:41-56` |
| FR-D-07 | Animated GIF setting changes do not affect already loaded images | `Text.resx` → `AnimatedGifsSettingsChangeNotice` |

### 2.5 Actions: Move, Delete, Rename

| ID | Requirement | Source |
|----|-------------|--------|
| FR-A-01 | Move the selected image to the selected folder (in folder tree) | `MainViewModel.cs:92-103` |
| FR-A-02 | Delete the selected image (send to OS recycle bin) | `MainViewModel.cs:110-121` |
| FR-A-03 | Rename the selected image with user-provided new name; preserve file extension automatically | `ImagesViewModel.cs:125-160` |
| FR-A-04 | Validate rename inputs: reject `\ / * ? : < > \| "` and `Path.GetInvalidPathChars()` | `ImagesViewModel.cs:131-140` |
| FR-A-05 | On rename collision (target exists), throw `IOException` with localized message | `RenameAction.cs:27-31` |

### 2.6 Settings Management (`SettingsViewModel` + Groups)

**Settings Group Model** (`SettingsGroupViewModelBase`):
- Each group has a unique `Name`, a `Header` (for display), an `IsVisible` flag, and a `Dictionary<string, object> SettingsStore`.
- Reactive: any property change via `RaiseAndSetIfChanged` is automatically persisted to `SettingsStore` via reflection.
- Serialization: the entire settings tree is serialized to JSON at `%APPDATA%/Image Sort/config.json` (or `debug_config.json` in debug builds).
- Deserialization: JSON values are parsed back via `JsonElementToValue`, which handles `bool`, `string`, `int`, `object[]` (string arrays), and `Hotkey` (JSON object with `Key` and `Modifiers`).
- During UI tests (`UI_TEST` env var), config is saved to `ui_test_config.json`.

**Source:** `src/ImageSort/SettingsManagement/SettingsGroupViewModelBase.cs:8-44`  
**Source:** `src/ImageSort.WPF/SettingsManagement/SettingsHelper.cs:15-80`

#### 2.6.1 General Settings Group

| ID | Setting | Type | Default | Source |
|----|---------|------|---------|--------|
| FR-SG-01 | `DarkMode` | `bool` | `false` | `GeneralSettingsGroupViewModel.cs:17-23` |
| FR-SG-02 | `CheckForUpdatesOnStartup` | `bool` | `true` | `GeneralSettingsGroupViewModel.cs:25-31` |
| FR-SG-03 | `InstallPrereleaseBuilds` | `bool` | `false` | `GeneralSettingsGroupViewModel.cs:33-39` |
| FR-SG-04 | `AnimateGifs` | `bool` | `true` | `GeneralSettingsGroupViewModel.cs:41-47` |
| FR-SG-05 | `AnimateGifThumbnails` | `bool` | `true` | `GeneralSettingsGroupViewModel.cs:49-55` |
| FR-SG-06 | `ShowInExplorerContextMenu` | `bool` | auto-detected from registry | `GeneralSettingsGroupViewModel.cs:57-63` |

**Dark mode** toggles `AdonisUI` color scheme immediately.  
**Explorer context menu** writes/removes registry keys under `HKCU\Software\Classes\{Directory,Drive,Folder}\shell\ImageSort` with the command `"Image Sort.exe" "%L"`.

**Source:** `src/ImageSort.WPF/SettingsManagement/GeneralSettingsGroupViewModel.cs:67-127`

#### 2.6.2 Pinned Folder Settings

| ID | Setting | Type | Default | Source |
|----|---------|------|---------|--------|
| FR-SP-01 | `PinnedFolders` | `IEnumerable<string>` | empty list | `PinnedFolderSettingsViewModel.cs:16-22` |

`IsVisible = false` — this group is not shown in the settings UI; it only exists for persistence.

#### 2.6.3 Window Position Settings (Generic `TWindow : Window`)

| ID | Setting | Type | Default | Source |
|----|---------|------|---------|--------|
| FR-SW-01 | `Left` | `int` | `100` | `WindowPositionSettingsViewModel.cs:16-17` |
| FR-SW-02 | `Top` | `int` | `100` | `WindowPositionSettingsViewModel.cs:21-22` |
| FR-SW-03 | `Width` | `int` | `1000` | `WindowPositionSettingsViewModel.cs:23` |
| FR-SW-04 | `Height` | `int` | `600` | `WindowPositionSettingsViewModel.cs:11` |
| FR-SW-05 | `IsMaximized` | `bool` | `false` | `WindowPositionSettingsViewModel.cs:13` |
| FR-SW-06 | `ScreenCount` | `int` | `0` (tracked) | `WindowPositionSettingsViewModel.cs:19` |

**Restore logic** (`WindowHelper.cs:20-42`): If `ScreenCount` differs from current system screen count, position defaults are used (prevents off-screen windows after monitor changes). Otherwise, saved position is restored.

**Save logic** (`WindowHelper.cs:44-61`): If maximized, temporarily restore to normal to capture the non-maximized bounds, then save all values.

**Source:** `src/ImageSort.WPF/SettingsManagement/WindowPosition/WindowPositionSettingsViewModel.cs:7-65`  
**Source:** `src/ImageSort.WPF/Views/WindowHelper.cs:11-62`

#### 2.6.4 Metadata Panel Settings

| ID | Setting | Type | Default | Source |
|----|---------|------|---------|--------|
| FR-SMP-01 | `IsExpanded` | `bool` | `false` | `MetadataPanelSettings.cs:19-24` |
| FR-SMP-02 | `MetadataPanelWidth` | `int` | `300` | `MetadataPanelSettings.cs:26-31` |

### 2.7 Hotkey / Key-Binding System

#### 2.7.1 Data Model

`Hotkey` is a C# `record` with `Key Key` and `ModifierKeys Modifiers` (flags: `Control`, `Shift`, `Alt`, `Windows`). String representation: `"Ctrl + Shift + A"`.

**Source:** `src/ImageSort.WPF/SettingsManagement/ShortCutManagement/Hotkey.cs:8-27`

#### 2.7.2 Bindable Actions (Complete List)

| # | VM Property | Default Key | Default Modifiers | Description | Source Line |
|---|-------------|-------------|-------------------|-------------|-------------|
| 1 | `Move` | `Up` | `None` | Move selected image to selected folder | `KeybindingsSettingsGroupViewModel.cs:18` |
| 2 | `Delete` | `Down` | `None` | Delete selected image | `KeybindingsSettingsGroupViewModel.cs:26` |
| 3 | `Rename` | `R` | `None` | Rename selected image | `KeybindingsSettingsGroupViewModel.cs:34` |
| 4 | `GoLeft` | `Left` | `None` | Select previous image | `KeybindingsSettingsGroupViewModel.cs:43` |
| 5 | `GoRight` | `Right` | `None` | Select next image | `KeybindingsSettingsGroupViewModel.cs:51` |
| 6 | `CreateFolder` | `C` | `None` | Create folder under selected | `KeybindingsSettingsGroupViewModel.cs:60` |
| 7 | `FolderUp` | `W` | `None` | Navigate folder tree up | `KeybindingsSettingsGroupViewModel.cs:69` |
| 8 | `FolderLeft` | `A` | `None` | Collapse folder | `KeybindingsSettingsGroupViewModel.cs:77` |
| 9 | `FolderDown` | `S` | `None` | Navigate folder tree down | `KeybindingsSettingsGroupViewModel.cs:85` |
| 10 | `FolderRight` | `D` | `None` | Expand folder | `KeybindingsSettingsGroupViewModel.cs:93` |
| 11 | `Undo` | `Q` | `None` | Undo last action | `KeybindingsSettingsGroupViewModel.cs:102` |
| 12 | `Redo` | `E` | `None` | Redo last undone action | `KeybindingsSettingsGroupViewModel.cs:110` |
| 13 | `OpenFolder` | `O` | `None` | Open folder picker dialog | `KeybindingsSettingsGroupViewModel.cs:118` |
| 14 | `OpenSelectedFolder` | `Enter` | `None` | Open selected folder in tree | `KeybindingsSettingsGroupViewModel.cs:127` |
| 15 | `Pin` | `P` | `None` | Pin folder via dialog | `KeybindingsSettingsGroupViewModel.cs:136` |
| 16 | `PinSelected` | `F` | `None` | Pin selected folder | `KeybindingsSettingsGroupViewModel.cs:144` |
| 17 | `Unpin` | `U` | `None` | Unpin selected folder | `KeybindingsSettingsGroupViewModel.cs:152` |
| 18 | `MoveSelectedPinnedFolderUp` | `W` | `Control` | Move pinned folder up | `KeybindingsSettingsGroupViewModel.cs:160` |
| 19 | `MoveSelectedPinnedFolderDown` | `S` | `Control` | Move pinned folder down | `KeybindingsSettingsGroupViewModel.cs:168` |
| 20 | `SearchImages` | `I` | `None` | Focus search box | `KeybindingsSettingsGroupViewModel.cs:177` |
| 21 | `ToggleMetadataPanel` | `M` | `None` | Open/close metadata panel | `KeybindingsSettingsGroupViewModel.cs:185` |

#### 2.7.3 Hotkey Validation & Collision Handling

- **No collision detection is implemented.** Users can bind the same key combination to multiple actions.
- **No OS-level hotkey registration** (no `RegisterHotKey` Win32 API). Keys are captured via WPF `PreviewKeyDown` events on the `MainWindow`.
- When a `TextBox` is focused (e.g., search box), hotkey intercept is disabled (`MainWindow.xaml.cs:124`).
- The `HotkeyEditorControl` captures `PreviewKeyDown` on a text box, suppresses default behavior, and converts the key+modifiers to a `Hotkey` record. It ignores modifier-only keys (Ctrl, Alt, Shift, Win, Clear, OemClear, Apps). Pressing Delete/Backspace/Escape clears the hotkey to `null`. `Key.System` is remapped from `e.SystemKey` for Alt combinations.
- `RestoreDefaultBindings` command resets all 21 bindings to their defaults.

**Source:** `src/ImageSort.WPF/SettingsManagement/ShortCutManagement/HotkeyEditorControl.xaml.cs:40-67`  
**Source:** `src/ImageSort.WPF/MainWindow.xaml.cs:118-127`

---

## 3. UNDO/REDO & MUTATION ENGINE (THE ACTION SYSTEM)

### 3.1 Interface Definition

```csharp
public interface IReversibleAction
{
    string DisplayName { get; }
    void Act();
    void Revert();
}
```

**Source:** `src/ImageSort/Actions/IReversibleAction.cs:3-9`

There are exactly **three concrete implementations**: `MoveAction`, `DeleteAction`, `RenameAction`.

### 3.2 ActionsViewModel: The Action Stack

`ActionsViewModel` maintains **two stacks**:

| Stack | Type | Purpose |
|-------|------|---------|
| `done` | `Stack<IReversibleAction>` | Executed actions (undoable) |
| `undone` | `Stack<IReversibleAction>` | Undone actions (redoable) |

**Commands:**

| Command | Behavior |
|---------|----------|
| `Execute(IReversibleAction)` | Calls `action.Act()`. On success, pushes to `done`. Clears `undone` (new action invalidates redo history). On failure, notifies user of error via `NotifyUserOfError` Interaction. |
| `Undo()` | Pops from `done`, calls `action.Revert()`. On success, pushes to `undone`. On failure, notifies user. |
| `Redo()` | Pops from `undone`, calls `action.Act()`. On success, pushes to `done`. On failure, notifies user. |
| `Clear()` | Clears both stacks. |

**Observables:**
- `LastDone` (OAPH): Display name of top of `done` stack, or `null`.
- `LastUndone` (OAPH): Display name of top of `undone` stack, or `null`.
- Both update reactively on any of `Execute`, `Undo`, `Redo`, `Clear`.

**Source:** `src/ImageSort/ViewModels/ActionsViewModel.cs:13-118`

### 3.3 MoveAction

#### Construction (captured state)
- `oldDestination` = `Path.GetFullPath(file)` — the original file path (absolute).
- `newDestination` = `Path.Combine(Path.GetFullPath(toFolder), Path.GetFileName(file))`.
- `fileSystem` — injected `IFileSystem`.
- `notifyAct(old, new)` / `notifyRevert(new, old)` — callbacks to update the UI collection.
- **Preconditions**: file must exist (`FileNotFoundException` if not), target directory must exist (`DirectoryNotFoundException` if not).

#### Execute (`Act`)
1. `fileSystem.Move(oldDestination, newDestination)` — performs the actual file move.
2. `notifyAct?.Invoke(oldDestination, newDestination)` — in practice, calls `images.Replace(old, new)` on the `SourceList`.

#### Undo (`Revert`)
1. `fileSystem.Move(newDestination, oldDestination)` — moves the file back.
2. `notifyRevert?.Invoke(newDestination, oldDestination)` — `images.Replace(new, old)`.

#### Conflict handling
- **Destination exists**: `File.Move` will throw `IOException`. This is NOT pre-checked in the constructor; the exception propagates through `ActionsViewModel.Execute` → error notification.
- **Source deleted externally before Act**: Not pre-checked at execution time; `File.Move` throws.
- **Source deleted externally before Revert**: `File.Move` throws → error notification.

**Source:** `src/ImageSort/Actions/MoveAction.cs:8-58`

### 3.4 DeleteAction

#### Construction (captured state)
- `oldPath` = the file path.
- `recycleBin` — injected `IRecycleBin`.
- `notifyAct(path)` / `notifyRevert(path)` — callbacks.
- **Preconditions**: file must exist (`FileNotFoundException` if not).

#### Execute (`Act`)
1. If `deletedFile == null`, calls `recycleBin.Send(oldPath)` — sends file to OS recycle bin, returns an `IDisposable` handle (a token that can restore the file).
2. `notifyAct?.Invoke(oldPath)` — `images.Remove(oldPath)`.

#### Undo (`Revert`)
1. `deletedFile?.Dispose()` — invokes the restorer, which calls `RecycleBin.RestoreFileFromRecycleBin()`.
2. `deletedFile = null`.
3. `notifyRevert?.Invoke(oldPath)` — `images.InsertImage(oldPath)`.

#### Conflict handling
- **File already deleted externally**: `recycleBin.Send()` via `SHFileOperation` with `FO_DELETE | FOF_ALLOWUNDO` — if the file doesn't exist, `SHFileOperation` likely returns success (no-op). The `IDisposable` is still returned.
- **Undo when recycle bin item was permanently deleted externally**: `RestoreFileFromRecycleBin` iterates the shell recycle bin namespace (`shell.NameSpace(10)`) and attempts to match the file. If not found, throws `FileNotFoundException`. `ActionsViewModel` catches it → error notification.
- **Undo when recycle bin item was moved out externally**: Same as above — match by original path fails → `FileNotFoundException`.

**Source:** `src/ImageSort/Actions/DeleteAction.cs:8-52`  
**Source:** `src/ImageSort.WPF/FileSystem/RecycleBin.cs:10-61`

### 3.5 RenameAction

#### Construction (captured state)
- `oldPath` = `Path.GetFullPath(path)`.
- `newPath` = `Path.Combine(Path.GetDirectoryName(path), newName + Path.GetExtension(path))`.
- Note: The extension is **always preserved**. The user provides only the stem name.
- **Preconditions**: file exists (`FileNotFoundException` if not), new path must not already exist (`IOException` if it does).

#### Execute (`Act`)
1. `fileSystem.Move(oldPath, newPath)`.
2. `notifyAct?.Invoke(oldPath, newPath)` — `images.Replace(old, new)`.

#### Undo (`Revert`)
1. `fileSystem.Move(newPath, oldPath)`.
2. `notifyRevert?.Invoke(newPath, oldPath)` — `images.Replace(new, old)`.

#### Conflict handling
- **Destination exists at construction**: Throws `IOException` ("file already exists") — caught in `ImagesViewModel.RenameImage`, which notifies the user.
- **Destination exists at execution time** (file created between construction and execution): `File.Move` throws → `ActionsViewModel` error notification.
- **Source renamed externally between construction and execution**: `File.Move` throws → error notification.

**Source:** `src/ImageSort/Actions/RenameAction.cs:8-55`  
**Source:** `src/ImageSort/ViewModels/ImagesViewModel.cs:125-160`

---

## 4. PLATFORM & OS-LEVEL INTEGRATIONS

### 4.1 File System Abstraction (`IFileSystem`)

The core library defines a clean interface:

```csharp
public interface IFileSystem {
    IEnumerable<string> GetSubFolders(string path);
    IEnumerable<string> GetFiles(string folder);
    bool IsFolderEmpty(string path);
    bool FileExists(string path);     // default: File.Exists
    bool DirectoryExists(string path); // default: Directory.Exists
    void Move(string source, string destination); // default: File.Move
    void CreateFolder(string path);  // default: Directory.CreateDirectory
}
```

**Production implementation:** `FullAccessFileSystem` — uses `Directory.EnumerateDirectories` and `Directory.EnumerateFiles` with `SearchOption.TopDirectoryOnly` (no recursion). `IsFolderEmpty` uses `!Directory.EnumerateDirectories(...).Any()`.

**Source:** `src/ImageSort/FileSystem/IFileSystem.cs:6-21`  
**Source:** `src/ImageSort/FileSystem/FullAccessFileSystem.cs:7-14`

### 4.2 Recycle Bin Integration (WINDOWS-SPECIFIC)

#### Delete Flow

1. `DeleteAction.Act()` calls `IRecycleBin.Send(path, confirmationNeeded: false)`.
2. `RecycleBin.Send()` calls `FileOperationApiWrapper.Send(path, FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_WANTNUKEWARNING)`.
3. `FileOperationApiWrapper.Send()` invokes the Win32 function `SHFileOperation` from `shell32.dll` with:
   - `wFunc = FO_DELETE` (0x0003)
   - `pFrom = path + '\0' + '\0'` (double-null-terminated string)
   - `fFlags = FOF_ALLOWUNDO (0x0040) | flags`
4. **Key flag: `FOF_ALLOWUNDO`** — This is the flag that tells Windows to send the file to the Recycle Bin instead of permanently deleting it. Without this flag, the file would be deleted permanently.
5. **`FOF_WANTNUKEWARNING`** — Warns if the file is too large for the Recycle Bin (would require permanent deletion).
6. **`FOF_NOCONFIRMATION`** — Suppresses the confirmation dialog.

**This is NOT a permanent deletion.** The file is moved to the Windows Recycle Bin via the native shell API.

#### Restore Flow (Undo)

1. `DeleteAction.Revert()` calls `deletedFile.Dispose()`.
2. `Disposable.Create(path, RestoreFileFromRecycleBin)` → `RestoreFileFromRecycleBin(path)`.
3. The method opens the shell Recycle Bin namespace (`shell.NameSpace(10)`) via `Shell32.Shell` COM interop.
4. Iterates all items, reconstructs original path by combining `GetDetailsOf(item, 1)` (original location path) and `GetDetailsOf(item, 0)` (original file name). If the item has no extension in the display name, it appends the extension from `item.Path`.
5. When a match is found via `path.PathEquals(...)`, calls `Restore(item)` → `itemVerbs.Item(0).DoIt()` — invokes the default verb on the recycle bin item (restore).
6. If no match is found, throws `FileNotFoundException`.

#### Network Drives / Non-Windows Paths

- `SHFileOperation` with `FOF_ALLOWUNDO` only works for local drives. On network drives, it will perform a **permanent deletion** because the Recycle Bin does not exist on network locations. The current implementation does **not** detect or handle this case differently.
- On non-Windows platforms (`iced` in the new app will be cross-platform), this entire mechanism must be replaced with platform-appropriate trash APIs (e.g., `trash` crate on Linux, `trash` on macOS via the `trash` crate in Rust).

**Source:** `src/ImageSort.WPF/FileSystem/FileOperationApiWrapper.cs:6-164`  
**Source:** `src/ImageSort.WPF/FileSystem/RecycleBin.cs:10-61`  
**Source:** `src/ImageSort/FileSystem/IRecycleBin.cs:5-19`

### 4.3 Shell Icon Loading (WINDOWS-SPECIFIC)

`ShellFileLoader` retrieves folder icons from the Windows shell:

1. Calls `SHGetFileInfo` from `shell32.dll` with:
   - `dwFileAttributes = FILE_ATTRIBUTE_DIRECTORY (0x00000010)`
   - `uFlags = SHGFI_ICON (0x100) | SHGFI_SMALLICON (0x1)`
2. Gets the `HICON` from `SHFILEINFO.hIcon`, clones it to a `System.Drawing.Icon`, converts to `Bitmap`.
3. `DestroyIcon(hIcon)` — manual cleanup of the icon handle.
4. For WPF display: converts the `Bitmap` to a `BitmapImage` via a `MemoryStream` with PNG encoding.

**Source:** `src/ImageSort.WPF/FolderIcons/ShellFileLoader.cs:10-128`

### 4.4 FileSystemWatcher Integration

Both `ImagesViewModel` and `FolderTreeItemViewModel` use `System.IO.FileSystemWatcher` for real-time directory monitoring:

#### ImagesViewModel FileSystemWatcher
- **Activation**: Created/disposed whenever `CurrentFolder` changes (`ImagesViewModel.cs:162-180`).
- **Path**: The current image folder.
- **Filter**: `NotifyFilters.FileName`.
- **Buffer**: 64KB (`InternalBufferSize = 64000`).
- **Subdirectories**: Not included.
- **Events handled**:
  - `Created` → if extension matches supported types AND not already in the list, add to `SourceList`.
  - `Deleted` → find matching item by path (case-insensitive path comparison via `PathEquals`), remove from list.
  - `Renamed` → find by old path, `Replace(old, new)`.
- **Thread marshaling**: All event handlers schedule updates on `RxSchedulers.MainThreadScheduler` via `Schedule()`.
- **Destructor**: `~ImagesViewModel()` unsubscribes events and disposes the watcher.

#### FolderTreeItemViewModel FileSystemWatcher (per node)
- **Activation**: When `Path` is set and not null (`FolderTreeItemViewModel.cs:140-163`).
- **Path**: The folder's path.
- **Filter**: `NotifyFilters.DirectoryName`.
- **Subdirectories**: Not included.
- **Events handled**:
  - `Created` → if folder not already tracked, add new `FolderTreeItemViewModel`.
  - `Deleted` → find by path, remove from `subFolders` SourceList.
  - `Renamed` → find by old path, remove old, insert new viewmodel with new path.
- **Error handling**: Entire watcher setup wrapped in `catch {}` — if the watcher can't start (e.g., path doesn't exist, permissions), it silently does nothing.
- **Destructor**: `~FolderTreeItemViewModel()` unsubscribes and disposes via `CompositeDisposable`.

**Important**: The `FileSystemWatcher` path must be set **after** subscribing to events, or `EnableRaisingEvents` may fail silently. The current code sets `Path`, then subscribes, then sets `EnableRaisingEvents = true` (`ImagesViewModel.cs:166-180`).

**Source:** `src/ImageSort/ViewModels/ImagesViewModel.cs:22, 162-243`  
**Source:** `src/ImageSort/ViewModels/FolderTreeItemViewModel.cs:26-27, 140-213`

### 4.5 Registry Integration (Explorer Context Menu)

`GeneralSettingsGroupViewModel.cs:81-106` writes to Windows Registry at:
```
HKCU\Software\Classes\Directory\shell\ImageSort
HKCU\Software\Classes\Drive\shell\ImageSort
HKCU\Software\Classes\Folder\shell\ImageSort
```
Each key gets:
- Default value: `"Open with Image Sort"`
- `command` subkey default: `"path\to\Image Sort.exe" "%L"`
- `Icon` value: `"path\to\Image Sort.exe"`

Disabled in `DEBUG` builds. When disabled, all keys are deleted via `DeleteSubKeyTree`.

**Source:** `src/ImageSort.WPF/SettingsManagement/GeneralSettingsGroupViewModel.cs:81-106`

### 4.6 Auto-Updater (GitHub Releases)

Conditionally compiled (`#if !DO_NOT_INCLUDE_UPDATER`):

1. On startup, `InstallerRunner.CleanUpInstaller()` deletes leftover MSI from `%TEMP%\Image Sort`.
2. `GitHubUpdateFetcher` uses `Octokit` to check `Lolle2000la/Image-Sort` releases.
3. Compares `GitVersionInformation.SemVer` with latest release tag using `Semver` library.
4. If a newer version exists (pre-release filtering based on user setting), shows a `MessageBox`.
5. If user accepts, downloads the MSI asset (`ImageSort.x64.msi` or `ImageSort.x86.msi`).
6. `InstallerRunner.RunAsync()` saves the MSI to `%TEMP%\Image Sort\ImageSort.{arch}.msi`, then runs `msiexec /i "path" TARGETDIR="installDir" /passive AUTOSTART=1` with `runas` verb, then `Environment.Exit(0)`.

**Source:** `src/ImageSort.WindowsUpdater/GitHubUpdateFetcher.cs:12-85`  
**Source:** `src/ImageSort.WindowsUpdater/InstallerRunner.cs:8-49`  
**Source:** `src/ImageSort.WPF/App.xaml.cs:90-125`

### 4.7 Folder Picker Dialog

Uses `System.Windows.Forms.FolderBrowserDialog` (legacy WinForms dialog) with `ShowNewFolderButton = true`. Registered as handler for the `PickFolder` Interaction on `MainViewModel`.

**Source:** `src/ImageSort.WPF/MainWindow.xaml.cs:102-112`

### 4.8 Path Comparison Helper

`PathHelper.PathEquals` normalizes both paths via `Path.GetFullPath()` and compares case-insensitively (`OrdinalIgnoreCase`). Used throughout for all path comparisons in collections.

**Source:** `src/ImageSort/Helpers/PathHelper.cs:6-12`

### 4.9 String Extension

`StringHelper.EndsWithAny` checks if a string ends with any of the given suffixes using a specified `StringComparison`. Used for image extension filtering.

**Source:** `src/ImageSort/Helpers/StringHelper.cs:6-15`

---

## 5. NON-FUNCTIONAL & METADATA REQUIREMENTS

### 5.1 Localization Infrastructure

- **Framework**: RESX-based (`.resx` files with auto-generated `Designer.cs`).
- **Languages**: English (default), German (de-DE).
- **Two resource files**:
  - `Text.resx` / `Text.de-DE.resx` — General UI strings (73 entries).
  - `KeyBindingNames.resx` / `KeyBindingNames.de-DE.resx` — Hotkey descriptions (21 entries).
- **Culture override**: In `DEBUG && !DEBUG_LOCALIZATION`, forced to `en` culture (`App.xaml.cs:38-41`).

**Source:** `src/ImageSort.Localization/Text.Designer.cs` (auto-generated, 642 lines)  
**Source:** `src/ImageSort.Localization/Text.resx`  
**Source:** `src/ImageSort.Localization/Text.de-DE.resx`

### 5.2 Localization String Categories & Keys

| Category | Key Count | Example Keys |
|----------|-----------|--------------|
| Error Messages | 6 | `CouldNotActErrorText`, `CouldNotUndoErrorText`, `CouldNotRedoErrorText`, `CouldNotLoadImageErrorText`, `CouldNotDeleteFileExceptionMessage`, `RenameNewNameContainsIllegalCharacters` |
| Action Descriptions | 3 | `MoveActionMessage`, `DeleteActionMessage`, `RenameActionMessage` |
| UI Labels | 30+ | `Move`, `Delete`, `Rename`, `Undo`, `Redo`, `Open`, `OpenFolder`, `Pin`, `Unpin`, `Settings`, `CreateFolder`, `DarkMode`, etc. |
| Settings Headers | 6 | `GeneralSettingsHeader`, `KeyBindingsSettingsHeader`, `PinnedFoldersSettingsHeader`, `MetadataPanelHeader` |
| Prompts/Dialogs | 6 | `RenameImagePromptTitle`, `NewFolderPromptText`, `UpdateAvailablePromptText`, `SearchTermWatermark` |
| Key Binding Names | 21 | `GoLeft` → "Select image on the left", `FolderUp` → "Select the folder above", etc. |

**Template patterns**: Strings use `{FileName}`, `{Directory}`, `{ErrorMessage}`, `{ActMessage}`, `{OldFileName}`, `{NewFileName}`, `{TagName}` as substitution placeholders.

### 5.3 Image Metadata Extraction

#### Library
`MetadataExtractor` NuGet package v2.9.3 (Java-based `drewnoakes/metadata-extractor` ported to .NET).

#### Extractor Implementation
`FullAccessFileSystemMetadataExtractor.Extract(string path)`:
1. Calls `ImageMetadataReader.ReadMetadata(path)` — reads all metadata directories from the image file.
2. Returns `Dictionary<string, Dictionary<string, string>>`:
   - **Outer key**: Directory name (e.g., "JPEG", "EXIF IFD0", "GPS", "IPTC", "XMP", "ICC Profile", "PNG", "GIF", "BMP", "ICO", "WebP").
   - **Inner key**: Tag name (e.g., "Image Width", "Make", "Model", "Date/Time Original").
   - **Inner value**: Tag description (human-readable string).
3. This means **all EXIF, IPTC, XMP, ICC, and format-specific metadata fields** that the `MetadataExtractor` library supports are extracted — not a curated subset.

#### Supported output fields
The library supports 40+ directory types across JPEG, PNG, GIF, BMP, TIFF, ICO, WebP, PSD, RAW formats (CR2, NEF, ARW, DNG, etc.), PCX, EPS, and more. Common directories include:

| Directory | Example Tags |
|-----------|-------------|
| JPEG | Image Width, Image Height, Data Precision |
| EXIF IFD0 | Make, Model, Orientation, Date/Time, Software |
| EXIF SubIFD | Exposure Time, FNumber, ISO, Focal Length, Flash |
| GPS | Latitude, Longitude, Altitude |
| IPTC | Keywords, Caption, Copyright, Credit |
| XMP | Various XML-based metadata |
| ICC Profile | Color profile information |
| File Type | Detected File Type Name, Expected File Name Extension, MIME Type |
| PNG | Various PNG-specific metadata |
| GIF | GIF-specific metadata |

**Source:** `src/ImageSort/FileSystem/FullAccessFileSystemMetadataExtractor.cs:10-27`  
**Source:** `src/ImageSort/FileSystem/IMetadataExtractor.cs:5-8`

### 5.4 Metadata ViewModel Tree

```
MetadataViewModel
  ├── ImagePath (input)
  ├── IsExpanded (bool, toggled by 'M' key)
  ├── Metadata (MetadataResult: success/file-not-found/error)
  └── SectionViewModels: IEnumerable<MetadataSectionViewModel>
        ├── Title (string, e.g. "EXIF IFD0")
        ├── Fields (Dictionary<string, string>)
        └── FieldViewModels: IEnumerable<MetadataFieldViewModel>
              ├── Name (string, e.g. "Make")
              └── Value (string, e.g. "Canon")
```

**Lazy loading**: Metadata is only extracted when `IsExpanded` is true AND `ImagePath` is non-null (`MetadataViewModel.cs:46-50`). When the panel is collapsed, no extraction occurs.

**Error handling**: Three result types — `Success`, `FileDoesNotExist`, `UnexpectedError`. Exceptions during extraction are captured in `MetadataResult.Exception` and do not crash the app.

**Source:** `src/ImageSort/ViewModels/Metadata/MetadataViewModel.cs:14-110`

### 5.5 Supported Image Formats (for display, not just metadata)

| Extension | Display | Metadata Extraction (via MetadataExtractor) |
|-----------|---------|---------------------------------------------|
| `.png` | Yes | Yes |
| `.jpg` / `.jpeg` | Yes | Yes |
| `.gif` | Yes (including animated) | Yes |
| `.bmp` | Yes | Yes |
| `.tiff` / `.tif` | Yes | Yes |
| `.ico` | Yes | Yes |
| `.webp` | Yes | Yes |

**For Media Sort**: Video formats (`.mp4`, `.mkv`, `.webm`, `.avi`, `.mov`, etc.) and audio formats (`.mp3`, `.flac`, `.ogg`, `.wav`, `.aac`, `.m4a`, `.wma`, etc.) should be added.

### 5.6 Persistent Configuration Storage

- **Format**: JSON (System.Text.Json, indented).
- **Location**: `%APPDATA%/Image Sort/config.json` (release) or `debug_config.json` (debug).
- **Structure**: `{ "GroupName": { "PropertyName": value, ... }, ... }`
- **Save trigger**: On application close (`Closed` event → `settings.SaveAsync()`).
- **Restore trigger**: On startup (`OnStartup` → `settings.Restore()`).
- **UI test isolation**: If `UI_TEST` env var is set, config is loaded/saved from `ui_test_config.json` in the application directory.

**Source:** `src/ImageSort.WPF/SettingsManagement/SettingsHelper.cs:15-80`

### 5.7 Global Exception Handling

Three handlers registered in `App` constructor:
1. `DispatcherUnhandledException` — logs to trace, marks as handled (prevents crash).
2. `TaskScheduler.UnobservedTaskException` — logs to trace.
3. `AppDomain.CurrentDomain.UnhandledException` — logs to trace.

**Source:** `src/ImageSort.WPF/App.xaml.cs:60-79`

### 5.8 Supported Image Preloading / Caching

- `LazyCache` with `MemoryCache`:
  - Size limit: 20 entries (`SizeLimit = 20`).
  - Each entry counts as 1 unit (`Size = 1`).
  - Cache key: full file path.
  - Cache miss triggers synchronous load from disk.
  - No pre-emptive preloading of adjacent images — only the currently selected image is loaded.
  - GIF thumbnails are conditional on `AnimateGifThumbnails && AnimateGifs` settings.

**Source:** `src/ImageSort.WPF/FileSystem/ImageLoading.cs:17-28`

---

## APPENDIX A: Dependency Graph (for Rust equivalents)

| .NET Dependency | Purpose | Rust Equivalent Suggestion |
|-----------------|---------|---------------------------|
| `ReactiveUI` + `DynamicData` | MVVM framework, reactive collections, commands | `iced` built-in `Subscription`/`Command` + custom reactive state |
| `MetadataExtractor` | EXIF/IPTC/XMP/ICC metadata | `kamadak-exif`, `rexif`, `img-parts` (for images); `mp4ameta` (video); `id3` (audio) |
| `LazyCache` + `MemoryCache` | Image cache with size limit | Custom `LruCache` (or `lru` crate with `Arc`) |
| `Splat` | Service locator / DI | Manual DI via `iced` app state, or `shaku`/`anymap` |
| `AdonisUI` | WPF theming | `iced` theming via `Theme` / `Style` |
| `WpfAnimatedGif` | Animated GIF playback | `gif` crate + custom `wgpu` renderer |
| `Octokit` + `Semver` | GitHub update checking | `octocrab` + `semver` crate |
| `GitVersionTask` | Auto versioning from git | `git-version` or build script |
| `JsonSerializer` (System.Text.Json) | Config persistence | `serde_json` |

---

## APPENDIX B: Data Flow Summary Diagram (Text)

```
┌──────────────────────────────────────────────────────┐
│                    MainViewModel                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Folders  │  │  Images  │  │     Actions      │  │
│  │  VM      │  │   VM     │  │      VM          │  │
│  │          │  │          │  │ ┌──────┐┌───────┐│  │
│  │Pinned[]  │  │Images[]  │  │ │ done ││undone ││  │
│  │Selected  │  │Selected  │  │ │Stack ││Stack  ││  │
│  │Current   │  │Current   │  │ └──────┘└───────┘│  │
│  └────┬─────┘  └────┬─────┘  └────────┬─────────┘  │
│       │             │                 │             │
│  CurrentFolder.path─────►CurrentFolder│             │
│       │             │                 │             │
│       │    SelectedImage──►MetadataVM │             │
│       │             │                 │             │
│  Selected.path ◄──── MoveImageToFolder              │
│       │             │      DeleteImage              │
│       │             │      RenameImage──────────────┤
│       │             │                 │             │
│       │             │        Actions.Execute(action)│
│       │             │                 │             │
└───────┴─────────────┴─────────────────┴─────────────┘
                      │
             CurrentFolder change
                      │
              Actions.Clear()  (flushes history)
```

---

## APPENDIX C: Test Coverage Summary (for regression parity)

### Unit Tests (13 test classes, `xUnit` + `NSubstitute`)

| Test Class | Tests | Coverage |
|------------|-------|----------|
| `MoveActionTests` | 2 | Constructor validation, Act/Revert notification |
| `DeleteActionTests` | 2 | Act → recycle bin, Revert → restore, file-not-found |
| `RenameActionTests` | 2 | Act/Revert, collision detection |
| `MainViewModelTests` | 5 | Open folder, select picked folder, move, delete, select index preservation |
| `ImagesViewModelTests` | 5 | File filtering by extension, selected image, add/remove, search filter, rename with validation |
| `FoldersViewModelTests` | 8 | Change current, pin, unpin, cancel, pin selected, mark current, move pinned up/down |
| `FolderTreeItemViewModelTests` | (not read but exists) | |
| `ActionsViewModelTest` | 2 | Execute→Undo→Redo→Clear workflow, error notification on all failure paths |
| `MetadataViewModelTests` | 4 | Extract, file-not-exist, exception handling, section construction |
| `SettingsGroupViewModelBaseTests` | (exists) | |
| `SettingsViewModelTests` | (exists) | |

### UI Tests (3 test classes, `FlaUI` / `xUnit`)

| Test Class | Tests | Coverage |
|------------|-------|----------|
| `FileActionsTests` | 2 | Move + Undo + Redo (with double-add guard), Delete + Undo |
| `FolderActionsTests` | (exists) | |
| `SearchTests` | (exists) | |

---

*Document generated by static code analysis of Image-Sort commit tree. All line references correspond to the extracted source files. For questions or clarifications, reference the specific file+line annotations above.*
