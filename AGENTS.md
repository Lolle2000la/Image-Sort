# AGENTS.md

## Build & verify

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all --
cargo clippy --workspace --all-targets -- -D warnings
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
```

`media-sort-core` must never depend on `media-sort-backend` or `media-sort-gui`.

## libmpv

The GUI binary links libmpv at build time and loads it at runtime. The system must have `libmpv-dev` (or equivalent) installed to compile. Without it `cargo build` fails on the `libmpv-sys` crate.

At startup the app queries mpv for supported formats via `demuxer-lavf-list` and builds the media type registry dynamically. Video/audio support depends entirely on the installed mpv version.

## Configuration

Runtime config lives at `$CONFIG_DIR/media-sort/config.toml` (TOML). `$CONFIG_DIR` resolves via the `dirs` crate: `$XDG_CONFIG_HOME` (Linux), `~/Library/Application Support` (macOS), `%APPDATA%` (Windows).

On first launch the app silently migrates settings from legacy JSON paths. In debug builds (`cfg!(debug_assertions)`) the migration reads `debug_config.json` instead of `config.json`:
- `$CONFIG_DIR/Image Sort/config.json` or `debug_config.json` (old WPF app)
- `$CONFIG_DIR/media-sort/config.json` or `debug_config.json`

When the `UI_TEST` environment variable is set to a non-empty value, local test paths override all of the above (`ui_test_config.toml` / `ui_test_config.json`). This is handled in `media-sort-core/src/settings/store.rs`.

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
