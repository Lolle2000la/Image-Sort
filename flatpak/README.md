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
  in a module. The freedesktop runtime already provides everything libmpv links
  against (ffmpeg libraries, dav1d, libva, PulseAudio/PipeWire/ALSA, Wayland),
  so no codec stack is built here. Media Sort renders frames itself through
  libmpv's software render API, so mpv needs no display/GPU output of its own.
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

1. **Cut a stable release first.** Flathub's stable repository accepts only
   stable software, so the app source must point at a non-prerelease tag
   (`v3.0.0`, not an `-rc`/`-alpha` tag). Run `scripts/release.sh 3.0.0`.
2. Fill in the released `commit:` in the `media-sort` source of the manifest
   (Flathub wants both `tag` and `commit`) and bump the `<release>` entry in the
   metainfo if needed.
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
(`cargo/`), and needs `aiohttp` and `tomlkit`.

## Version maintenance

Every release touches the Flatpak packaging:

- bump `tag` (and `commit`) of the `media-sort` source in the manifest;
- add a matching `<release version="…" date="…"/>` to the metainfo;
- regenerate `cargo-sources.json` if `Cargo.lock` changed.

The `x-checker-data` blocks let Flathub's external data checker open automated
bump PRs for mpv and for the app itself.

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
- **`rust-version`:** the workspace's MSRV (`1.99`) must not exceed the Rust
  version shipped by the `rust-stable` SDK extension (currently 1.98.x on both
  `25.08` and `26.08`). Wait for Flathub to rebuild the extension, or lower the
  declared MSRV, before the first build.
