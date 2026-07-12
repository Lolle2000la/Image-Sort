---
title: Media Capabilities Matrix
description: Supported file formats, metadata extraction capabilities, and search filtering options in Media Sort.
---

Media Sort is a fully featured organizer supporting a wide range of media formats. It extracts extensive metadata from your media files, enabling you to inspect details and query them via the search bar.

## Supported Formats

Media Sort categorizes files into three distinct media types:

### 1. Images
Supported native image formats (decoded via the pure Rust `image` crate):
- **Formats:** JPEG/JPG, PNG, WebP, BMP, TIFF, TGA, Farbfeld (FF), AVIF, DDS, OpenEXR (EXR), HDR, ICO, QOI, and PNM (PBM, PGM, PPM, PAM).
- **GIF Handling:** Animated GIFs are internally parsed and routed to the video rendering stack to support autoplay, loop, and pause controls in the grid and preview panel.

### 2. Video & Containers
Supported video formats (handled dynamically using system-linked `libmpv` and underlying FFmpeg demuxers):
- **Supported Formats (e.g. under CachyOS/Linux with latest mpv v0.41.0):** MP4, MKV, WebM, AVI, MOV, WMV, FLV, M4V, MPEG/MPG, TS, VOB, 3GP, OGM, and animated GIF.
- **Dynamic Dependency:** Codec and container format compatibility depends entirely on the version of `libmpv` and the underlying FFmpeg compilation (e.g. `libavcodec`/`libavformat`) installed on the host system.

### 3. Audio
Supported native audio formats (decoded via Symponia / Rodio):
- **Formats:** MP3, FLAC, OGG (Vorbis), WAV, AAC, M4A (MPEG-4 Audio / ALAC), WMA, OPUS, and AIFF.

---

## Metadata Extraction

When you select a media file, the metadata panel displays details read directly from the file structure:

| Media Type | Extracted Metadata Attributes | Key Library |
| :--- | :--- | :--- |
| **Images** | EXIF fields (Camera model, Exposure time, ISO, F-number, Date taken, GPS coordinates, Dimensions) | `kamadak-exif` |
| **Audio** | Audio tags (Title, Artist, Album, Year, Genre, Track number, Bitrate, Duration) | `id3` / `metaflac` / `mp4ameta` |
| **Video** | Video file parameters (Codec, Resolution, Bitrate, Frame rate, Duration) | Custom parser |

---

## Search Filtering

You can search and filter the current file listing using the search bar (focus with the `I` key):

- **Filename Search:** Type any part of a filename to instantly filter the grid (e.g., searching "vacation").
- **Extension Filtering:** Filter files by their format extension since extensions are part of the filename (e.g., typing `.mp4` or `.flac` to only display files of that type).

*Note: Filtering matches against the filename structure case-insensitively; searching does not query internal file metadata fields (like EXIF tags or audio artists).*

---

## Performance & Caching

To keep navigation and browsing completely seamless, Media Sort implements a smart background prefetching and caching system:

- **Thumbnail Prefetching:** Eagerly spawns background worker tasks to generate thumbnails for upcoming files in your current directory.
- **LRU Cache:** Stores images and thumbnails in a high-speed Least Recently Used (LRU) cache (supporting up to 200 thumbnail handles and 20 full-resolution preview handles) to ensure instant retrieval.
- **Asynchronous File System Watcher:** Automatically monitors your active directory for changes (additions, deletions, or modifications) made outside the application and updates the UI grid dynamically.

