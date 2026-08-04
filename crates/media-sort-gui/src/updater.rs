use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use velopack::sources::GithubSource;
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};

use media_sort_core::path_utils::unique_temp_path;
use tokio::io::AsyncWriteExt;

const PUBKEY_ASC_BYTES: &[u8] = include_bytes!("../../../packaging/linux/pubkey.asc");

/// Hard cap on a single update package download. The feed controls the
/// declared content length, so a lying or compromised feed must not be able
/// to drive unbounded buffering; the declared size is enforced
/// incrementally as well (see `collect_capped`).
const MAX_PACKAGE_BYTES: u64 = 1 << 30; // 1 GiB

/// Hard cap on a single signature download. Signatures are a few KiB at
/// most; a feed must not be able to make us buffer arbitrarily more.
const MAX_SIGNATURE_SIZE: u64 = 1 << 20; // 1 MiB

fn verify_signature(
    public_key: &SignedPublicKey,
    package_path: &Path,
    sig_path: &Path,
) -> Result<(), String> {
    // Load the detached signature
    let sig_bytes =
        fs::read(sig_path).map_err(|e| format!("Failed to read signature file: {}", e))?;
    let sig = DetachedSignature::from_bytes(Cursor::new(sig_bytes))
        .map_err(|e| format!("Failed to parse PGP signature: {:?}", e))?;

    // Open file handle and memory-map the payload to ensure zero-copy validation
    let file =
        fs::File::open(package_path).map_err(|e| format!("Failed to open package file: {}", e))?;

    // SAFETY: We assume that the update package file is not concurrently modified or truncated
    // by another process in the system's temporary directory during signature verification.
    let mmap = unsafe { memmap2::Mmap::map(&file) }
        .map_err(|e| format!("Failed to memory-map package file: {}", e))?;

    // Verify signature against key and data
    sig.verify(public_key, &mmap)
        .map_err(|e| format!("PGP signature verification failed: {:?}", e))?;

    Ok(())
}

#[cfg(test)]
fn verify_signature_bytes(
    pubkey_bytes: &[u8],
    package_path: &Path,
    sig_path: &Path,
) -> Result<(), String> {
    let pubkey_str = std::str::from_utf8(pubkey_bytes)
        .map_err(|e| format!("Failed to parse public key bytes as UTF-8: {}", e))?;
    let (public_key, _) = SignedPublicKey::from_string(pubkey_str)
        .map_err(|e| format!("Failed to load PGP public key: {:?}", e))?;
    verify_signature(&public_key, package_path, sig_path)
}

fn verify_package_signature(package_path: &Path, sig_path: &Path) -> Result<(), String> {
    use std::sync::OnceLock;
    static PUBLIC_KEY: OnceLock<Option<SignedPublicKey>> = OnceLock::new();

    let public_key_opt = PUBLIC_KEY.get_or_init(|| {
        let pubkey_str = match std::str::from_utf8(PUBKEY_ASC_BYTES) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to parse public key bytes as UTF-8: {}", e);
                return None;
            }
        };
        match SignedPublicKey::from_string(pubkey_str) {
            Ok((public_key, _)) => Some(public_key),
            Err(e) => {
                tracing::error!("Failed to load PGP public key: {:?}", e);
                None
            }
        }
    });

    let public_key = public_key_opt
        .as_ref()
        .ok_or_else(|| "GPG public key is invalid or failed to parse".to_string())?;

    verify_signature(public_key, package_path, sig_path)
}

pub fn pre_startup_verify_packages() {
    let context = velopack::locator::LocationContext::Unknown;
    let Ok(locator) = velopack::locator::auto_locate_app_manifest(context) else {
        return;
    };
    let packages_dir = locator.get_packages_dir();
    if !packages_dir.exists() {
        return;
    }

    let mut invalid = false;
    match fs::read_dir(&packages_dir) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("Failed to read packages directory entry: {}", e);
                        invalid = true;
                        break;
                    }
                };
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "nupkg") {
                    let mut sig_file_name = path.file_name().unwrap_or_default().to_os_string();
                    sig_file_name.push(".sig");
                    let sig_path = path.with_file_name(sig_file_name);
                    if !sig_path.exists() {
                        tracing::warn!(
                            "Found unverified update package without signature: {:?}",
                            path
                        );
                        invalid = true;
                        break;
                    }

                    if let Err(e) = verify_package_signature(&path, &sig_path) {
                        tracing::error!("GPG verification failed for pending update: {}", e);
                        invalid = true;
                        break;
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!(
                "Failed to read packages directory '{:?}': {}. Purging directory for security.",
                packages_dir,
                e
            );
            invalid = true;
        }
    }

    if invalid {
        tracing::warn!("Purging packages directory due to signature verification failure.");
        let _ = fs::remove_dir_all(&packages_dir);
        let _ = fs::create_dir_all(&packages_dir);
    }
}

