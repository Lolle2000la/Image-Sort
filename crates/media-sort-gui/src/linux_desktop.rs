//! Linux desktop integration for the window icon and Wayland app id.
//!
//! Wayland compositors never read the window icon from the client; they resolve
//! a toplevel's icon (and display name) through the desktop entry whose file
//! name matches the toplevel's `app_id`, which winit sets from the window's
//! `application_id`. An AppImage that was never "integrated" by appimaged,
//! AppImageLauncher or Gear Lever has no such entry in the XDG data
//! directories, so the shell falls back to a generic icon even though the
//! AppImage itself ships a `MediaSort.desktop` and the icon files.
//!
//! [`integration`] closes that gap at startup:
//!
//! 1. If a desktop entry anywhere in the XDG data directories already points at
//!    the running executable/AppImage (any integration tool, any file name),
//!    its file name is adopted as the app id and nothing is written.
//! 2. Otherwise, if no `MediaSort.desktop` exists yet, a hidden
//!    (`NoDisplay=true`) entry plus the app icons are written into the user's
//!    XDG data directories - the same locations the integration tools use, so
//!    every compositor resolves the icon without needing the newer
//!    `xdg-toplevel-icon-v1` protocol (which the pinned winit cannot speak).
//!
//! The entry is hidden so nothing appears in application menus; it exists only
//! to give the shell an icon, display name and `StartupWMClass` for the window.
//! Only entries carrying the [`MARKER_KEY`] marker are ever rewritten or
//! removed; packaged and hand-written entries are never touched. Writes use
//! [`atomic_write`], so a crash cannot leave a truncated entry behind.
//!
//! While the app runs, instances share a lock file
//! (`$XDG_DATA_HOME/media-sort/desktop-integration.lock`); when the last
//! instance exits, the provisioned entry and icon are removed again so a
//! cleanly closed app leaves no integration behind. This is best-effort: a
//! crash (or SIGKILL) skips the cleanup and leaves the hidden files for the
//! next run to reuse or remove, and the lock file itself stays behind
//! (removing lock files while another process may be opening them is racy).

