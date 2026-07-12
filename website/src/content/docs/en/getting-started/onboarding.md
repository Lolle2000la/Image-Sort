---
title: First Run Onboarding
description: What happens when you launch Media Sort for the first time.
---

When you launch Media Sort for the first time, the application starts directly in the media grid view. Here is what to expect and how to configure your workspace.

## Startup Behavior

On first launch, Media Sort automatically opens your system's default **Pictures** directory (e.g., `~/Pictures` on Linux, `~/Pictures` on macOS). If you have no files there, the grid will be empty — use the folder picker to select a different directory. On subsequent launches, Media Sort reopens the last folder you worked in (controlled by the **Reopen last opened folder** setting).

You can also pass a directory path as a command-line argument to open it directly:
```bash
media-sort-gui /path/to/your/media
```

If you previously used the legacy WPF version (v2.x), Media Sort will **silently migrate** your settings (pinned folders, dark mode, hotkeys, window position) from the old `config.json` into the new `config.toml` format. No manual action is required.

## 1. Select a Root Directory

To choose a directory containing the media files (images, videos, audio) you want to sort:

- Click the **Open folder** button under **Folder** in the control panel on the left to open the native folder picker dialog.
- Alternatively, press `O` to open the native folder picker dialog.
- Press `Enter` to open the currently selected folder in the folder tree.
- Media Sort will recursively scan the selected folder and all subfolders for supported media formats, presenting them in a structured grid.

## 2. Set Up Pinned Folders

Pinned folders are your target destinations for sorting. By pinning folders, you assign them quick action shortcuts for moving selected media with a single keystroke.

- Navigate to your target directory using the folder tree panel.
- Press `F` to pin the selected folder, or press `P` to open a folder picker and pin any directory on your system.
- Once pinned, these folders are assigned quick-key shortcuts: press `Alt + 1` through `Alt + 9` to move the selected media file directly to the corresponding pinned folder.
- Use `Ctrl + W` and `Ctrl + S` to reorder pinned folders, which changes which folder each number key targets.

## 3. Customize Your Setup

Open the **Settings** dialog by clicking the **Open** button under **Settings** in the control panel to configure:
- **Theme** — choose between Dark, Light, Dracula, Nord, Catppuccin, and more
- **Keybindings** — remap all shortcuts to your preferred layout
- **Language** — switch between English, German, and Japanese
- **GIF animation** — toggle animated GIF playback in the grid and preview