const GITHUB_REPO_ID: u64 = 119281525;

#[derive(serde::Deserialize)]
struct GithubRepoMetadata {
    html_url: String,
}

async fn fetch_canonical_repo_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("media-sort-gui-updater")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let url = format!("https://api.github.com/repositories/{}", GITHUB_REPO_ID);
    let metadata: GithubRepoMetadata = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(metadata.html_url)
}

/// Rejects feed-supplied file names that could escape the packages
/// directory. Requires exactly one plain file-name component: absolute
/// paths, parent/current-directory components (`..`, `.`), Windows drive
/// prefixes (`C:`), separators and multi-component paths are all rejected.
/// The separator/colon checks are platform-independent on purpose — a name
/// that is harmless on unix (where `\` and `:` are ordinary characters)
/// must not become dangerous if the packages dir is ever processed on
/// Windows, and vice versa.
fn validate_release_file_name(file_name: &str) -> Result<(), String> {
    // `/`/`\` escape the packages dir; `:` is a Windows drive prefix. The
    // URL-reserved `%`, `#`, `?` and whitespace would change the request
    // target when the name is interpolated into the release download URL
    // (`#` starts a fragment, `?` a query, `%` enables escape smuggling) —
    // velopack asset names never contain them.
    if file_name.contains(['/', '\\', ':', '%', '#', '?'])
        || file_name.chars().any(char::is_whitespace)
    {
        return Err(format!("Invalid update package file name: {file_name:?}"));
    }
    let mut components = Path::new(file_name).components();
    match components.next() {
        Some(std::path::Component::Normal(_)) if components.next().is_none() => Ok(()),
        _ => Err(format!("Invalid update package file name: {file_name:?}")),
    }
}

/// Rejects feed-supplied version strings before they are interpolated into
/// the release URL. Path separators or whitespace would escape the
/// intended download path or break the URL, the URL-reserved `%`/`#`/`?`
/// would change the request target (`#` starts a fragment, `?` a query,
/// `%` enables escape smuggling), and a bare `.`/`..` would resolve as a
/// path component in the no-`v` fallback URL. The checks are
/// platform-independent on purpose, mirroring `validate_release_file_name`.
fn validate_version(version: &str) -> Result<(), String> {
    if version.is_empty()
        || version.contains(['/', '\\', '%', '#', '?'])
        || version.chars().any(char::is_whitespace)
        || version == "."
        || version == ".."
    {
        return Err(format!("Invalid update version: {version:?}"));
    }
    Ok(())
}

async fn purge_packages_dir(packages_dir: &Path) {
    let dir = packages_dir.to_path_buf();
    let _ = tokio::task::spawn_blocking(move || {
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
    })
    .await;
}

/// Streaming chunk source abstraction so `collect_capped` can be unit
/// tested with a mock source: production uses `reqwest::Response` (whose
/// `chunk()` method drives the same incremental read), tests use a queue
/// of pre-made chunks.
trait ChunkSource {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, String>;
}

impl ChunkSource for reqwest::Response {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, String> {
        self.chunk()
            .await
            .map(|opt| opt.map(|bytes| bytes.to_vec()))
            .map_err(|e| e.to_string())
    }
}

/// Accumulates the chunk stream under a hard `cap` and — when `expected`
/// is set — under the feed-declared size, returning the buffered payload.
/// Aborts as soon as the running total exceeds either limit, rejects a
/// final size mismatch, and propagates stream errors as `Err`. `what`
/// names the payload in error messages.
async fn collect_capped<S: ChunkSource>(
    source: &mut S,
    expected: Option<u64>,
    cap: u64,
    what: &str,
) -> Result<Vec<u8>, String> {
    let mut total: u64 = 0;
    let mut bytes: Vec<u8> = Vec::new();
    loop {
        let chunk = source.next_chunk().await?;
        let Some(chunk) = chunk else { break };
        total += chunk.len() as u64;
        if total > cap || expected.is_some_and(|expected| total > expected) {
            return Err(match expected {
                Some(expected) => format!("{what} too large: expected {expected} bytes"),
                None => format!("{what} too large: exceeded hard cap of {cap} bytes"),
            });
        }
        bytes.extend_from_slice(&chunk);
    }
    if let Some(expected) = expected
        && total != expected
    {
        return Err(format!(
            "{what} size mismatch: expected {expected}, got {total}"
        ));
    }
    Ok(bytes)
}

/// Unique temp path for a file that will be renamed onto `final_path`.
/// Shared with the settings store via `media-sort-core`'s `path_utils`. The
/// temp deliberately does NOT carry the `.nupkg` extension, so
/// `pre_startup_verify_packages` never mistakes a leftover temp for an
/// unverified staged package.

