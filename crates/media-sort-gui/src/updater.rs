use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use velopack::sources::GithubSource;
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};

const PUBKEY_ASC_BYTES: &[u8] = include_bytes!("../../../packaging/linux/pubkey.asc");

fn verify_package_signature(package_path: &Path, sig_path: &Path) -> Result<(), String> {
    // 1. Load public key from the embedded bytes
    let pubkey_str = std::str::from_utf8(PUBKEY_ASC_BYTES)
        .map_err(|e| format!("Failed to parse public key bytes as UTF-8: {}", e))?;
    let (public_key, _) = SignedPublicKey::from_string(pubkey_str)
        .map_err(|e| format!("Failed to load PGP public key: {:?}", e))?;

    // 2. Load the detached signature
    let sig_bytes =
        fs::read(sig_path).map_err(|e| format!("Failed to read signature file: {}", e))?;
    let sig = DetachedSignature::from_bytes(Cursor::new(sig_bytes))
        .map_err(|e| format!("Failed to parse PGP signature: {:?}", e))?;

    // 3. Load the package file data
    let package_bytes =
        fs::read(package_path).map_err(|e| format!("Failed to read package file: {}", e))?;

    // 4. Verify signature against key and data
    sig.verify(&public_key, &package_bytes)
        .map_err(|e| format!("PGP signature verification failed: {:?}", e))?;

    Ok(())
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
    if let Ok(entries) = fs::read_dir(&packages_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "nupkg") {
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

    if invalid {
        tracing::warn!("Purging packages directory due to signature verification failure.");
        let _ = fs::remove_dir_all(&packages_dir);
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
    let file_name = info.TargetFullRelease.FileName.clone();
    let package_path = packages_dir.join(&file_name);
    let sig_path = packages_dir.join(format!("{}.sig", file_name));
    let version = info.TargetFullRelease.Version.clone();

    let info_clone = info.clone();
    let repo_url_clone = repo_url.clone();
    tokio::task::spawn_blocking(move || {
        let source = GithubSource::new(&repo_url_clone, None, allow_prerelease);
        let um = UpdateManager::new(source, None, None).map_err(|e| e.to_string())?;
        um.download_updates(&info_clone, None)
            .map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())??;

    {
        let client = reqwest::Client::builder()
            .user_agent("media-sort-gui-updater")
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

        let response = client.get(&sig_url).send().await;
        let response = match response {
            Ok(res) if res.status().is_success() => res,
            _ => client
                .get(&sig_url_fallback)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch signature: {}", e))?,
        };

        if !response.status().is_success() {
            let _ = fs::remove_dir_all(&packages_dir);
            return Err(format!(
                "Failed to download signature file for verification: HTTP {}",
                response.status()
            ));
        }

        let sig_bytes = response.bytes().await.map_err(|e| e.to_string())?;
        fs::write(&sig_path, sig_bytes)
            .map_err(|e| format!("Failed to write signature file: {}", e))?;

        if let Err(e) = verify_package_signature(&package_path, &sig_path) {
            let _ = fs::remove_dir_all(&packages_dir);
            return Err(format!("GPG signature verification failed: {}", e));
        }
    }

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
