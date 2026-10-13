use winreg::RegKey;
use winreg::enums::*;

const DISPLAY_NAME: &str = "Open with Media Sort";

/// Registry subkeys written by this (Rust) version. The leaf name matches the
/// Velopack package ID (`vpk pack -u MediaSort`) so the context-menu entry and
/// the installer agree on one identity.
const REG_PATHS: &[&str] = &[
    "Software\\Classes\\Directory\\shell\\MediaSort",
    "Software\\Classes\\Drive\\shell\\MediaSort",
];

/// Registry subkeys written by the legacy WPF (v2.x) build. They were never
/// renamed, so users who ran the old version keep a context-menu entry whose
/// command points at an uninstalled path. They are removed best-effort on
/// register/unregister.
const LEGACY_REG_PATHS: &[&str] = &[
    "Software\\Classes\\Directory\\shell\\ImageSort",
    "Software\\Classes\\Drive\\shell\\ImageSort",
];

fn remove_legacy_entries(hkcu: &RegKey) {
    for reg_path in LEGACY_REG_PATHS {
        let _ = hkcu.delete_subkey_all(format!("{}\\command", reg_path));
        let _ = hkcu.delete_subkey_all(*reg_path);
    }
}

pub fn register(app_path: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let icon = app_path.to_string();
    let command = format!("\"{}\" \"%1\"", app_path);

    // Drop the WPF-era entries before writing ours, otherwise a user who ran
    // v2.x ends up with two identical-looking "Open with Media Sort" entries.
    remove_legacy_entries(&hkcu);

    for reg_path in REG_PATHS {
        let (key, _) = hkcu
            .create_subkey(reg_path)
            .map_err(|e| format!("Failed to create registry key {}: {}", reg_path, e))?;
        key.set_value("", &DISPLAY_NAME)
            .map_err(|e| format!("Failed to set default value for {}: {}", reg_path, e))?;
        key.set_value("Icon", &icon)
            .map_err(|e| format!("Failed to set Icon value for {}: {}", reg_path, e))?;

        let cmd_path = format!("{}\\command", reg_path);
        let (cmd_key, _) = hkcu
            .create_subkey(&cmd_path)
            .map_err(|e| format!("Failed to create command key {}: {}", cmd_path, e))?;
        cmd_key
            .set_value("", &command)
            .map_err(|e| format!("Failed to set command for {}: {}", cmd_path, e))?;
    }

    tracing::info!("Registered Media Sort context menu entries in Windows registry");
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for reg_path in REG_PATHS {
        let cmd_path = format!("{}\\command", reg_path);

        let _ = hkcu.delete_subkey_all(&cmd_path);
        let _ = hkcu.delete_subkey_all(reg_path);
    }

    // Also clear the WPF-era keys in case the setting is toggled off by
    // someone who only ever ran v2.x (register() never ran for them).
    remove_legacy_entries(&hkcu);

    tracing::info!("Removed Media Sort context menu entries from Windows registry");
    Ok(())
}

pub fn is_registered() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    Ok(REG_PATHS
        .iter()
        .any(|reg_path| hkcu.open_subkey(format!("{reg_path}\\command")).is_ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // There is only one set of real shell registry paths, so these scenarios
    // cannot be isolated from each other: both mutate the same per-user keys.
    // cargo-nextest runs every test in its own process, so splitting the
    // lifecycle across tests makes them race -- one test's unregister() deletes
    // the keys the other is asserting on. Keep the whole lifecycle in a single
    // test so it works under any runner, including process-per-test nextest.
    #[test]
    fn test_register_unregister_cycle_and_legacy_wpf_cleanup() {
        let app_path = "C:\\Program Files\\MediaSort\\media-sort-gui.exe";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        let _ = unregister();
        assert!(!is_registered().unwrap());

        // Simulate a machine that still has the WPF v2.x entries.
        for reg_path in LEGACY_REG_PATHS {
            hkcu.create_subkey(reg_path).unwrap();
            hkcu.create_subkey(format!("{}\\command", reg_path))
                .unwrap();
        }

        register(app_path).unwrap();
        assert!(is_registered().unwrap());

        // The legacy leaf keys are gone, ours exist, and the shell sees exactly
        // one entry per location.
        for reg_path in LEGACY_REG_PATHS {
            assert!(hkcu.open_subkey(*reg_path).is_err());
        }
        for reg_path in REG_PATHS {
            assert!(hkcu.open_subkey(format!("{}\\command", reg_path)).is_ok());
        }

        unregister().unwrap();
        assert!(!is_registered().unwrap());
    }
}
