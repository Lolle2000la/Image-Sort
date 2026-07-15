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
    #[serde(skip)]
    pub custom_path: Option<PathBuf>,

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
        if let Ok(val) = std::env::var("UI_TEST")
            && !val.is_empty()
        {
            return PathBuf::from("ui_test_config.toml");
        }
        let base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("media-sort");
        std::fs::create_dir_all(&base).ok();
        base.join("config.toml")
    }

    pub fn load() -> Result<Self, SettingsError> {
        let toml_path = Self::config_path();
        if toml_path.exists() {
            let data = std::fs::read_to_string(&toml_path)?;
            let mut store: SettingsStore = toml::from_str(&data)?;
            store.custom_path = Some(toml_path);
            return Ok(store);
        }

        // Search for legacy WPF C# JSON config to migrate
        if let Some(mut store) = {
            let wpf_base = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Image Sort");
            let wpf_json_path = if cfg!(debug_assertions) {
                wpf_base.join("debug_config.json")
            } else {
                wpf_base.join("config.json")
            };

            if wpf_json_path.exists()
                && let Ok(data) = std::fs::read_to_string(&wpf_json_path)
                && let Some(store) = parse_wpf_settings(&data)
            {
                Some(store)
            } else {
                None
            }
        } {
            store.custom_path = Some(toml_path.clone());
            store.save()?;
            return Ok(store);
        }

        let store = Self {
            custom_path: Some(toml_path),
            ..Self::default()
        };
        Ok(store)
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        let path = if let Some(ref custom) = self.custom_path {
            custom.clone()
        } else {
            Self::config_path()
        };
        let data = toml::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    pub fn parse_wpf_settings(data: &str) -> Option<Self> {
        parse_wpf_settings(data)
    }
}

#[derive(Debug, Deserialize)]
struct WpfHotkey {
    #[serde(rename = "Key")]
    key: i32,
    #[serde(rename = "Modifiers")]
    modifiers: i32,
}

fn map_wpf_key_to_rust(key_val: i32) -> String {
    match key_val {
        2 => "Backspace".to_string(),
        3 => "Tab".to_string(),
        6 => "Enter".to_string(),
        18 => "Space".to_string(),
        19 => "PageUp".to_string(),
        20 => "PageDown".to_string(),
        21 => "End".to_string(),
        22 => "Home".to_string(),
        23 => "Left".to_string(),
        24 => "Up".to_string(),
        25 => "Right".to_string(),
        26 => "Down".to_string(),
        27 => "Esc".to_string(),
        32 => "Delete".to_string(),
        val @ 34..=43 => ((val - 34 + i32::from(b'0')) as u8 as char).to_string(),
        val @ 44..=69 => ((val - 44 + i32::from(b'A')) as u8 as char).to_string(),
        val @ 74..=83 => ((val - 74 + i32::from(b'0')) as u8 as char).to_string(),
        val @ 90..=101 => {
            format!("F{}", val - 89)
        }
        _ => String::new(),
    }
}