/// Removes only the temp artifacts this run created, leaving previously
/// staged PGP-verified packages alone. A no-op when a download aborted
/// before the write. The final `package_path`/`sig_path` are never
/// touched: unverified content lives only in the uniquely-named temps,
/// so nothing a previous run staged and verified can be deleted here.
fn cleanup_run_artifacts(partial_path: &Path, sig_temp_path: &Path) {
    let _ = fs::remove_file(partial_path);
    let _ = fs::remove_file(sig_temp_path);
}

/// Whether a rename failure is worth the remove-destination-and-retry pass
/// of `rename_with_retry`. On Windows `std::fs::rename` maps to
/// MoveFileExW with MOVEFILE_REPLACE_EXISTING, so only a locked or
/// read-only destination fails — surfaced as `PermissionDenied`. On unix
/// `rename(2)` silently replaces an existing destination, so the retry
/// should never fire; `AlreadyExists` is included for parity with the
/// platform's error kinds but is unreachable in practice. Every other
/// error kind propagates unchanged: removing a verified package because
/// of an unrelated failure (e.g. a transient I/O error) must never happen.
fn is_stale_destination_error(err: &std::io::Error) -> bool {
    match err.kind() {
        std::io::ErrorKind::PermissionDenied => true,
        #[cfg(unix)]
        std::io::ErrorKind::AlreadyExists => true,
        _ => false,
    }
}

/// Renames `partial` onto `dest`, retrying once after removing `dest` when
/// the failure signals a stale/locked destination (see
/// `is_stale_destination_error`). On any other error kind the ORIGINAL
/// error is propagated immediately and nothing is removed; a failed retry
/// keeps the original error visible in the returned message.
fn rename_with_retry(partial: &Path, dest: &Path) -> Result<(), String> {
    let original = match fs::rename(partial, dest) {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };
    if !is_stale_destination_error(&original) {
        return Err(original.to_string());
    }
    match fs::remove_file(dest) {
        Ok(()) => fs::rename(partial, dest)
            .map_err(|retry_err| format!("{original}; retry failed: {retry_err}")),
        Err(remove_err) => Err(format!(
            "{original}; failed to remove stale destination: {remove_err}"
        )),
    }
}

/// Moves the PGP-verified staged files onto their final paths: the package
/// first, then its signature. Only called after the signature check passed,
/// so `package_path`/`sig_path` only ever receive verified content.
fn promote_staged_package(
    partial_path: &Path,
    sig_temp_path: &Path,
    package_path: &Path,
    sig_path: &Path,
) -> Result<(), String> {
    rename_with_retry(partial_path, package_path)
        .and_then(|()| rename_with_retry(sig_temp_path, sig_path))
}

