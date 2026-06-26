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
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::Io(e) => write!(f, "IO error: {e}"),
            SettingsError::Serde(e) => write!(f, "Serialization error: {e}"),
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
        let base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort");
        std::fs::create_dir_all(&base).ok();
        if cfg!(debug_assertions) {
            base.join("debug_config.json")
        } else {
            base.join("config.json")
        }
    }

    pub fn load() -> Result<Self, SettingsError> {
        let path = Self::config_path();
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        let path = Self::config_path();
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }
}