use std::fs::{self, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use media_sort_core::path_utils::{atomic_write, paths_equal};

/// Desktop file name (without extension) used by the packaged releases; also
/// the window's `application_id` whenever no externally integrated entry is
/// found. Must match `MediaSort.desktop` inside the AppImage (vpk `-u
/// MediaSort`) and `StartupWMClass=MediaSort` from the release workflow.
pub const DESKTOP_ID: &str = "MediaSort";

/// Icon name referenced by the provisioned entry. Matches the icon name the
/// AppImage ships (`usr/share/icons/hicolor/scalable/apps/MediaSort.svg`).
const ICON_NAME: &str = "MediaSort";

/// Marks the desktop entry as provisioned by this module so foreign files with
/// the same name are never overwritten or removed.
const MARKER_KEY: &str = "X-MediaSort-Provisioned";

/// The app icon installed into the user's `hicolor` icon theme: the same
/// scalable icon (`packaging/linux/icon.svg`) the AppImage ships, so the two
/// can never drift apart. GNOME (librsvg) and KDE (QtSvg) both render theme
/// SVGs natively - Breeze itself ships SVG-only icons.
const ICON: &[u8] = include_bytes!("../../../packaging/linux/icon.svg");

/// Desktop files are tiny; anything larger is not trusted to be one.
const MAX_DESKTOP_FILE_BYTES: u64 = 64 * 1024;

/// The window's `application_id` plus the guard that removes the provisioned
/// files again when the last running instance exits.
pub struct DesktopIntegration {
    /// The `application_id` to set on the window.
    pub app_id: String,
    /// Kept alive for the process lifetime; dropping it cleans up (best
    /// effort) when no other instance is still running.
    _guard: Option<IntegrationGuard>,
}

/// Resolves the window `application_id` for this process and makes the app
/// icon discoverable by desktop shells when the usual integration is missing.
///
/// Keep the returned value alive until the app exits - its guard holds the
/// shared lock that lets the last instance remove the provisioned files.
/// `$APPIMAGE` (when running from an AppImage), `$XDG_DATA_HOME` and
/// `$XDG_DATA_DIRS` are read from the environment; failures to write are
/// logged and never prevent the app from starting.
pub fn integration() -> DesktopIntegration {
    let data_home = dirs::data_local_dir();
    let data_dirs = xdg_data_dirs();
    let appimage = std::env::var_os("APPIMAGE").map(PathBuf::from);
    let exe = std::env::current_exe().ok();

    // Acquire the shared lock before provisioning so an instance that is
    // exiting right now cannot delete the files this run is about to write.
    let guard = data_home
        .as_deref()
        .and_then(|home| match IntegrationGuard::acquire(home) {
            Ok(guard) => Some(guard),
            Err(err) => {
                tracing::debug!(%err, "desktop integration cleanup is disabled for this run");
                None
            }
        });

    let app_id = resolve_and_provision(
        data_home.as_deref(),
        &data_dirs,
        appimage.as_deref(),
        exe.as_deref(),
    );

    DesktopIntegration {
        app_id,
        _guard: guard,
    }
}

/// The platform-independent heart of [`integration`], split out so tests can
/// drive it with temporary directories.
fn resolve_and_provision(
    data_home: Option<&Path>,
    data_dirs: &[PathBuf],
    appimage: Option<&Path>,
    exe: Option<&Path>,
) -> String {
    let entry_file = format!("{DESKTOP_ID}.desktop");
    let own_entry = data_home.map(|home| home.join("applications").join(&entry_file));
    let own_is_ours = own_entry
        .as_deref()
        .is_some_and(|entry| entry.is_file() && is_managed_by_us(entry));
    // A packaged entry (e.g. installed from a distribution package) must win
    // over anything we would write under the same name.
    let foreign_entry = data_dirs
        .iter()
        .any(|dir| dir.join("applications").join(&entry_file).is_file());

    if own_is_ours
        && foreign_entry
        && let Some(own_entry) = own_entry.as_deref()
    {
        // Our provisioned entry would shadow the packaged one; drop it so the
        // packaged entry's icon and running state are used instead.
        if let Err(err) = fs::remove_file(own_entry) {
            tracing::warn!(path = %own_entry.display(), %err, "could not remove provisioned desktop entry");
        }
    }

    // Adopt an existing desktop entry that launches this very file. This
    // covers appimaged (`appimagekit-*.desktop`), AppImageLauncher, Gear
    // Lever, `xdg-desktop-menu install` and user-created entries alike,
    // without having to guess any naming scheme.
    if let Some(appimage) = appimage
        && let Some(id) = find_entry_for_appimage(data_home, data_dirs, appimage)
    {
        tracing::debug!(desktop_id = %id, "adopted existing desktop entry for the running executable");
        return id;
    }

    let Some(data_home) = data_home else {
        return DESKTOP_ID.to_string();
    };

    // Hand-written or installed by another tool - never touch it. Its app id
    // still matches our window, so the icon resolution is up to it.
    if own_entry.as_deref().is_some_and(Path::exists) && !own_is_ours {
        return DESKTOP_ID.to_string();
    }

    if foreign_entry {
        return DESKTOP_ID.to_string();
    }

    let Some(target) = appimage.or(exe) else {
        return DESKTOP_ID.to_string();
    };
    let Some(exec) = exec_argument(target) else {
        tracing::debug!("skipping desktop entry provisioning: path cannot be represented safely");
        return DESKTOP_ID.to_string();
    };

    if let Err(err) = provision(data_home, &exec) {
        tracing::warn!(%err, "could not provision desktop entry for the window icon");
    }

    DESKTOP_ID.to_string()
}

/// Looks for a `*.desktop` file in the XDG application directories whose
/// `Exec`/`TryExec` launches `appimage`; returns its file name (the desktop
/// entry id the compositor will use).
fn find_entry_for_appimage(
    data_home: Option<&Path>,
    data_dirs: &[PathBuf],
    appimage: &Path,
) -> Option<String> {
    let mut applications_dirs: Vec<PathBuf> = Vec::new();
    if let Some(data_home) = data_home {
        applications_dirs.push(data_home.join("applications"));
    }
    applications_dirs.extend(data_dirs.iter().map(|dir| dir.join("applications")));

    for dir in applications_dirs {
        for file in desktop_files(&dir) {
            let Some(contents) = read_capped(&file) else {
                continue;
            };
            if exec_matches(&contents, appimage)
                && let Some(id) = file.file_stem().and_then(|stem| stem.to_str())
            {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// `*.desktop` files directly inside `dir`, sorted for deterministic results.
fn desktop_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "desktop"))
        .collect();
    files.sort();
    files
}

/// Reads a file only when it is small enough to plausibly be a desktop entry.
fn read_capped(path: &Path) -> Option<String> {
    if fs::metadata(path).ok()?.len() > MAX_DESKTOP_FILE_BYTES {
        return None;
    }
    fs::read_to_string(path).ok()
}

/// True when any `Exec`/`TryExec` entry in the desktop file refers to `target`.
fn exec_matches(contents: &str, target: &Path) -> bool {
    contents.lines().any(|line| {
        candidate_paths(line).iter().any(|candidate| {
            let candidate = Path::new(candidate);
            candidate.is_absolute() && paths_equal(candidate, target)
        })
    })
}

/// Extracts the executable paths a desktop-file line may contain. `Exec` is a
/// command line (tokenized with quoting/escaping); `TryExec` is a single path
/// string (where `\s` encodes a space, as written by AppImageKit's integration
/// script).
fn candidate_paths(line: &str) -> Vec<String> {
    if let Some(value) = line.strip_prefix("Exec=") {
        exec_tokens(value)
    } else if let Some(value) = line.strip_prefix("TryExec=") {
        vec![unescape_value(value.trim().trim_matches('"'))]
    } else {
        Vec::new()
    }
}

/// Splits an `Exec` value into arguments, honoring double quotes and the
/// `\s`/`\n`/`\t`/`\r`/`\\` escapes used by desktop-file config parsers.
fn exec_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_token = false;
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let escaped = match chars.next() {
                    Some('s') => ' ',
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some(other) => other,
                    None => '\\',
                };
                current.push(escaped);
                has_token = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                has_token = true;
            }
            _ if ch.is_whitespace() && !in_quotes => {
                if has_token {
                    tokens.push(std::mem::take(&mut current));
                    has_token = false;
                }
            }
            _ => {
                current.push(ch);
                has_token = true;
            }
        }
    }
    if has_token {
        tokens.push(current);
    }
    tokens
}

