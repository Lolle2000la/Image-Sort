# Flatpak / Flathub packaging

This directory holds everything needed to build and submit **Media Sort** as a
Flatpak on [Flathub](https://flathub.org):

| File | Purpose |
| --- | --- |
| `app.mediasort.MediaSort.yml` | The Flatpak build manifest (this is the file submitted to Flathub). |
| `cargo-sources.json` | Vendored Rust crates, generated from `Cargo.lock` (Flathub builds offline). |

Upstream metadata lives with the other Linux packaging files:

- `packaging/linux/app.mediasort.MediaSort.metainfo.xml` — AppStream metadata (Flathub store listing).
- `packaging/linux/app.mediasort.MediaSort.desktop` — desktop entry.
- `packaging/linux/icon.svg` — the icon (installed as `app.mediasort.MediaSort.svg`).

## Application id

The id is **`app.mediasort.MediaSort`**, derived from the `mediasort.app` domain
(declared in `website/public/CNAME`). This makes the app eligible for the
Flathub *Verified* badge: after the first build, add the verification token
Flathub gives you to `website/public/.well-known/org.flathub.VerifiedApps.txt`
and click *Verify* on the app's Flathub page. The id is also the value of the
window's Wayland/X11 app id at runtime (see below), so keep the metadata file
names, the `<id>` in the metainfo, the `Icon=` in the desktop file and the
`app-id` in the manifest in sync.

## How it is built

- **Runtime/SDK:** `org.freedesktop.Platform` / `org.freedesktop.Sdk` `26.08`
  (Freedesktop). Flathub requires the latest runtime at submission time; bump
  this and re-check when a new yearly branch lands.
- **Rust:** the `org.freedesktop.Sdk.Extension.rust-stable` SDK extension.
- **libmpv:** not part of the runtime, so `mpv` (v0.41.0) is built from source
  in a module, together with its two unconditional build dependencies that the
  runtime does not ship — `libass` and `libplacebo`. Everything the three link
  against comes from the runtime/SDK (ffmpeg libraries, dav1d, libva,
  PulseAudio/PipeWire/ALSA, freetype/fontconfig/harfbuzz/fribidi,
  vulkan/shaderc/lcms2), so no codec stack is built here. Media Sort renders
  frames itself through libmpv's software render API, so mpv needs no
  display/GPU output of its own.
- **Hardware decoding (VA-API):** mpv gates VA-API behind one of its display
  back-ends, and both of those are missing a dependency in the runtime — its
  Wayland back-end wants `wayland-protocols` and its X11 back-end wants
  `libXpresent`. Both are built as small modules so `hwdec=auto-copy` works on
  Wayland and X11 sessions alike (software decoding when a GPU cannot do it).
- **ffmpeg CLI:** the runtime ships `/usr/bin/ffmpeg`, which is what
  `media-sort-backend`'s `find_ffmpeg()` picks up for video thumbnails.
- **The self-updater (`velopack` feature) is intentionally not built** — Flatpak
  owns updates. That is also why the manifest needs no network access.

## Permissions

The app is a media *sorter*: it moves, renames, copies and deletes files in a
folder the user opens, including into pinned destination folders elsewhere.
That is file-management work, so it needs real filesystem access rather than the
document-portal view of a single chosen folder, hence `--filesystem=home` plus
the removable-media mount points. There is no `--filesystem=host` and no
network access. Device/audio access is `--device=dri` (wgpu/Vulkan rendering and
VA-API hardware decoding) and `--socket=pulseaudio`.

## Building locally

Requires `flatpak-builder` (or the `org.flatpak.Builder` Flatpak) and the SDK:

```bash
flatpak install flathub org.freedesktop.Sdk//26.08 \
  org.freedesktop.Sdk.Extension.rust-stable//26.08
```

Then, from the repository root:

```bash
flatpak-builder --user --install --force-clean build-dir flatpak/app.mediasort.MediaSort.yml
flatpak run app.mediasort.MediaSort
```

The manifest's app source points at the **released tag** (`v3.0.0`), so for a
local build of uncommitted work, temporarily point that source at this checkout
(e.g. a `type: dir` or local `type: git` source) instead.

Lint the manifest before/after changes:

```bash
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest flatpak/app.mediasort.MediaSort.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo build-dir/repo
```

## Submitting to Flathub

Flathub hosts the manifest in its own repository and builds it on Flathub's
infrastructure — nothing here is published automatically.

1. **Cut a stable release that includes this packaging.** Flathub's stable
   repository accepts only stable software, and the manifest installs the
   metadata from the tagged source tree, so the source tag must contain
   `packaging/linux/app.mediasort.MediaSort.*` and `flatpak/`. `v3.0.0` predates
   them, so cut a follow-up (e.g. `scripts/release.sh 3.0.1`).
2. Point the manifest's `media-sort` source at that tag *and* its commit
   (Flathub wants both) and add the matching `<release>` to the metainfo.
3. Regenerate `cargo-sources.json` whenever `Cargo.lock` changes (see below).
4. Fork [`flathub/flathub`](https://github.com/flathub/flathub), create a branch
   named `app.mediasort.MediaSort`, add `app.mediasort.MediaSort.yml` and
   `cargo-sources.json` at the repository root, and open a pull request.
5. Flathub reviews the submission. After it is merged the manifest moves to
   `flathub/app.mediasort.MediaSort`, builds start automatically, and future
   updates are a pull request to that repository.

## Regenerating `cargo-sources.json`

Run this after any change to `Cargo.lock`:

```bash
python3 flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json
```

`flatpak-cargo-generator.py` comes from
[flatpak-builder-tools](https://github.com/flatpak/flatpak-builder-tools)
(`cargo/`, pinned in `.github/workflows/ci.yml`), and needs `aiohttp` and
`tomlkit`. CI regenerates it and fails if the committed file differs, so it
cannot silently drift from `Cargo.lock`.

## Version maintenance

Every release touches the Flatpak packaging:

- bump `tag` (and `commit`) of the `media-sort` source in the manifest;
- add a matching `<release version="…" date="…"/>` to the metainfo;
- regenerate `cargo-sources.json` if `Cargo.lock` changed.

The `x-checker-data` blocks let Flathub's external data checker open automated
bump PRs for mpv, libass, libplacebo and the app itself.

The CI job `flatpak-cargo-sources` regenerates `flatpak/cargo-sources.json` from
`Cargo.lock` and fails if it does not match the committed file, so a dependency
bump that forgets to regenerate it is caught before the Flathub build.

## Known follow-ups

- **Screenshots:** `website/public/screenshots/*.png` are stills extracted from
  the rendered demo videos (the automation HUD was painted out). The metainfo
  references them at `https://mediasort.app/screenshots/…`, so the **website
  must be deployed before the Flathub build** — Flathub mirrors the URLs at
  build time and the build fails if they are unreachable.
- **Portals:** deletion goes through the FreeDesktop trash (`trash` crate)
  rather than the `org.freedesktop.portal.Trash` portal, and
  *reveal in file manager* shells out to `dbus-send`/`org.freedesktop.FileManager1`
  (falling back to `xdg-open`). Both work under `--filesystem=home`, but the
  portals are the more sandbox-friendly alternative if reviewers ask.
