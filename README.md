# Media Sort

Sort your media files fast. Version 3.0 is a rewrite in Rust.

![Screenshot from the user interface of Media Sort](https://github.com/Lolle2000la/Image-Sort/raw/master/Image-Sort-Screenshot.gif)

## What is Media Sort?

Media Sort is a desktop app for sorting your media collection. It handles images, video, and audio files. You drive it from the keyboard.

### Why use it

- **It's fast.** Built in Rust, rendered with wgpu.
- **Keyboard-first.** All actions are bindable. Move, delete, rename, browse: your hands stay on the keyboard.
- **Undo and redo.** Moved a file into the wrong folder? Press Q. You can undo it. Press E to redo.
- **Pinned folders.** Pin folders you often sort into, then quick-move files with Alt+1 through 9.
- **In-app previews.** View images, play videos with seek and volume control, listen to audio. No need to open another app.
- **Metadata panel.** EXIF for photos, ID3/FLAC/MP4 tags for audio. Shown in a side panel you can collapse.
- **Multi-language.** English, German, Japanese. Picks up your system locale.

## Supported formats

| Type | Formats |
|------|---------|
| Images | PNG, JPEG, GIF, BMP, TIFF, ICO, WebP, JXL, HEIC, HEIF, AVIF |
| Audio | MP3, FLAC, OGG (Vorbis), WAV, AAC (M4A), WMA, Opus, AIFF |
| Video | MP4, MKV, WebM, AVI, MOV, WMV, FLV, M4V, and any other format your installed mpv can handle |

## How to use

> The central idea behind Media Sort is speed. The best way to use it is to keep your hands on the keyboard. But you can also use the mouse if you want. The app should help you either way.

### The folder tree

On the left is your folder tree. Navigate it with the WASD keys. W is up, S is down, A collapses a folder, D expands it. Hit Enter to open the selected folder, or O to pick some other folder.

### Pinned folders

Pin folders you use often with P (pin current) or F (pin selected). Unpin with U. After pinning, Alt+1 through 9 moves the current file to that pinned folder instantly. Reorder pins with Ctrl+W (up) and Ctrl+S (down).

This lets you sort files from one location into several destinations, even across different drives.

### Sorting

| Action | Shortcut |
|--------|----------|
| Move to selected folder | Up Arrow |
| Delete (move to trash) | Down Arrow |
| Previous file | Left Arrow |
| Next file | Right Arrow |
| Rename file | R |
| Undo | Q |
| Redo | E |
| Create new folder | C |
| Toggle search/filter | I |
| Toggle metadata panel | M |

All keybindings are customizable in the Settings dialog. Media keys (Play/Pause, Volume, Track skip) work for video and audio too.

## Privacy Policy

Read the [Privacy Policy](https://imagesort.org/privacy_policy.html).

No data is willingly collected. If you turn on "Check for updates on startup" in Settings, the app queries the GitHub API. That may collect data per GitHub's [privacy statement](https://help.github.com/en/github/site-policy/github-privacy-statement). Turn it off if you don't want that.

## Requirements

- Linux, macOS, or Windows
- Rust 1.80 or later (if building from source)
- libmpv (needed for video and audio playback)

Install libmpv through your package manager:
- Ubuntu/Debian: `libmpv-dev`
- Fedora: `mpv-libs-devel`
- Arch: `mpv`
- macOS: `brew install mpv`
- Windows: download from [mpv.io](https://mpv.io/installation/)

## Building

```
git clone https://github.com/Lolle2000la/Image-Sort.git
cd Image-Sort
cargo build --release
cargo run --release
```

Pre-built binaries for Linux, macOS, and Windows will ship with the v3.0 release.