/// Unescapes a desktop-file value string (`\s` is a space, and so on).
fn unescape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        out.push(match chars.next() {
            Some('s') => ' ',
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some(other) => other,
            None => '\\',
        });
    }
    out
}

/// True when the desktop entry was written by this module.
fn is_managed_by_us(path: &Path) -> bool {
    read_capped(path).is_some_and(|contents| {
        contents.lines().any(|line| {
            line.trim_start()
                .strip_prefix(MARKER_KEY)
                .and_then(|rest| rest.strip_prefix('='))
                .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
        })
    })
}

/// Writes the hidden desktop entry and the app icons below `data_home`
/// (`$XDG_DATA_HOME`, i.e. `~/.local/share`).
fn provision(data_home: &Path, exec: &str) -> io::Result<()> {
    let applications = data_home.join("applications");
    fs::create_dir_all(&applications)?;
    write_if_changed(
        &applications.join(format!("{DESKTOP_ID}.desktop")),
        desktop_entry_contents(exec).as_bytes(),
    )?;

    let icon_dir = data_home
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps");
    fs::create_dir_all(&icon_dir)?;
    write_if_changed(&icon_dir.join(format!("{ICON_NAME}.svg")), ICON)?;
    Ok(())
}

/// A shared lock on `$XDG_DATA_HOME/media-sort/desktop-integration.lock`, held
/// for the process lifetime. The last instance to exit upgrades the lock to
/// exclusive and removes the provisioned files.
struct IntegrationGuard {
    lock: fs::File,
    data_home: PathBuf,
}

impl IntegrationGuard {
    fn acquire(data_home: &Path) -> io::Result<Self> {
        let dir = data_home.join("media-sort");
        fs::create_dir_all(&dir)?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("desktop-integration.lock"))?;

        // A concurrent exit briefly holds the exclusive lock; retry instead of
        // losing the cleanup for this whole run.
        for _ in 0..5 {
            match lock.try_lock_shared() {
                Ok(()) => {
                    return Ok(Self {
                        lock,
                        data_home: data_home.to_path_buf(),
                    });
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(TryLockError::Error(err)) => return Err(err),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "desktop integration lock is busy",
        ))
    }
}

impl Drop for IntegrationGuard {
    fn drop(&mut self) {
        match self.lock.try_lock() {
            Ok(()) => cleanup_provisioned(&self.data_home),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(err)) => {
                tracing::debug!(%err, "could not probe the desktop integration lock");
            }
        }
    }
}

