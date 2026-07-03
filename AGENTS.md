# AGENTS.md

## Build & verify

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all --
cargo clippy --workspace --all-targets -- -D warnings

# Documentation Website (Astro / Node.js 24 LTS)
cd website
npm ci
npm run build
```

Clang is required (libmpv-sys needs `libclang`).

## Pre-commit hooks

Hooks run `cargo fmt --all --` and `cargo clippy --fix --allow-dirty --allow-staged --all-targets -- -D warnings`. Formatter and clippy always scan the whole workspace, not just staged files. If clippy fails the commit is rejected.

## Workspace layout

Three crates in `crates/` with strict dependency order:

```
media-sort-gui       (iced 0.14, winit, wgpu — the app binary)
  └─ media-sort-backend  (filesystem, media decoding, libmpv)
       └─ media-sort-core    (settings, i18n, undo/redo, no system deps)

website/             (Astro, Starlight docs template, React components)
```

`media-sort-core` must never depend on `media-sort-backend` or `media-sort-gui`.

## libmpv

The GUI binary links libmpv at build time and loads it at runtime. The system must have `libmpv-dev` (or equivalent) installed to compile. Without it `cargo build` fails on the `libmpv-sys` crate.

At startup the app queries mpv for supported formats via `demuxer-lavf-list` and builds the media type registry dynamically. Video/audio support depends entirely on the installed mpv version.

## Configuration

Runtime config lives at `$CONFIG_DIR/media-sort/config.toml` (TOML). `$CONFIG_DIR` resolves via the `dirs` crate: `$XDG_CONFIG_HOME` (Linux), `~/Library/Application Support` (macOS), `%APPDATA%` (Windows).

On first launch the app silently migrates settings from the legacy WPF JSON config. In debug builds (`cfg!(debug_assertions)`) the migration reads `debug_config.json` instead of `config.json`:
- `$CONFIG_DIR/Image Sort/config.json` or `debug_config.json` (old WPF app)

When the `UI_TEST` environment variable is set to a non-empty value, the config path is overridden to `ui_test_config.toml` in the current directory. This is handled in `media-sort-core/src/settings/store.rs`.

## Internationalization

Fluent `.ftl` files in `resources/locale/{en,de,ja}/`. Adding a string requires entries in all three locales. The language is auto-detected from the system locale at startup.

## Tests

Tests live in `crates/*/tests/` as integration test files (not `#[cfg(test)]` modules). Run a single crate's tests:

```bash
cargo test -p media-sort-backend
cargo test -p media-sort-core
```

Run a single test file:

```bash
cargo test -p media-sort-backend --test filesystem_tests
```

Test fixtures are in `crates/media-sort-backend/tests/fixtures/`.

## Dev vs release builds

The dev profile sets `opt-level = 3` for all dependencies (`[profile.dev.package."*"]`). This keeps the GUI responsive in debug builds (iced/wgpu are slow without optimizations). Release builds use `lto = "fat"` and `codegen-units = 1`.

## Git

Conventional commits (`feat:`, `fix:`, `chore:`). The `docs/` directory is the GitHub Pages site served at `imagesort.org` via Jekyll. It is intentionally kept even though the help pages are outdated for v3.0.

`.Image-Sort-master/` is a local-only reference copy of the legacy WPF codebase. It is gitignored and must never be committed.

**CI Routing:** GitHub Actions path filtering decouples builds. Commits touching `website/` trigger the Astro production build and push to the `gh-pages` branch. Changes to application crates skip web deployment and route to the multi-platform Rust test matrices.

## Architecture pattern (iced / The Elm Architecture)

The GUI follows iced's TEA pattern with a unidirectional data flow:

- **Model** — `AppState` (`crates/media-sort-gui/src/state.rs:14`) holds all UI state
- **Messages** — `Message` enum (`crates/media-sort-gui/src/message.rs:8`) with nested sub-enums: `FolderMessage`, `MediaMessage`, `SettingsMessage`, `VideoMessage`
- **Update** — `app::update()` (`crates/media-sort-gui/src/app.rs:14`) is the pure reducer, ~950 lines, returning `Task<Message>` for side effects
- **View** — `app::view()` delegates to `main_layout_view()` which composes 10 view sub-modules
- **Subscription** — `app::subscription()` (`crates/media-sort-gui/src/app.rs:1218`) merges 4 streams into `Message`:

| Stream | Source | Purpose |
|--------|--------|---------|
| Tick | `iced::time::every(16ms)` | Main loop tick; handles deferred exit + settings save |
| Keyboard | `subscriptions::keyboard` | Raw key events via `winit`, matched against configurable keybindings |
| Events | `iced::event::listen()` | Window resize/move/close, mouse drag (divider resize) |
| Video | `subscriptions::video_player` | mpv worker thread events (frame ready, playback progress, etc.) |

Note: `crates/media-sort-gui/src/update.rs` is a 1-line stub (`// Update logic is in app.rs`). The module exists only to hold the `#[cfg(test)]` module with unit tests.

## Module overview

### media-sort-core (no system deps)

| Module | Purpose |
|--------|---------|
| `actions/` | `ReversibleAction` trait + `MoveAction`, `RenameAction`, `DeleteAction` |
| `history.rs` | Undo/redo with `done`/`undone` stacks of `Box<dyn ReversibleAction>` |
| `l10n.rs` | Fluent-backed localization, auto-detects system locale, `tr()` for lookups |
| `media_type.rs` | `MediaType` enum (Image/Video/Audio), global `MediaRegistry` (OnceLock), extension lists |
| `models.rs` | `MediaEntry`, `FolderNode`, `PinnedFolder` data types |
| `path_utils.rs` | Cross-platform path comparison utilities |
| `settings/` | `SettingsStore` + sub-modules: `general`, `keybindings`, `metadata_panel`, `pinned_folders`, `window_position` |
| `build.rs` | Auto-generates `locales_codegen.rs` from `resources/locale/` (see below) |

### media-sort-backend

| Module | Purpose |
|--------|---------|
| `filesystem/scanner.rs` | `walkdir`-based media file discovery |
| `filesystem/trash.rs` | Delete-to-trash (wrapping `platform/trash.rs`) |
| `filesystem/watcher.rs` | `notify` + `notify-debouncer-mini` filesystem change events |
| `media/mpv_context.rs` | `MpvContext`, `VideoWorker` thread, `VideoCommand`/`VideoEvent` channel protocol |
| `media/image_decoder.rs` | Full-resolution image loading via `image` crate |
| `media/audio_decoder.rs` | `AudioPlayer` using `rodio` output + `symphonia` decoding |
| `media/thumbnail.rs` | Thumbnail generation |
| `metadata/image_meta.rs` | EXIF extraction via `kamadak-exif` |
| `metadata/audio_meta.rs` | Audio tags via `id3`/`metaflac`/`mp4ameta` |
| `metadata/video_meta.rs` | Video metadata extraction |
| `platform/trash.rs` | OS-specific trash implementation |

### media-sort-gui

| Module | Purpose |
|--------|---------|
| `app.rs` | `update()`, `view()`, `theme()`, `subscription()`, helper async tasks |
| `main.rs` | Entry point: init mpv registry, load settings, launch iced application |
| `message.rs` | `Message` and sub-enum definitions |
| `state.rs` | `AppState` struct, folder tree logic, media scanning, `detect_media_type()` |
| `view/` | 10 view files: `main_layout`, `folder_tree`, `folder_panel`, `media_grid`, `media_preview`, `metadata_panel`, `control_panel`, `search_bar`, `settings_dialog`, `credits_dialog` |
| `widgets/` | Custom widgets: `video_canvas`, `video_player` (controls), `video_shader` (wgpu), `rename_modal`, `create_folder_modal`, `folder_icon` |
| `subscriptions/` | `keyboard.rs`, `video_player.rs`, `prefetch.rs` (thumbnail generation) |

### website (Astro + Starlight)

| Directory / File | Purpose |
| --- | --- |
| `astro.config.mjs` | Multi-language routing configuration, Starlight options, and React integration layer |
| `src/content/docs/` | Localized technical documentation manuals (`.md` and `.mdx` extensions) |
| `src/components/` | Custom interactive UI layers (e.g., React keyboard maps) embedded into manuals |

## Settings

`SettingsStore` (`crates/media-sort-core/src/settings/store.rs:54`) has 5 sub-structs:

- `GeneralSettings` — locale, dark mode, reopen folder, update checks, GIF animation, folder tree width
- `KeyBindings` — all user-configurable shortcuts
- `MetadataPanelSettings` — expanded state and panel width
- `PinnedFoldersSettings` — list of pinned folder paths as strings
- `WindowPosition` — left, top, width, height

Settings are **saved eagerly** on nearly every state mutation (not deferred to exit). This ensures crash resilience but means every toggle/action triggers a disk write. The `UI_TEST` env var swaps the config path to `ui_test_config.toml` for testing.

To add a new persisted setting:
1. Add the field to the appropriate sub-struct in `crates/media-sort-core/src/settings/`
2. Add `#[serde(default)]` (or a concrete default) so old configs remain loadable
3. If the setting has a UI toggle, call `state.settings.save()` after mutation

## Video pipeline

The video playback path is complex and worth understanding before touching:

1. **Startup** — `main.rs` queries mpv via `MpvContext::query_supported_extensions()` and initializes the global `MediaRegistry`
2. **Subscription** — `video_player_subscription()` spawns a `VideoWorker` background thread that owns the `MpvContext` and runs an mpv event loop
3. **Communication** — GUI sends `VideoCommand` (Load, Seek, SetVolume, TogglePause, Stop, Deactivate) via `tokio::sync::mpsc::Sender`; worker responds with `VideoEvent` (FrameReady, PlaybackProgress, Muted, Volume, Paused)
4. **Rendering** — Frame RGBA data arrives as `VideoEvent::FrameReady { rgba: Arc<Vec<u8>>, width, height }`, stored in `AppState`. The `video_canvas` widget (`widgets/video_canvas.rs`) renders it via a custom wgpu shader (`widgets/video_shader.rs`) for zero-copy Vulkan interop. This requires `ash` + `raw-window-handle` + `wgpu`.
5. **Lifecycle** — When the user navigates away from a video, `Deactivate` is sent. The worker stops rendering and the video frame is cleared.

The entire pipeline depends on `libmpv-sys` at build time and a working `libmpv` installation at runtime. Without it, video playback silently does nothing (the sender is `None`).

## Audio player

Audio playback uses `rodio` for output and `symphonia` (all codecs) for decoding, independent of mpv. The `AudioPlayer` is created in `AppState::new()` and is `None` if initialization fails (non-fatal). Commands: `PlayAudio`, `PauseAudio`, `StopAudio`. There is no seek or progress tracking for audio.

## GIF handling

GIF files are classified as `MediaType::Video`, not `MediaType::Image`. The `MediaType::Video::extensions()` list includes `"gif"`, and native image extensions do not include it. Two settings control behavior:

- `animate_gifs` — whether the preview animates GIFs
- `animate_gif_thumbnails` — whether grid thumbnails animate

At the file system level the `image` crate can decode GIF natively. The mpv path is also available if the installed mpv supports GIF demuxing.

## Two `detect_media_type` functions — beware

There are **two different** media type detection functions:

1. `MediaRegistry::determine_type()` in `media-sort-core/src/media_type.rs:87` — strict priority order (native image → native audio → mpv-discovered), uses the global OnceLock registry. Returns `None` for unknown extensions.
2. `detect_media_type()` in `media-sort-gui/src/state.rs:566` — simple linear scan of hardcoded extension lists from `MediaType::extensions()`. Defaults to `MediaType::Image` for unknown extensions.

The GUI scanner uses (2), metadata loading uses (1). This can cause mismatches if mpv discovers additional extensions at startup (e.g., a custom mpv build with extra demuxers). If you add formats, update both.

## Caching

| Cache | Type | Capacity | Purpose |
|-------|------|----------|---------|
| `thumbnail_cache` | `LruCache<PathBuf, Handle>` | 200 | Grid thumbnails |
| `image_cache` | `LruCache<PathBuf, Handle>` | 20 | Full-resolution preview images |
| `unsupported_files` | `HashSet<PathBuf>` | unbounded | Files that failed thumbnail generation; prevents retry spam |

When selection changes, next/previous images are preloaded into `image_cache`.

## build.rs — locale code generation

`crates/media-sort-core/build.rs` scans `resources/locale/` and generates `locales_codegen.rs` into `OUT_DIR`. It:
- Discovers available locales from subdirectory names
- Reads `# locale-name: <display name>` comments from each locale's `main.ftl`
- Generates `AVAILABLE_LOCALES`, `locale_display_name()`, and `load_ftl()` functions
- Emits `cargo:rerun-if-changed` for each locale file so rebuilds trigger on FTL changes

The generated code is never committed — it lives in the build output directory.

## Platform-specific code

| Feature | Windows | macOS | Linux |
|---------|---------|-------|-------|
| Open externally | `cmd /C start` | `open` | `xdg-open` |
| Trash | `platform/trash.rs` (windows impl) | `platform/trash.rs` (macos impl) | `platform/trash.rs` (freedesktop impl) |
| Windows integration | `integration_with_windows` setting, `#[cfg(target_os = "windows")]` | N/A | N/A |
| Config dir | `%APPDATA%` | `~/Library/Application Support` | `$XDG_CONFIG_HOME` |
| Update mechanism | `check_for_updates_on_startup` toggle in settings | (same) | (same) |

## Test coverage

Integration tests live in:
- `crates/media-sort-core/tests/core_tests.rs`
- `crates/media-sort-backend/tests/` — `filesystem_tests.rs`, `metadata_tests.rs`, `audio_tests.rs`
- `crates/media-sort-backend/tests/fixtures/` — test media files

The GUI crate has `#[cfg(test)]` unit tests in `app.rs` and `state.rs` but **no integration test files** under `crates/media-sort-gui/tests/`. GUI tests use instantiated `AppState` with `SettingsStore::default()`.

There is no test suite for `media-sort-gui`. To run the existing tests:
```bash
cargo test -p media-sort-core
cargo test -p media-sort-backend
cargo test -p media-sort-gui          # runs #[cfg(test)] modules only
```

## Key dependencies beyond Rust std

| Crate | Used for |
|-------|----------|
| `lucide-icons` | Icon font bundled at compile time via `.font()` |
| `rfd` | Native file/folder picker dialogs |
| `lru` | LRU caches |
| `wgpu` + `winit` + `ash` + `raw-window-handle` | Vulkan interop for video frames |
| `symphonia` (all codecs) + `rodio` | Audio decoding and output |
| `kamadak-exif` + `id3` + `metaflac` + `mp4ameta` | Metadata extraction |
| `notify` + `notify-debouncer-mini` | Filesystem watcher |
| `walkdir` | Media file scanning |
| `trash` | Cross-platform delete-to-trash |
| `fluent` + `fluent-bundle` + `unic-langid` | i18n |

## Maintaining documentation

### AGENTS.md
When you add a new module, dependency, build step, architectural decision, or test infrastructure, update this file. When you change how an existing system works (e.g., the video pipeline, settings persistence, media type detection), update the relevant section.

### Help pages (`docs/`)
The `docs/` directory is the GitHub Pages site at `imagesort.org`. As of v3.0 the help pages are outdated and intentionally kept for historical reference. If you update the user-facing UI or workflow, update the corresponding help page:
- `docs/help.md` (English)
- `docs/help/help.de.md` (German)

### Documentation website (`website/`)
The `website/` directory is the new Astro/Starlight documentation site deployed to `gh-pages`. When adding or updating docs content, work in this directory. The dev server runs with `astro dev --background`.

### Locale files
When you add user-facing strings, add entries to all three locale files (`resources/locale/{en,de,ja}/main.ftl`). The build script detects changes automatically.**

## Operational Capabilities & Environment

### Environment-Aware Execution Protocol
1. **Tool Introspection:** At the start of a task, inspect your active environment schema to identify any available specialized tools (e.g., semantic search providers, codebase indexers, memory managers, or specific file-parsing utilities).
2. **Conditional Prioritization:** - **If matching tools exist:** You must prioritize them over generic terminal commands (`bash`, `sh`) or custom manual scripts to achieve the task efficiently.
   - **If no matching tools exist:** Fall back freely to standard shell commands, core utilities, or manual file discovery methods to complete the objective.
3. **No Redundant Execution:** Do not manually replicate a task via command-line tools (like raw `grep` or manual script compilation) if an explicit environment tool is already configured and exposed to your context to handle it.
