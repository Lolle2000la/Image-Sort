---
title: First Run Onboarding
description: What happens when you launch Media Sort for the first time.
---

When you launch Media Sort for the first time, the application starts directly in the media grid view. Here is what to expect and how to configure your workspace.

## Startup Behavior

On first launch, Media Sort automatically opens your system's default **Pictures** directory (e.g., `~/Pictures` on Linux, `~/Pictures` on macOS). If you have no files there, the grid will be empty. You can use the folder picker to select a different directory.

If you previously used the legacy WPF version (v2.x), Media Sort will **silently migrate** your settings (pinned folders, dark mode, hotkeys, window position) from the old `config.json` into the new `config.toml` format. No manual action is required.

## 1. Select a Root Directory

To choose a directory containing the media files (images, videos, audio) you want to sort:

- Click the **Open** button under **Folder** in the left sidebar to open the native folder picker dialog.
- Alternatively, press `O` to open the native folder picker dialog.
- Media Sort will automatically scan the selected folder and recursively find all supported formats, presenting them in a structured grid.

## 2. Set Up Pinned Folders

Pinned folders are your target destinations for sorting. By pinning folders, you assign them quick action shortcuts for moving selected media with a single keystroke.

- Navigate to your target directory using the folder tree panel on the left.
- Select a folder and press `F` to pin it, or use the Pinned Folders panel to add target directories.
- Once pinned, these folders are assigned quick-key shortcuts: press `Alt + 1` through `Alt + 9` to move the selected media file directly to the corresponding pinned folder.

## 3. Customize Your Setup

Open the **Settings** dialog from the left sidebar to configure:
- **Theme** — choose between Dark, Light, Dracula, Nord, Catppuccin, and more
- **Keybindings** — remap all shortcuts to your preferred layout
- **Language** — switch between English, German, and Japanese
- **GIF animation** — toggle animated GIF playback in the grid and preview