/// Removes the provisioned desktop entry and icon again. Only files this
/// module owns are touched: the entry must carry the marker, and the icon must
/// be byte-for-byte the one we install (a user-replaced icon is kept). A
/// foreign `MediaSort.desktop` (packaged or hand-written) also keeps the icon,
/// since it may reference it.
fn cleanup_provisioned(data_home: &Path) {
    let entry = data_home
        .join("applications")
        .join(format!("{DESKTOP_ID}.desktop"));
    let entry_exists = entry.is_file();
    let entry_is_foreign = entry_exists && !is_managed_by_us(&entry);

    if entry_exists
        && !entry_is_foreign
        && let Err(err) = fs::remove_file(&entry)
    {
        tracing::debug!(path = %entry.display(), %err, "could not remove provisioned desktop entry");
    }

    if entry_is_foreign {
        return;
    }
    let icon = data_home
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps")
        .join(format!("{ICON_NAME}.svg"));
    if fs::read(&icon).is_ok_and(|bytes| bytes == ICON) {
        let _ = fs::remove_file(&icon);
    }
}

/// The provisioned desktop entry. `NoDisplay=true` keeps it out of application
/// menus; shells still use it for window/icon matching.
fn desktop_entry_contents(exec: &str) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Media Sort\n\
         Exec={exec} %U\n\
         Icon={ICON_NAME}\n\
         Terminal=false\n\
         NoDisplay=true\n\
         StartupWMClass={DESKTOP_ID}\n\
         Categories=Utility;\n\
         {MARKER_KEY}=true\n"
    )
}

