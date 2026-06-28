use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::settings::general::GeneralSettings;
use crate::settings::keybindings::KeyBindings;
use crate::settings::metadata_panel::MetadataPanelSettings;
use crate::settings::pinned_folders::PinnedFoldersSettings;
use crate::settings::window_position::WindowPosition;

#[derive(Debug)]
pub enum SettingsError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::Io(e) => write!(f, "IO error: {e}"),
            SettingsError::Serde(e) => write!(f, "Serialization error: {e}"),
            SettingsError::TomlDe(e) => write!(f, "TOML deserialization error: {e}"),
            SettingsError::TomlSer(e) => write!(f, "TOML serialization error: {e}"),
        }
    }
}

impl From<std::io::Error> for SettingsError {
    fn from(e: std::io::Error) -> Self {
        SettingsError::Io(e)
    }
}

impl From<serde_json::Error> for SettingsError {
    fn from(e: serde_json::Error) -> Self {
        SettingsError::Serde(e)
    }
}

impl From<toml::de::Error> for SettingsError {
    fn from(e: toml::de::Error) -> Self {
        SettingsError::TomlDe(e)
    }
}

impl From<toml::ser::Error> for SettingsError {
    fn from(e: toml::ser::Error) -> Self {
        SettingsError::TomlSer(e)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SettingsStore {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub pinned_folders: PinnedFoldersSettings,
    #[serde(default)]
    pub window_position: WindowPosition,
    #[serde(default)]
    pub metadata_panel: MetadataPanelSettings,
}

impl SettingsStore {
    pub fn config_path() -> PathBuf {
        if let Ok(val) = std::env::var("UI_TEST") {
            if !val.is_empty() {
                return PathBuf::from("ui_test_config.toml");
            }
        }
        let base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort");
        std::fs::create_dir_all(&base).ok();
        if cfg!(debug_assertions) {
            base.join("debug_config.toml")
        } else {
            base.join("config.toml")
        }
    }

    pub fn load() -> Result<Self, SettingsError> {
        let toml_path = Self::config_path();
        if toml_path.exists() {
            let data = std::fs::read_to_string(&toml_path)?;
            let store: SettingsStore = toml::from_str(&data)?;
            return Ok(store);
        }

        // Search for existing JSON config files to migrate
        let mut migrated_settings: Option<SettingsStore> = None;

        if let Ok(val) = std::env::var("UI_TEST") {
            if !val.is_empty() {
                let old_json_path = PathBuf::from("ui_test_config.json");
                if old_json_path.exists() {
                    if let Ok(data) = std::fs::read_to_string(&old_json_path) {
                        if let Ok(store) = serde_json::from_str::<SettingsStore>(&data) {
                            migrated_settings = Some(store);
                        }
                    }
                }
            }
        }

        if migrated_settings.is_none() {
            // Check new media-sort JSON path next
            let base = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("media-sort");
            let rust_json_path = if cfg!(debug_assertions) {
                base.join("debug_config.json")
            } else {
                base.join("config.json")
            };

            if rust_json_path.exists() {
                if let Ok(data) = std::fs::read_to_string(&rust_json_path) {
                    if let Ok(store) = serde_json::from_str::<SettingsStore>(&data) {
                        migrated_settings = Some(store);
                    }
                }
            }
        }

        if migrated_settings.is_none() {
            // Check legacy WPF C# JSON path next
            let wpf_base = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Image Sort");
            let wpf_json_path = if cfg!(debug_assertions) {
                wpf_base.join("debug_config.json")
            } else {
                wpf_base.join("config.json")
            };

            if wpf_json_path.exists() {
                if let Ok(data) = std::fs::read_to_string(&wpf_json_path) {
                    if let Ok(store) = serde_json::from_str::<SettingsStore>(&data) {
                        migrated_settings = Some(store);
                    }
                }
            }
        }

        if let Some(store) = migrated_settings {
            // Persist the migrated settings immediately to the new TOML path
            store.save()?;
            Ok(store)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        let path = Self::config_path();
        let data = toml::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }
}