fn parse_wpf_settings(data: &str) -> Option<SettingsStore> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    let mut store = SettingsStore::default();

    // 1. General settings
    if let Some(general) = json.get("General") {
        if let Some(val) = general.get("DarkMode").and_then(|v| v.as_bool()) {
            store.general.theme = if val {
                "Dark".to_string()
            } else {
                "Light".to_string()
            };
        }
        if let Some(val) = general
            .get("CheckForUpdatesOnStartup")
            .and_then(|v| v.as_bool())
        {
            store.general.check_for_updates_on_startup = val;
        }
        if let Some(val) = general
            .get("InstallPrereleaseBuilds")
            .and_then(|v| v.as_bool())
        {
            store.general.install_prerelease_builds = val;
        }
        if let Some(val) = general.get("AnimateGifs").and_then(|v| v.as_bool()) {
            store.general.animate_gifs = val;
        }
    }

    // 2. PinnedFolders settings
    if let Some(pinned) = json.get("PinnedFolders")
        && let Some(folders_val) = pinned.get("PinnedFolders")
        && let Some(arr) = folders_val.as_array()
    {
        store.pinned_folders.paths = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }

    // 3. MetadataPanel settings
    if let Some(meta) = json.get("MetadataPanel") {
        if let Some(val) = meta.get("IsExpanded").and_then(|v| v.as_bool()) {
            store.metadata_panel.is_expanded = val;
        }
        if let Some(val) = meta.get("MetadataPanelWidth").and_then(|v| v.as_u64()) {
            store.metadata_panel.panel_width = val as u16;
        }
    }

    // 4. WindowPosition settings
    if let Some(win) = json.get("MainWindow") {
        if let Some(val) = win.get("Left").and_then(|v| v.as_i64()) {
            store.window_position.left = val as i32;
        }
        if let Some(val) = win.get("Top").and_then(|v| v.as_i64()) {
            store.window_position.top = val as i32;
        }
        if let Some(val) = win.get("Width").and_then(|v| v.as_u64()) {
            store.window_position.width = val as u32;
        }
        if let Some(val) = win.get("Height").and_then(|v| v.as_u64()) {
            store.window_position.height = val as u32;
        }
        if let Some(val) = win.get("IsMaximized").and_then(|v| v.as_bool()) {
            store.window_position.maximized = val;
        }
        if let Some(val) = win.get("ScreenCount").and_then(|v| v.as_u64()) {
            store.window_position.screen_count = val as u32;
        }
    }

    // 5. KeyBindings settings
    if let Some(kb_val) = json.get("KeyBindings") {
        let parse_binding = |key_name: &str| -> Option<crate::settings::keybindings::KeyBinding> {
            let val = kb_val.get(key_name)?;
            let wpf_hk: WpfHotkey = serde_json::from_value(val.clone()).ok()?;
            let rust_key = map_wpf_key_to_rust(wpf_hk.key);
            if rust_key.is_empty() {
                return None;
            }
            Some(crate::settings::keybindings::KeyBinding {
                key: crate::settings::keybindings::Key::parse(&rust_key)?,
                ctrl: (wpf_hk.modifiers & 2) != 0,
                shift: (wpf_hk.modifiers & 4) != 0,
                alt: (wpf_hk.modifiers & 1) != 0,
                meta: false,
            })
        };

        if let Some(b) = parse_binding("Move") {
            store.keybindings.move_to_folder = b;
        }
        if let Some(b) = parse_binding("Delete") {
            store.keybindings.delete = b;
        }
        if let Some(b) = parse_binding("Rename") {
            store.keybindings.rename = b;
        }
        if let Some(b) = parse_binding("GoLeft") {
            store.keybindings.go_left = b;
        }
        if let Some(b) = parse_binding("GoRight") {
            store.keybindings.go_right = b;
        }
        if let Some(b) = parse_binding("CreateFolder") {
            store.keybindings.create_folder = b;
        }
        if let Some(b) = parse_binding("FolderUp") {
            store.keybindings.folder_up = b;
        }
        if let Some(b) = parse_binding("FolderLeft") {
            store.keybindings.folder_left = b;
        }
        if let Some(b) = parse_binding("FolderDown") {
            store.keybindings.folder_down = b;
        }
        if let Some(b) = parse_binding("FolderRight") {
            store.keybindings.folder_right = b;
        }
        if let Some(b) = parse_binding("Undo") {
            store.keybindings.undo = b;
        }
        if let Some(b) = parse_binding("Redo") {
            store.keybindings.redo = b;
        }
        if let Some(b) = parse_binding("OpenFolder") {
            store.keybindings.open_folder = b;
        }
        if let Some(b) = parse_binding("OpenSelectedFolder") {
            store.keybindings.open_selected_folder = b;
        }
        if let Some(b) = parse_binding("Pin") {
            store.keybindings.pin = b;
        }
        if let Some(b) = parse_binding("PinSelected") {
            store.keybindings.pin_selected = b;
        }
        if let Some(b) = parse_binding("Unpin") {
            store.keybindings.unpin = b;
        }
        if let Some(b) = parse_binding("MoveSelectedPinnedFolderUp") {
            store.keybindings.move_pinned_up = b;
        }
        if let Some(b) = parse_binding("MoveSelectedPinnedFolderDown") {
            store.keybindings.move_pinned_down = b;
        }
        if let Some(b) = parse_binding("SearchImages") {
            store.keybindings.search_images = b;
        }
        if let Some(b) = parse_binding("ToggleMetadataPanel") {
            store.keybindings.toggle_metadata_panel = b;
        }
    }

    Some(store)
}
