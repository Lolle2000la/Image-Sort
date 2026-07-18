use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use velopack::sources::GithubSource;
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};

const PUBKEY_ASC_BYTES: &[u8] = include_bytes!("../../../packaging/linux/pubkey.asc");

fn verify_signature(
    public_key: &SignedPublicKey,
    package_path: &Path,
    sig_path: &Path,
) -> Result<(), String> {
    // 1. Load the detached signature
    let sig_bytes =
        fs::read(sig_path).map_err(|e| format!("Failed to read signature file: {}", e))?;
    let sig = DetachedSignature::from_bytes(Cursor::new(sig_bytes))
        .map_err(|e| format!("Failed to parse PGP signature: {:?}", e))?;

    // 2. Load the package file data
    let package_bytes =
        fs::read(package_path).map_err(|e| format!("Failed to read package file: {}", e))?;

    // 3. Verify signature against key and data
    sig.verify(public_key, &package_bytes)
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
    static PUBLIC_KEY: OnceLock<SignedPublicKey> = OnceLock::new();

    if PUBLIC_KEY.get().is_none() {
        let pubkey_str = std::str::from_utf8(PUBKEY_ASC_BYTES)
            .map_err(|e| format!("Failed to parse public key bytes as UTF-8: {}", e))?;
        let (public_key, _) = SignedPublicKey::from_string(pubkey_str)
            .map_err(|e| format!("Failed to load PGP public key: {:?}", e))?;
        let _ = PUBLIC_KEY.set(public_key);
    }
    let public_key = PUBLIC_KEY.get().ok_or("Public key not initialized")?;

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

    #[test]
    fn test_signature_verification_success() {
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join("media_sort_test_data_success.nupkg");
        let sig_path = temp_dir.join("media_sort_test_data_success.nupkg.sig");

        fs::write(&data_path, TEST_DATA).unwrap();
        fs::write(&sig_path, TEST_SIG).unwrap();

        let result = verify_signature_bytes(TEST_PUBKEY, &data_path, &sig_path);

        let _ = fs::remove_file(&data_path);
        let _ = fs::remove_file(&sig_path);

        assert!(result.is_ok(), "Verification failed: {:?}", result);
    }

    #[test]
    fn test_signature_verification_tampered_payload() {
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join("media_sort_test_data_tampered.nupkg");
        let sig_path = temp_dir.join("media_sort_test_data_tampered.nupkg.sig");

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
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join("media_sort_test_data_invalid.nupkg");
        let sig_path = temp_dir.join("media_sort_test_data_invalid.nupkg.sig");

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
}
