use std::fs;
use std::path::Path;
use velopack::sources::GithubSource;
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};

const PUBKEY_ASC_BYTES: &[u8] = include_bytes!("../../../packaging/linux/pubkey.asc");

#[cfg(target_os = "linux")]
fn dearmor_gpg_key(asc_bytes: &[u8], dest_gpg_path: &Path) -> Result<(), String> {
    use std::io::Write;
    let mut child = std::process::Command::new("gpg")
        .arg("--dearmor")
        .arg("--output")
        .arg(dest_gpg_path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to spawn gpg --dearmor: {}. Please ensure GnuPG/GPG is installed.",
                e
            )
        })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or("Failed to open stdin of gpg command")?;
        stdin.write_all(asc_bytes).map_err(|e| e.to_string())?;
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("gpg --dearmor failed".to_string())
    }
}

#[cfg(target_os = "linux")]
fn verify_gpg_signature(
    package_path: &Path,
    sig_path: &Path,
    pubkey_path: &Path,
) -> Result<(), String> {
    let output = std::process::Command::new("gpg")
        .arg("--no-default-keyring")
        .arg("--keyring")
        .arg(pubkey_path)
        .arg("--verify")
        .arg(sig_path)
        .arg(package_path)
        .output()
        .map_err(|e| format!("Failed to execute gpg verify: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("GPG verification failed:\n{}", stderr))
    }
}

#[cfg(target_os = "linux")]
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
                let sig_path = path.with_extension("sig");
                if !sig_path.exists() {
                    tracing::warn!(
                        "Found unverified update package without signature: {:?}",
                        path
                    );
                    invalid = true;
                    break;
                }

                let temp_dir = std::env::temp_dir();
                let pubkey_path = temp_dir.join("media_sort_pubkey.gpg");
                if let Err(e) = dearmor_gpg_key(PUBKEY_ASC_BYTES, &pubkey_path) {
                    tracing::error!("Failed to dearmor embedded public key: {}", e);
                    invalid = true;
                    break;
                }

                if let Err(e) = verify_gpg_signature(&path, &sig_path, &pubkey_path) {
                    tracing::error!("GPG verification failed for pending update: {}", e);
                    invalid = true;
                    let _ = fs::remove_file(&pubkey_path);
                    break;
                }
                let _ = fs::remove_file(&pubkey_path);
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

    #[cfg(target_os = "linux")]
    let (packages_dir, file_name, package_path, sig_path, version) = {
        let locator = velopack::locator::auto_locate_app_manifest(
            velopack::locator::LocationContext::Unknown,
        )
        .map_err(|e| format!("Failed to locate app manifest: {}", e))?;
        let packages_dir = locator.get_packages_dir();
        let file_name = info.TargetFullRelease.FileName.clone();
        let package_path = packages_dir.join(&file_name);
        let sig_path = packages_dir.join(format!("{}.sig", file_name));
        let version = info.TargetFullRelease.Version.clone();
        (packages_dir, file_name, package_path, sig_path, version)
    };

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

    #[cfg(target_os = "linux")]
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

        let temp_dir = std::env::temp_dir();
        let pubkey_path = temp_dir.join("media_sort_pubkey.gpg");
        if let Err(e) = dearmor_gpg_key(PUBKEY_ASC_BYTES, &pubkey_path) {
            let _ = fs::remove_dir_all(&packages_dir);
            let _ = fs::remove_file(&pubkey_path);
            return Err(format!("Failed to dearmor GPG key: {}", e));
        }

        if let Err(e) = verify_gpg_signature(&package_path, &sig_path, &pubkey_path) {
            let _ = fs::remove_dir_all(&packages_dir);
            let _ = fs::remove_file(&pubkey_path);
            return Err(format!("GPG signature verification failed: {}", e));
        }

        let _ = fs::remove_file(&pubkey_path);
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