/// Formats `path` as a single, quoted `Exec` argument.
///
/// The value passes through two parsers: the key-file layer that reads the
/// value (`\\`, `\s`, `\n`, `\t`, `\r` are its only backslash escapes - any
/// other `\x` makes the whole file unreadable for GLib) and the shell-style
/// tokenizer that splits `Exec`. `"` and `\` are therefore escaped twice:
/// once for the shell layer, once for the key-file layer.
///
/// Paths containing `%`, `$`, backticks or control characters are rejected.
/// Control characters would corrupt the file. `%` has no portable encoding
/// either: doubling it to `%%` passes `desktop-file-validate`, but GLib's
/// load-time check resolves the *raw* `argv[0]` (including the unexpanded
/// `%%`) against the filesystem and discards the entry, while a literal `%`
/// is an invalid field code for the validator. `$` and backticks cannot be
/// escaped portably: GLib's key-file layer rejects `\$`/`` \` `` outright,
/// KDE's parser keeps the backslash inside quotes literally, and leaving them
/// unescaped fails `desktop-file-validate`. Such paths keep the system's
/// default icon.
fn exec_argument(path: &Path) -> Option<String> {
    let path = path.to_str()?;
    if path.is_empty()
        || path
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '%' | '$' | '`'))
    {
        return None;
    }

    let mut quoted = String::with_capacity(path.len() + 2);
    quoted.push('"');
    for ch in path.chars() {
        match ch {
            // Shell layer `\"`, plus one more backslash for the key-file layer.
            '"' => quoted.push_str(r#"\\""#),
            // Shell layer `\\`, plus one more backslash per backslash.
            '\\' => quoted.push_str(r"\\\\"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    Some(quoted)
}

/// Writes `contents` to `path` only when they differ, so repeated startups do
/// not touch mtimes (icon themes watch these directories).
fn write_if_changed(path: &Path, contents: &[u8]) -> io::Result<()> {
    if fs::read(path).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    atomic_write(path, contents)
}

/// `$XDG_DATA_DIRS` with the spec default, filtered to absolute paths.
fn xdg_data_dirs() -> Vec<PathBuf> {
    let from_env = std::env::var_os("XDG_DATA_DIRS").filter(|value| !value.is_empty());
    parse_data_dirs(from_env.as_deref())
}

/// Splits an `$XDG_DATA_DIRS` value; `None` uses the spec default. Relative
/// entries are ignored, as required by the XDG base directory specification.
fn parse_data_dirs(value: Option<&std::ffi::OsStr>) -> Vec<PathBuf> {
    let dirs: Vec<PathBuf> = match value {
        Some(value) => std::env::split_paths(value).collect(),
        None => vec![
            PathBuf::from("/usr/local/share"),
            PathBuf::from("/usr/share"),
        ],
    };
    dirs.into_iter().filter(|dir| dir.is_absolute()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary `$XDG_DATA_HOME` that cleans up after itself.
    struct TempDataHome(PathBuf);

    impl TempDataHome {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "media-sort-linux-desktop-{name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp data home");
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDataHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_file(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, contents).expect("write file");
    }

    fn installed_entry_for(appimage: &Path, exec_key: &str) -> String {
        format!(
            "[Desktop Entry]\nType=Application\nName=Media Sort\n{exec_key}={} %U\n",
            appimage.display()
        )
    }

    #[test]
    fn adopts_a_desktop_entry_that_launches_the_appimage() {
        let home = TempDataHome::new("adopt");
        let appimage = home.path().join("Foo.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        write_file(
            &home.path().join("applications/appimagekit-Foo.desktop"),
            &installed_entry_for(&appimage, "Exec"),
        );

        let id = resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        assert_eq!(id, "appimagekit-Foo");
        assert!(!home.path().join("applications/MediaSort.desktop").exists());
    }

    #[test]
    fn adopts_a_desktop_entry_that_launches_the_appimage_via_an_env_wrapper() {
        let home = TempDataHome::new("env-wrapper");
        let appimage = home.path().join("Foo.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        write_file(
            &home.path().join("applications/custom-name.desktop"),
            &format!(
                "[Desktop Entry]\nExec=env APPIMAGELAUNCHER_DISABLE=1 \"{}\" %U\n",
                appimage.display()
            ),
        );

        let id = resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        assert_eq!(id, "custom-name");
    }

    #[test]
    fn adopts_a_try_exec_entry_with_escaped_spaces() {
        let home = TempDataHome::new("tryexec");
        let subdir = home.path().join("My Apps");
        fs::create_dir_all(&subdir).expect("create subdir");
        let appimage = subdir.join("Foo.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        let escaped = appimage.display().to_string().replace(' ', "\\s");
        write_file(
            &home.path().join("applications/foo.desktop"),
            &format!("[Desktop Entry]\nTryExec={escaped}\n"),
        );

        let id = resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        assert_eq!(id, "foo");
    }

    #[test]
    fn adopts_an_entry_matching_a_symlinked_appimage() {
        let home = TempDataHome::new("symlink");
        let real = home.path().join("real.AppImage");
        fs::write(&real, b"appimage").expect("write appimage");
        let link = home.path().join("link.AppImage");
        std::os::unix::fs::symlink(&real, &link).expect("create symlink");
        write_file(
            &home.path().join("applications/foo.desktop"),
            &installed_entry_for(&real, "Exec"),
        );

        let id = resolve_and_provision(Some(home.path()), &[], Some(&link), None);

        assert_eq!(id, "foo");
    }

    #[test]
    fn provisions_a_hidden_entry_and_icons_when_not_integrated() {
        let home = TempDataHome::new("provision");
        let appimage = home.path().join("MediaSort-linux-x64.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");

        let id = resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        assert_eq!(id, DESKTOP_ID);
        let contents =
            fs::read_to_string(home.path().join("applications/MediaSort.desktop")).unwrap();
        assert!(contents.contains(&format!("Exec=\"{}\" %U", appimage.display())));
        assert!(contents.contains("Icon=MediaSort"));
        assert!(contents.contains("NoDisplay=true"));
        assert!(contents.contains("StartupWMClass=MediaSort"));
        assert!(contents.contains("X-MediaSort-Provisioned=true"));
        assert!(
            home.path()
                .join("icons/hicolor/scalable/apps/MediaSort.svg")
                .is_file()
        );
    }

    #[test]
    fn provisions_for_the_plain_executable_without_an_appimage() {
        let home = TempDataHome::new("plain-exe");
        let exe = home.path().join("target/debug/media-sort-gui");

        let id = resolve_and_provision(Some(home.path()), &[], None, Some(&exe));

        assert_eq!(id, DESKTOP_ID);
        let contents =
            fs::read_to_string(home.path().join("applications/MediaSort.desktop")).unwrap();
        assert!(contents.contains(&format!("Exec=\"{}\" %U", exe.display())));
    }

    #[test]
    fn refreshes_the_provisioned_entry_when_the_executable_moves() {
        let home = TempDataHome::new("refresh");
        let old = home.path().join("v1/MediaSort.AppImage");
        let new = home.path().join("v2/MediaSort.AppImage");
        write_file(&old, "old");
        write_file(&new, "new");

        assert_eq!(
            resolve_and_provision(Some(home.path()), &[], Some(&old), None),
            DESKTOP_ID
        );
        assert_eq!(
            resolve_and_provision(Some(home.path()), &[], Some(&new), None),
            DESKTOP_ID
        );

        let contents =
            fs::read_to_string(home.path().join("applications/MediaSort.desktop")).unwrap();
        assert!(contents.contains(&format!("Exec=\"{}\" %U", new.display())));
        assert!(!contents.contains(&format!("Exec=\"{}\" %U", old.display())));
    }

    #[test]
    fn does_not_shadow_a_packaged_desktop_entry() {
        let home = TempDataHome::new("shadow-system");
        let system_data = home.path().join("system-data");
        write_file(
            &system_data.join("applications/MediaSort.desktop"),
            "[Desktop Entry]\nType=Application\nName=Media Sort\nExec=media-sort-gui\nIcon=MediaSort\n",
        );
        let appimage = home.path().join("MediaSort-linux-x64.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");

        let id = resolve_and_provision(Some(home.path()), &[system_data], Some(&appimage), None);

        assert_eq!(id, DESKTOP_ID);
        assert!(!home.path().join("applications/MediaSort.desktop").exists());
    }

    #[test]
    fn leaves_a_hand_written_user_entry_alone() {
        let home = TempDataHome::new("user-entry");
        let user_entry = home.path().join("applications/MediaSort.desktop");
        let custom = "[Desktop Entry]\nType=Application\nName=Custom\nExec=/opt/custom\n";
        write_file(&user_entry, custom);

        let id = resolve_and_provision(Some(home.path()), &[], None, None);

        assert_eq!(id, DESKTOP_ID);
        assert_eq!(fs::read_to_string(&user_entry).unwrap(), custom);
        assert!(!home.path().join("icons").exists());
    }

    #[test]
    fn removes_its_own_entry_once_a_packaged_one_appears() {
        let home = TempDataHome::new("heal-shadow");
        let appimage = home.path().join("MediaSort-linux-x64.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        assert_eq!(
            resolve_and_provision(Some(home.path()), &[], Some(&appimage), None),
            DESKTOP_ID
        );
        assert!(home.path().join("applications/MediaSort.desktop").exists());

        let system_data = home.path().join("system-data");
        write_file(
            &system_data.join("applications/MediaSort.desktop"),
            "[Desktop Entry]\nType=Application\nName=Media Sort\nExec=media-sort-gui\n",
        );

        let id = resolve_and_provision(Some(home.path()), &[system_data], Some(&appimage), None);

        assert_eq!(id, DESKTOP_ID);
        assert!(!home.path().join("applications/MediaSort.desktop").exists());
    }

    #[test]
    fn skips_provisioning_when_no_executable_path_is_available() {
        let home = TempDataHome::new("no-target");

        let id = resolve_and_provision(Some(home.path()), &[], None, None);

        assert_eq!(id, DESKTOP_ID);
        assert!(!home.path().join("applications/MediaSort.desktop").exists());
    }

    #[test]
    fn returns_the_constant_id_without_a_data_home() {
        let home = TempDataHome::new("no-data-home");
        let appimage = home.path().join("MediaSort.AppImage");

        let id = resolve_and_provision(None, &[], Some(&appimage), None);

        assert_eq!(id, DESKTOP_ID);
    }

    #[test]
    fn ignores_desktop_entries_for_other_executables() {
        let home = TempDataHome::new("other-exe");
        write_file(
            &home.path().join("applications/other.desktop"),
            "[Desktop Entry]\nExec=\"/opt/other/App.AppImage\" %U\n",
        );
        let appimage = home.path().join("MediaSort.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");

        let id = resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        assert_eq!(id, DESKTOP_ID);
        assert!(home.path().join("applications/MediaSort.desktop").exists());
    }

    #[test]
    fn exec_tokens_handles_quoting_and_escapes() {
        assert_eq!(
            exec_tokens(r#"env FOO=bar "/srv/My Apps/Foo.AppImage" %U"#),
            vec!["env", "FOO=bar", "/srv/My Apps/Foo.AppImage", "%U"]
        );
        assert_eq!(
            exec_tokens(r"/srv/My\sApps/Foo.AppImage %U"),
            vec!["/srv/My Apps/Foo.AppImage", "%U"]
        );
        assert_eq!(
            exec_tokens(r#""/srv/quoted\"path/x" u"#),
            vec!["/srv/quoted\"path/x", "u"]
        );
    }

    #[test]
    fn exec_argument_quotes_and_escapes_special_characters() {
        // `/tmp/100 junk/My "App" \z` becomes, on disk:
        //   - `"` -> `\\"`  (shell `\"` + one backslash for the key-file layer)
        //   - `\` -> `\\\\` (shell `\\` + one backslash per backslash)
        assert_eq!(
            exec_argument(Path::new(r#"/tmp/100 junk/My "App" \z"#)).unwrap(),
            r#""/tmp/100 junk/My \\"App\\" \\\\z""#
        );
        assert_eq!(
            exec_argument(Path::new("/tmp/plain.AppImage")).unwrap(),
            "\"/tmp/plain.AppImage\""
        );
    }

    #[test]
    fn exec_argument_rejects_unrepresentable_characters() {
        assert_eq!(exec_argument(Path::new("")), None);
        assert_eq!(exec_argument(Path::new("/tmp/line\nbreak.AppImage")), None);
        assert_eq!(exec_argument(Path::new("/tmp/tab\tname.AppImage")), None);
        // No encoding of these is accepted by desktop-file-validate, GLib and
        // KDE at the same time (see `exec_argument`).
        assert_eq!(
            exec_argument(Path::new("/tmp/percent%20name.AppImage")),
            None
        );
        assert_eq!(exec_argument(Path::new("/tmp/dollar$name.AppImage")), None);
        assert_eq!(exec_argument(Path::new("/tmp/back`tick.AppImage")), None);
    }

    #[test]
    fn data_dirs_use_spec_defaults_and_drop_relative_paths() {
        use std::ffi::OsStr;

        assert_eq!(
            parse_data_dirs(None),
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        );
        assert_eq!(
            parse_data_dirs(Some(OsStr::new("/opt/share:relative:/usr/share"))),
            vec![PathBuf::from("/opt/share"), PathBuf::from("/usr/share")]
        );
    }

    #[test]
    fn cleanup_removes_the_provisioned_entry_and_icon() {
        let home = TempDataHome::new("cleanup");
        let appimage = home.path().join("MediaSort.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        cleanup_provisioned(home.path());

        assert!(!home.path().join("applications/MediaSort.desktop").exists());
        assert!(
            !home
                .path()
                .join("icons/hicolor/scalable/apps/MediaSort.svg")
                .exists()
        );
    }

    #[test]
    fn cleanup_keeps_foreign_entries_and_their_icons() {
        let home = TempDataHome::new("cleanup-foreign");
        let entry = home.path().join("applications/MediaSort.desktop");
        write_file(
            &entry,
            "[Desktop Entry]\nType=Application\nName=Custom\nExec=/opt/custom\n",
        );
        let icon = home
            .path()
            .join("icons/hicolor/scalable/apps/MediaSort.svg");
        write_file(&icon, "not our icon");

        cleanup_provisioned(home.path());

        assert!(entry.exists());
        assert!(icon.exists());
    }

    #[test]
    fn cleanup_keeps_a_user_replaced_icon() {
        let home = TempDataHome::new("cleanup-icon");
        let appimage = home.path().join("MediaSort.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);
        let icon = home
            .path()
            .join("icons/hicolor/scalable/apps/MediaSort.svg");
        write_file(&icon, "user replacement");

        cleanup_provisioned(home.path());

        assert!(!home.path().join("applications/MediaSort.desktop").exists());
        assert!(icon.exists());
    }

    #[test]
    fn only_the_last_instance_cleans_up() {
        let home = TempDataHome::new("cleanup-lock");
        let appimage = home.path().join("MediaSort.AppImage");
        fs::write(&appimage, b"appimage").expect("write appimage");
        resolve_and_provision(Some(home.path()), &[], Some(&appimage), None);

        let first = IntegrationGuard::acquire(home.path()).expect("first lock");
        let second = IntegrationGuard::acquire(home.path()).expect("second lock");

        drop(first);
        assert!(
            home.path().join("applications/MediaSort.desktop").exists(),
            "a still-running instance must keep the entry"
        );

        drop(second);
        assert!(!home.path().join("applications/MediaSort.desktop").exists());
        assert!(
            !home
                .path()
                .join("icons/hicolor/scalable/apps/MediaSort.svg")
                .exists()
        );
    }
}