pub async fn check_for_update_async(
    settings: &media_sort_core::settings::general::GeneralSettings,
) -> Result<Option<UpdateInfo>, String> {
    let repo_url = fetch_canonical_repo_url()
        .await
        .map_err(|e| e.to_string())?;
    let allow_prerelease =
        settings.install_prerelease_builds || env!("CARGO_PKG_VERSION").contains('-');

    tokio::task::spawn_blocking(move || {
        let source = GithubSource::new(&repo_url, None, allow_prerelease);
        let um = UpdateManager::new(source, None, None).map_err(|e| e.to_string())?;
        match um.check_for_updates().map_err(|e| e.to_string())? {
            UpdateCheck::UpdateAvailable(update) => Ok(Some(*update)),
            _ => Ok(None),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn download_and_apply_async(
    info: UpdateInfo,
    allow_prerelease: bool,
) -> Result<(), String> {
    let repo_url = fetch_canonical_repo_url()
        .await
        .map_err(|e| e.to_string())?;

    let locator =
        velopack::locator::auto_locate_app_manifest(velopack::locator::LocationContext::Unknown)
            .map_err(|e| format!("Failed to locate app manifest: {}", e))?;
    let packages_dir = locator.get_packages_dir();

    // Validate every feed-supplied file name before it can influence the
    // packages directory (full package, base release and delta files).
    let file_name = info.TargetFullRelease.FileName.clone();
    validate_release_file_name(&file_name)?;
    if let Some(ref base) = info.BaseRelease {
        validate_release_file_name(&base.FileName)?;
    }
    for delta in &info.DeltasToTarget {
        validate_release_file_name(&delta.FileName)?;
    }

    let package_path = packages_dir.join(&file_name);
    let sig_path = packages_dir.join(format!("{}.sig", file_name));
    let version = info.TargetFullRelease.Version.clone();

    // The version is feed-supplied like the file name: reject separators
    // and whitespace (and the `.`/`..` components they would otherwise
    // allow) before the string is interpolated into the release URL.
    validate_version(&version)?;

    // This run's unique temp paths, computed up front so failure cleanup
    // below can remove exactly the files this run created — and nothing a
    // previous run staged and PGP-verified.
    let partial_path = unique_temp_path(&package_path, "partial");
    let sig_temp_path = unique_temp_path(&sig_path, "sig");

    // 1. Download the package BEFORE velopack touches anything. velopack's
    // `download_updates()` extracts the embedded Squirrel.exe over the live
    // Update.exe (Windows) as part of the download step; running it on
    // unverified content would leave a malicious updater behind even when
    // the PGP check afterwards fails. Downloading + verifying here first
    // makes `download_updates()` below a no-op (it early-returns when the
    // package file already exists), so unverified content never reaches
    // velopack's extraction path.
    let client = reqwest::Client::builder()
        .user_agent("media-sort-gui-updater")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    let package_url = format!("{}/releases/download/v{}/{}", repo_url, version, file_name);
    let package_url_fallback = format!("{}/releases/download/{}/{}", repo_url, version, file_name);

    let mut response = match client.get(&package_url).send().await {
        Ok(res) if res.status().is_success() => res,
        _ => client
            .get(&package_url_fallback)
            .send()
            .await
            .map_err(|e| format!("Failed to download update package: {e}"))?,
    };
    if !response.status().is_success() {
        // Nothing has been written yet, so there are no run artifacts to
        // clean up.
        return Err(format!(
            "Failed to download update package: HTTP {}",
            response.status()
        ));
    }
    let expected_size = Some(info.TargetFullRelease.Size);
    // Read the package incrementally with a hard cap rather than trusting
    // response.bytes() to buffer an unbounded amount: the feed controls
    // content length, so a lying or compromised feed must not be able to
    // drive unbounded RSS. The chunks are streamed straight into this
    // run's unique staged temp file — never buffered whole in memory, so
    // a package near the 1 GiB cap does not cost 1 GiB of RAM on top of
    // the file — while the expected size is enforced incrementally
    // (abort as soon as the download exceeds it) and once more at the
    // end, and the file is fsynced before verification so it is durable
    // once renamed into place.
    let staged_write: Result<(), String> = async {
        let mut file = tokio::fs::File::create(&partial_path)
            .await
            .map_err(|e| e.to_string())?;
        let mut total: u64 = 0;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| format!("Failed to read update package: {e}"))?
        {
            total += chunk.len() as u64;
            if total > MAX_PACKAGE_BYTES || expected_size.is_some_and(|e| total > e) {
                return Err(match expected_size {
                    Some(expected) => {
                        format!("Update package too large: expected {expected} bytes")
                    }
                    None => format!(
                        "Update package too large: exceeded hard cap of {MAX_PACKAGE_BYTES} bytes"
                    ),
                });
            }
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write update package: {e}"))?;
        }
        if let Some(expected) = expected_size
            && total != expected
        {
            return Err(format!(
                "Update package size mismatch: expected {expected}, got {total}"
            ));
        }
        file.sync_all()
            .await
            .map_err(|e| format!("Failed to write update package: {e}"))
    }
    .await;
    if let Err(e) = staged_write {
        // Only this run's temp artifacts are removed: the final paths
        // (possibly a previously verified package) and other runs' temps
        // are left alone.
        cleanup_run_artifacts(&partial_path, &sig_temp_path);
        return Err(e);
    }

    // 2. Fetch and verify the detached PGP signature over the package.
    {
        let client = reqwest::Client::builder()
            .user_agent("media-sort-gui-updater")
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let sig_url = format!(
            "{}/releases/download/v{}/{}.sig",
            repo_url, version, file_name
        );
        let sig_url_fallback = format!(
            "{}/releases/download/{}/{}.sig",
            repo_url, version, file_name
        );

        let mut response = match client.get(&sig_url).send().await {
            Ok(res) if res.status().is_success() => res,
            _ => client.get(&sig_url_fallback).send().await.map_err(|e| {
                // This run's package is still staged at its temp path;
                // previously verified files are untouched.
                cleanup_run_artifacts(&partial_path, &sig_temp_path);
                format!("Failed to fetch signature: {e}")
            })?,
        };

        if !response.status().is_success() {
            cleanup_run_artifacts(&partial_path, &sig_temp_path);
            return Err(format!(
                "Failed to download signature file for verification: HTTP {}",
                response.status()
            ));
        }

        // The signature is tiny but feed-supplied: stream it under a hard
        // cap instead of trusting response.bytes() to buffer an unbounded
        // amount, and only write it to disk once it is fully received.
        let sig_bytes = collect_capped(&mut response, None, MAX_SIGNATURE_SIZE, "Signature file")
            .await
            .map_err(|e| {
                cleanup_run_artifacts(&partial_path, &sig_temp_path);
                format!("Failed to download signature file: {e}")
            })?;

        // Staged write to this run's unique temp: the signature only
        // reaches its final path together with the verified package (see
        // `promote_staged_package`), so a crash mid-run never leaves a
        // `.nupkg`/`.sig` pair that `pre_startup_verify_packages` would
        // verify or purge against.
        let sig_write_res: Result<(), String> = tokio::task::spawn_blocking({
            let sig_temp_path = sig_temp_path.clone();
            move || -> Result<(), String> {
                let mut file = fs::File::create(&sig_temp_path).map_err(|e| e.to_string())?;
                file.write_all(&sig_bytes).map_err(|e| e.to_string())?;
                file.sync_all().map_err(|e| e.to_string())?;
                Ok(())
            }
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?;
        if let Err(e) = sig_write_res {
            cleanup_run_artifacts(&partial_path, &sig_temp_path);
            return Err(format!("Failed to write signature file: {e}"));
        }

        let verify_res = tokio::task::spawn_blocking({
            let partial_path = partial_path.clone();
            let sig_temp_path = sig_temp_path.clone();
            move || verify_package_signature(&partial_path, &sig_temp_path)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?;

        if let Err(e) = verify_res {
            // The staged package failed PGP verification: it may be
            // malicious. Purge the whole packages dir rather than leaving
            // poisoned content for `pre_startup_verify_packages` or a
            // later run to trip over.
            purge_packages_dir(&packages_dir).await;
            return Err(format!("GPG signature verification failed: {e}"));
        }
    }

    // 3. Promote the verified content into place: the package and its
    // signature are renamed onto their final paths only now that the
    // signature check passed. Until this point `package_path`/`sig_path`
    // still hold whatever a previous run verified (or nothing), so a
    // failure below never leaves unverified content where velopack or
    // `pre_startup_verify_packages` would apply it. The stale/locked
    // destination retry of `rename_with_retry` covers a package replaced
    // mid-flight on Windows.
    let promote_res = tokio::task::spawn_blocking({
        let partial_path = partial_path.clone();
        let sig_temp_path = sig_temp_path.clone();
        let package_path = package_path.clone();
        let sig_path = sig_path.clone();
        move || promote_staged_package(&partial_path, &sig_temp_path, &package_path, &sig_path)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;
    if let Err(e) = promote_res {
        // Only this run's temps can remain (a partial promote); the final
        // paths are either untouched or hold verified content.
        cleanup_run_artifacts(&partial_path, &sig_temp_path);
        return Err(format!("Failed to stage verified update package: {e}"));
    }

    // 4. velopack apply: the package already exists in the packages dir, so
    // `download_updates()` early-returns and never extracts anything from
    // content that was not PGP-verified above.
    tokio::task::spawn_blocking({
        let info_clone = info.clone();
        let repo_url_clone = repo_url.clone();
        move || {
            let source = GithubSource::new(&repo_url_clone, None, allow_prerelease);
            let um = UpdateManager::new(source, None, None).map_err(|e| e.to_string())?;
            um.download_updates(&info_clone, None)
                .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    // 5. Re-verify immediately before the apply step to narrow the local
    // TOCTOU window between the earlier verification and execution. A
    // failure here purges the whole packages dir, like the earlier
    // verification failure, so poisoned content is never left staged.
    let reverify_res = tokio::task::spawn_blocking({
        let package_path = package_path.clone();
        let sig_path = sig_path.clone();
        move || verify_package_signature(&package_path, &sig_path)
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Err(e) = reverify_res {
        purge_packages_dir(&packages_dir).await;
        return Err(format!("Re-verification before apply failed: {e}"));
    }

    // 6. Apply. Everything below this point operates on PGP-verified
    // content only.
    tokio::task::spawn_blocking(move || {
        let source = GithubSource::new(&repo_url, None, allow_prerelease);
        let um = UpdateManager::new(source, None, None).map_err(|e| e.to_string())?;
        um.apply_updates_and_restart(&info)
            .map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PUBKEY: &[u8] = b"-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nmDMEalvSuBYJKwYBBAHaRw8BAQdA6gzplEurqkbHUPq+ZbI0EpiWTL+qIEVZGaeu\nO1yw/Vi0F1Rlc3QgPHRlc3RAZXhhbXBsZS5jb20+iJAEExYKADgWIQQXiJEkcn0T\nuXxHoSsXvW6a5c13bQUCalvSuAIbAwULCQgHAgYVCgkICwIEFgIDAQIeAQIXgAAK\nCRAXvW6a5c13bfNwAQCo7crYPuLSCWedK4Jv3Ex9kr5x1rG5FaeAmRr2ugr/eQD+\nO6375lNxCnBthV30GKBNf92X1/qRWOIukvq8csBkVgS4OARqW9K4EgorBgEEAZdV\nAQUBAQdAQIq23WWVowtxaxyRNFAyq3jsQI8ZS15oG89Q9QA4nAwDAQgHiHgEGBYK\nACAWIQQXiJEkcn0TuXxHoSsXvW6a5c13bQUCalvSuAIbDAAKCRAXvW6a5c13bV4e\nAP9BlI/vwGvWJajIU8MXI444b70wuYEZ1SGnaK83NLwiOgEA0d6fEi/qkm9XMTdn\nikCNDWMSOJLbaMTpzz0Kzp/TTwc=\n=7Qpu\n-----END PGP PUBLIC KEY BLOCK-----";

    const TEST_SIG: &[u8] = &[
        0x88, 0x75, 0x04, 0x00, 0x16, 0x0a, 0x00, 0x1d, 0x16, 0x21, 0x04, 0x17, 0x88, 0x91, 0x24,
        0x72, 0x7d, 0x13, 0xb9, 0x7c, 0x47, 0xa1, 0x2b, 0x17, 0xbd, 0x6e, 0x9a, 0xe5, 0xcd, 0x77,
        0x6d, 0x05, 0x02, 0x6a, 0x5b, 0xd2, 0xbb, 0x00, 0x0a, 0x09, 0x10, 0x17, 0xbd, 0x6e, 0x9a,
        0xe5, 0xcd, 0x77, 0x6d, 0x5b, 0x6f, 0x00, 0xfe, 0x2b, 0xe8, 0xff, 0x23, 0x00, 0xd4, 0x38,
        0x9d, 0x7a, 0x84, 0x1b, 0xab, 0x0b, 0xb4, 0xc0, 0x59, 0x38, 0xdb, 0xec, 0x1b, 0x8c, 0x24,
        0x5d, 0x34, 0xec, 0x57, 0x28, 0x32, 0x29, 0x96, 0x84, 0x63, 0x01, 0x00, 0xd2, 0x73, 0xc6,
        0xd5, 0x2b, 0x22, 0xaf, 0x67, 0x81, 0x7b, 0x68, 0x2b, 0x0c, 0x5b, 0xe6, 0x5f, 0xd2, 0x53,
        0x85, 0xf2, 0x47, 0x36, 0x93, 0x57, 0x99, 0x64, 0xd6, 0x6d, 0x4f, 0xcf, 0xad, 0x00,
    ];

    const TEST_DATA: &[u8] = b"hello world";

    static TEST_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn get_temp_paths(prefix: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        use std::sync::atomic::Ordering;
        let pid = std::process::id();
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join(format!("media_sort_test_{}_{}_{}.nupkg", prefix, pid, id));
        let sig_path = temp_dir.join(format!(
            "media_sort_test_{}_{}_{}.nupkg.sig",
            prefix, pid, id
        ));
        (data_path, sig_path)
    }

    #[test]
    fn test_signature_verification_success() {
        let (data_path, sig_path) = get_temp_paths("success");

        fs::write(&data_path, TEST_DATA).unwrap();
        fs::write(&sig_path, TEST_SIG).unwrap();

        let result = verify_signature_bytes(TEST_PUBKEY, &data_path, &sig_path);

        let _ = fs::remove_file(&data_path);
        let _ = fs::remove_file(&sig_path);

        assert!(result.is_ok(), "Verification failed: {:?}", result);
    }

    #[test]
    fn test_signature_verification_tampered_payload() {
        let (data_path, sig_path) = get_temp_paths("tampered");

        fs::write(&data_path, b"tampered content").unwrap();
        fs::write(&sig_path, TEST_SIG).unwrap();

        let result = verify_signature_bytes(TEST_PUBKEY, &data_path, &sig_path);

        let _ = fs::remove_file(&data_path);
        let _ = fs::remove_file(&sig_path);

        assert!(
            result.is_err(),
            "Verification should have failed for tampered content"
        );
        assert!(result.unwrap_err().contains("verification failed"));
    }

    #[test]
    fn test_signature_verification_invalid_sig() {
        let (data_path, sig_path) = get_temp_paths("invalid");

        fs::write(&data_path, TEST_DATA).unwrap();
        fs::write(&sig_path, b"not a valid pgp signature").unwrap();

        let result = verify_signature_bytes(TEST_PUBKEY, &data_path, &sig_path);

        let _ = fs::remove_file(&data_path);
        let _ = fs::remove_file(&sig_path);

        assert!(
            result.is_err(),
            "Verification should have failed for invalid signature"
        );
    }

    #[test]
    fn test_validate_release_file_name_accepts_plain_name() {
        assert!(validate_release_file_name("MediaSort-1.2.3-win-x64-full.nupkg").is_ok());
        assert!(validate_release_file_name("package.nupkg").is_ok());
    }

    #[test]
    fn test_validate_release_file_name_rejects_traversal_and_prefixes() {
        // ParentDir/CurDir/Prefix are single components: the old
        // components().count() == 1 check let these through, and
        // packages_dir.join("..") escapes the packages directory.
        for bad in [
            "..",
            ".",
            "../evil.nupkg",
            "C:",
            "C:evil.nupkg",
            "/etc/passwd",
            r"\server\share\evil.nupkg",
            "a/b.nupkg",
            "a\\b.nupkg",
            "",
        ] {
            assert!(
                validate_release_file_name(bad).is_err(),
                "{bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_validate_release_file_name_rejects_url_reserved() {
        // Interpolated into the release download URL: `#` starts a
        // fragment, `?` a query, `%` enables escape smuggling (e.g. %2F),
        // whitespace changes the request target.
        for bad in [
            "a%2F..",
            "a#frag.nupkg",
            "a?x=1.nupkg",
            "a b.nupkg",
            "a\tb.nupkg",
        ] {
            assert!(
                validate_release_file_name(bad).is_err(),
                "{bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_validate_version_accepts_plain_version() {
        assert!(validate_version("1.2.3").is_ok());
        assert!(validate_version("1.2.3-beta.1").is_ok());
        assert!(validate_version("2.0.0.0").is_ok());
    }

    #[test]
    fn test_validate_version_rejects_separators_and_whitespace() {
        // `.`/`..` would resolve as path components in the no-`v` fallback
        // URL and are rejected alongside separators and whitespace.
        for bad in ["", ".", "..", "1/2", "1\\2", "1.2 3", "1.2\n3", "1.2\t3"] {
            assert!(validate_version(bad).is_err(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn test_validate_version_rejects_url_reserved() {
        // Interpolated into the release URL: `#` starts a fragment, `?` a
        // query, `%` enables escape smuggling (e.g. %2F as a separator).
        for bad in ["1.2%2F3", "1.2#x", "1.2?x=1"] {
            assert!(validate_version(bad).is_err(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn test_unique_temp_path_shape_and_uniqueness() {
        let pkg = Path::new("pkg-dir/MediaSort-1.2.3-full.nupkg");
        let a = unique_temp_path(pkg, "partial");
        let b = unique_temp_path(pkg, "partial");
        let sig = unique_temp_path(Path::new("pkg-dir/MediaSort-1.2.3-full.nupkg.sig"), "sig");

        assert_ne!(a, b, "each run must get a distinct temp name");
        let pid = std::process::id().to_string();
        let name = |p: &Path| p.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name(&a).starts_with(&format!("MediaSort-1.2.3-full.partial.{pid}.")));
        assert!(name(&b).starts_with(&format!("MediaSort-1.2.3-full.partial.{pid}.")));
        assert!(name(&sig).starts_with(&format!("MediaSort-1.2.3-full.nupkg.sig.{pid}.")));
        // Temps must not look like staged packages to the startup scanner.
        assert_ne!(a.extension(), Some(std::ffi::OsStr::new("nupkg")));
    }

    fn test_dir(prefix: &str) -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "media_sort_retry_{}_{}_{}",
            prefix,
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_rename_with_retry_success() {
        let dir = test_dir("success");
        let partial = dir.join("pkg.partial.0.0");
        let dest = dir.join("pkg.nupkg");
        fs::write(&partial, b"data").unwrap();

        rename_with_retry(&partial, &dest).unwrap();

        assert!(!partial.exists());
        assert_eq!(fs::read(&dest).unwrap(), b"data");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rename_with_retry_replaces_existing_dest() {
        // unix: rename(2) replaces the destination outright; Windows:
        // MoveFileExW passes MOVEFILE_REPLACE_EXISTING. Either way the new
        // content wins and the retry is not needed.
        let dir = test_dir("replace");
        let partial = dir.join("pkg.partial.0.0");
        let dest = dir.join("pkg.nupkg");
        fs::write(&partial, b"new").unwrap();
        fs::write(&dest, b"old").unwrap();

        rename_with_retry(&partial, &dest).unwrap();

        assert_eq!(fs::read(&dest).unwrap(), b"new");
        assert!(!partial.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_promote_staged_package_moves_both_files() {
        let dir = test_dir("promote_ok");
        let partial = dir.join("pkg.partial.0.0");
        let sig_temp = dir.join("pkg.sig.0.0");
        let package_path = dir.join("pkg.nupkg");
        let sig_path = dir.join("pkg.nupkg.sig");
        fs::write(&partial, b"pkg").unwrap();
        fs::write(&sig_temp, b"sig").unwrap();

        promote_staged_package(&partial, &sig_temp, &package_path, &sig_path).unwrap();

        assert!(!partial.exists());
        assert!(!sig_temp.exists());
        assert_eq!(fs::read(&package_path).unwrap(), b"pkg");
        assert_eq!(fs::read(&sig_path).unwrap(), b"sig");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_promote_staged_package_failure_leaves_finals_untouched() {
        // A missing staged package fails the promote: nothing must be
        // renamed and the final paths (possibly a previously verified
        // package) must stay intact.
        let dir = test_dir("promote_fail");
        let partial = dir.join("pkg.partial.0.0");
        let sig_temp = dir.join("pkg.sig.0.0");
        let package_path = dir.join("pkg.nupkg");
        let sig_path = dir.join("pkg.nupkg.sig");
        fs::write(&sig_temp, b"sig").unwrap();
        fs::write(&package_path, b"old verified").unwrap();

        let err =
            promote_staged_package(&partial, &sig_temp, &package_path, &sig_path).unwrap_err();

        assert!(!err.is_empty());
        assert!(sig_temp.exists(), "the signature must not be moved either");
        assert_eq!(fs::read(&package_path).unwrap(), b"old verified");
        assert!(!sig_path.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rename_with_retry_non_retryable_error_propagates() {
        // Renaming a file onto an existing DIRECTORY fails with a
        // non-retryable error kind (EISDIR on unix): the ORIGINAL error
        // must be propagated, the destination must not be removed and the
        // partial must be left alone.
        let dir = test_dir("direrr");
        let partial = dir.join("pkg.partial.0.0");
        let dest = dir.join("dest_dir");
        fs::write(&partial, b"data").unwrap();
        fs::create_dir(&dest).unwrap();

        let result = rename_with_retry(&partial, &dest);

        let err = result.unwrap_err();
        assert!(!err.is_empty());
        assert!(
            dest.is_dir(),
            "non-retryable failure must not remove the destination"
        );
        assert!(
            partial.exists(),
            "non-retryable failure must not remove the partial"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_stale_destination_error_classification() {
        use std::io::ErrorKind;
        assert!(is_stale_destination_error(&std::io::Error::from(
            ErrorKind::PermissionDenied
        )));
        #[cfg(unix)]
        assert!(is_stale_destination_error(&std::io::Error::from(
            ErrorKind::AlreadyExists
        )));
        #[cfg(not(unix))]
        assert!(!is_stale_destination_error(&std::io::Error::from(
            ErrorKind::AlreadyExists
        )));
        assert!(!is_stale_destination_error(&std::io::Error::from(
            ErrorKind::NotFound
        )));
        assert!(!is_stale_destination_error(&std::io::Error::from(
            ErrorKind::Other
        )));
    }

    struct MockChunkSource {
        chunks: std::collections::VecDeque<Result<Option<Vec<u8>>, String>>,
    }

    impl MockChunkSource {
        fn new(chunks: Vec<Result<Option<Vec<u8>>, String>>) -> Self {
            Self {
                chunks: chunks.into(),
            }
        }
    }

    impl ChunkSource for MockChunkSource {
        async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, String> {
            self.chunks.pop_front().unwrap_or(Ok(None))
        }
    }

    #[tokio::test]
    async fn test_collect_capped_ok() {
        let mut src = MockChunkSource::new(vec![
            Ok(Some(b"hel".to_vec())),
            Ok(Some(b"lo".to_vec())),
            Ok(None),
        ]);

        let bytes = collect_capped(&mut src, Some(5), 1024, "Update package")
            .await
            .unwrap();

        assert_eq!(bytes, b"hello");
    }

    #[tokio::test]
    async fn test_collect_capped_no_expected_size() {
        let mut src = MockChunkSource::new(vec![Ok(Some(vec![0u8; 5])), Ok(None)]);

        let bytes = collect_capped(&mut src, None, 1024, "Signature file")
            .await
            .unwrap();

        assert_eq!(bytes.len(), 5);
    }

    #[tokio::test]
    async fn test_collect_capped_size_mismatch() {
        let mut src = MockChunkSource::new(vec![Ok(Some(b"hello".to_vec())), Ok(None)]);

        let err = collect_capped(&mut src, Some(6), 1024, "Update package")
            .await
            .unwrap_err();

        assert!(err.contains("size mismatch"), "{err}");
        assert!(err.contains("expected 6"), "{err}");
    }

    #[tokio::test]
    async fn test_collect_capped_exceeds_expected_size() {
        let mut src = MockChunkSource::new(vec![Ok(Some(b"hello".to_vec())), Ok(None)]);

        let err = collect_capped(&mut src, Some(4), 1024, "Update package")
            .await
            .unwrap_err();

        assert!(err.contains("too large"), "{err}");
        assert!(err.contains("expected 4"), "{err}");
    }

    #[tokio::test]
    async fn test_collect_capped_exceeds_hard_cap() {
        let mut src = MockChunkSource::new(vec![Ok(Some(vec![0u8; 10])), Ok(None)]);

        let err = collect_capped(&mut src, None, 8, "Signature file")
            .await
            .unwrap_err();

        assert!(err.contains("hard cap"), "{err}");
    }

    #[tokio::test]
    async fn test_collect_capped_stream_error_propagates() {
        let mut src = MockChunkSource::new(vec![Err("boom".to_string())]);

        let err = collect_capped(&mut src, None, 8, "Signature file")
            .await
            .unwrap_err();

        assert!(err.contains("boom"), "{err}");
    }
}
