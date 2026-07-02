use winreg::RegKey;
use winreg::enums::*;

const DISPLAY_NAME: &str = "Open with Media Sort";

const REG_PATHS: &[&str] = &[
    "Software\\Classes\\Directory\\shell\\ImageSort",
    "Software\\Classes\\Drive\\shell\\ImageSort",
];

pub fn register(app_path: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let icon = app_path.to_string();
    let command = format!("\"{}\" \"%1\"", app_path);

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

    tracing::info!("Removed Media Sort context menu entries from Windows registry");
    Ok(())
}

pub fn is_registered() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for reg_path in REG_PATHS {
        let cmd_path = format!("{}\\command", reg_path);
        if hkcu.open_subkey(&cmd_path).is_ok() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_unregister_cycle() {
        let app_path = "C:\\Program Files\\MediaSort\\media-sort-gui.exe";

        let _ = unregister();

        assert!(!is_registered().unwrap());

        register(app_path).unwrap();
        assert!(is_registered().unwrap());

        unregister().unwrap();
        assert!(!is_registered().unwrap());
    }
}
