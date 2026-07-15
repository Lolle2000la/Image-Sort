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

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use crate::settings::keybindings::Key;
    use crate::settings::metadata_panel::MetadataPanelSettings;
    use crate::settings::store::{SettingsError, SettingsStore};
    use crate::settings::window_position::WindowPosition;

    fn test_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("media-sort-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    fn test_temp_subdir() -> PathBuf {
        let dir = test_temp_dir().join(format!("sub-{}", rand_u32()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    fn rand_u32() -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }

    #[test]
    fn test_settings_default() {
        let settings = SettingsStore::default();
        assert_eq!(settings.general.theme, "Light");
        assert!(settings.general.check_for_updates_on_startup);
        assert!(settings.general.animate_gifs);
    }

    #[test]
    fn test_settings_save_load_roundtrip() {
        let mut settings = SettingsStore::default();
        settings.general.theme = "Dark".to_string();
        settings.general.check_for_updates_on_startup = false;
        settings.general.animate_gifs = false;

        let json = serde_json::to_string(&settings).unwrap();
        let loaded: SettingsStore = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.general.theme, "Dark");
        assert!(!loaded.general.check_for_updates_on_startup);
        assert!(!loaded.general.animate_gifs);
    }

    #[test]
    fn test_settings_keybindings_defaults() {
        let kb = &SettingsStore::default().keybindings;

        let keys = [
            &kb.move_to_folder.key,
            &kb.delete.key,
            &kb.rename.key,
            &kb.go_left.key,
            &kb.go_right.key,
            &kb.create_folder.key,
            &kb.folder_up.key,
            &kb.folder_left.key,
            &kb.folder_down.key,
            &kb.folder_right.key,
            &kb.undo.key,
            &kb.redo.key,
            &kb.open_folder.key,
            &kb.open_selected_folder.key,
            &kb.pin.key,
            &kb.pin_selected.key,
            &kb.unpin.key,
            &kb.move_pinned_up.key,
            &kb.move_pinned_down.key,
            &kb.search_images.key,
            &kb.toggle_metadata_panel.key,
        ];

        assert_eq!(keys.len(), 21);
        for (i, key) in keys.iter().enumerate() {
            let name = key.display_name();
            assert!(!name.is_empty(), "keybinding {} has empty key", i);
        }
    }

    #[test]
    fn test_settings_empty_json_uses_defaults() {
        let json = "{}";
        let settings: SettingsStore = serde_json::from_str(json).unwrap();
        assert_eq!(settings.general.theme, "Light");
        assert!(settings.general.check_for_updates_on_startup);
        assert!(
            !settings
                .keybindings
                .move_to_folder
                .key
                .display_name()
                .is_empty()
        );
    }

    #[test]
    fn test_window_position_default() {
        let wp = WindowPosition::default();
        assert_eq!(wp.left, 100);
        assert_eq!(wp.top, 100);
        assert_eq!(wp.width, 1000);
        assert_eq!(wp.height, 600);
        assert!(!wp.maximized);
        assert_eq!(wp.screen_count, 1);
    }

    #[test]
    fn test_metadata_panel_settings_default() {
        let mps = MetadataPanelSettings::default();
        assert!(!mps.is_expanded);
        assert_eq!(mps.panel_width, 300);
    }

    #[test]
    fn test_settings_load_corrupted_json() {
        let dir = std::env::temp_dir().join(format!("mediasort_config_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("test_config.json");

        std::fs::write(&config_path, "this is not valid json {{{").unwrap();

        let result = std::fs::read_to_string(&config_path)
            .map_err(SettingsError::from)
            .and_then(|data| {
                serde_json::from_str::<SettingsStore>(&data).map_err(SettingsError::from)
            });
        assert!(result.is_err());
        match result {
            Err(SettingsError::Serde(_)) => {}
            _ => panic!("Expected Serde error, got {:?}", result.err()),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_settings_load_truncated_toml() {
        let dir = std::env::temp_dir().join(format!("mediasort_config2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("test_config.toml");

        std::fs::write(&config_path, "[general]\ntheme = \"Dar").unwrap();

        let result = std::fs::read_to_string(&config_path)
            .map_err(SettingsError::from)
            .and_then(|data| toml::from_str::<SettingsStore>(&data).map_err(SettingsError::from));
        assert!(result.is_err());
        match result {
            Err(SettingsError::TomlDe(_)) => {}
            _ => panic!("Expected TomlDe error, got {:?}", result.err()),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_settings_load_extra_unknown_fields() {
        let dir = std::env::temp_dir().join(format!("mediasort_config3_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("test_config.json");

        std::fs::write(
            &config_path,
            r#"{"general": {"theme": "Dark"}, "unknown_field": "should be ignored"}"#,
        )
        .unwrap();

        let result = std::fs::read_to_string(&config_path)
            .map_err(SettingsError::from)
            .and_then(|data| {
                serde_json::from_str::<SettingsStore>(&data).map_err(SettingsError::from)
            });
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.general.theme, "Dark");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_wpf_settings_migration() {
        let raw_json = r#"{
            "General": {
                "DarkMode": true,
                "CheckForUpdatesOnStartup": false,
                "InstallPrereleaseBuilds": true,
                "AnimateGifs": false
            },
            "PinnedFolders": {
                "PinnedFolders": [
                    "/some/path/1",
                    "/some/path/2"
                ]
            },
            "MetadataPanel": {
                "IsExpanded": true,
                "MetadataPanelWidth": 250
            },
            "MainWindow": {
                "Left": 50,
                "Top": 60,
                "Width": 1200,
                "Height": 800,
                "IsMaximized": true,
                "ScreenCount": 2
            },
            "KeyBindings": {
                "Move": { "Key": 24, "Modifiers": 0 },
                "Delete": { "Key": 26, "Modifiers": 2 },
                "SearchImages": { "Key": 52, "Modifiers": 4 }
            }
        }"#;

        let store =
            SettingsStore::parse_wpf_settings(raw_json).expect("Failed to parse wpf settings");

        assert_eq!(store.general.theme, "Dark");
        assert!(!store.general.check_for_updates_on_startup);
        assert!(store.general.install_prerelease_builds);
        assert!(!store.general.animate_gifs);

        assert_eq!(store.pinned_folders.paths.len(), 2);
        assert_eq!(store.pinned_folders.paths[0], "/some/path/1");
        assert_eq!(store.pinned_folders.paths[1], "/some/path/2");

        assert!(store.metadata_panel.is_expanded);
        assert_eq!(store.metadata_panel.panel_width, 250);

        assert_eq!(store.window_position.left, 50);
        assert_eq!(store.window_position.top, 60);
        assert_eq!(store.window_position.width, 1200);
        assert_eq!(store.window_position.height, 800);
        assert!(store.window_position.maximized);
        assert_eq!(store.window_position.screen_count, 2);

        assert_eq!(store.keybindings.move_to_folder.key, Key::ArrowUp);
        assert!(!store.keybindings.move_to_folder.ctrl);
        assert_eq!(store.keybindings.delete.key, Key::ArrowDown);
        assert!(store.keybindings.delete.ctrl);
        assert_eq!(store.keybindings.search_images.key, Key::Character('I'));
        assert!(store.keybindings.search_images.shift);
    }

    #[test]
    fn test_parse_wpf_settings_empty_json() {
        let store = SettingsStore::parse_wpf_settings("{}");
        assert!(
            store.is_some(),
            "empty JSON should produce default settings"
        );
    }

    #[test]
    fn test_parse_wpf_settings_partial_json() {
        let json = r#"{"General": {"DarkMode": true}}"#;
        let store = SettingsStore::parse_wpf_settings(json);
        assert!(store.is_some());
        let s = store.unwrap();
        assert_eq!(s.general.theme, "Dark");
        assert!(s.general.animate_gifs);
    }

    #[test]
    fn test_parse_wpf_settings_pinned_folders_not_array() {
        let json = r#"{"PinnedFolders": {"PinnedFolders": "not an array"}}"#;
        let store = SettingsStore::parse_wpf_settings(json);
        assert!(
            store.is_some(),
            "should not crash on non-array pinned folders"
        );
        let s = store.unwrap();
        assert!(s.pinned_folders.paths.is_empty());
    }

    #[test]
    fn test_parse_wpf_settings_null_pinned_folders() {
        let json = r#"{"PinnedFolders": {"PinnedFolders": null}}"#;
        let store = SettingsStore::parse_wpf_settings(json);
        assert!(store.is_some());
        let s = store.unwrap();
        assert!(s.pinned_folders.paths.is_empty());
    }

    #[test]
    fn test_parse_wpf_settings_unknown_theme() {
        let json = r#"{"ThemeSettings": {"Theme": "SomeUnknownTheme"}}"#;
        let store = SettingsStore::parse_wpf_settings(json);
        assert!(store.is_some());
        let s = store.unwrap();
        assert!(!s.general.theme.is_empty());
    }

    #[test]
    fn test_settings_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test io");
        let se = SettingsError::Io(io_err);
        assert!(se.to_string().contains("IO error"));

        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let se2 = SettingsError::Serde(json_err);
        assert!(se2.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_settings_error_from_io() {
        let io_err = io::Error::other("test");
        let se: SettingsError = io_err.into();
        assert!(matches!(se, SettingsError::Io(_)));
    }

    #[test]
    fn test_settings_error_from_serde() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let se: SettingsError = json_err.into();
        assert!(matches!(se, SettingsError::Serde(_)));
    }

    #[test]
    fn test_settings_error_toml_display() {
        let toml_err = toml::from_str::<String>("[[[").unwrap_err();
        let se = SettingsError::TomlDe(toml_err);
        let s = se.to_string();
        assert!(s.contains("TOML deserialization"));

        let bad_val = f64::NAN;
        let toml_ser_err = toml::to_string(&bad_val).unwrap_err();
        let se2 = SettingsError::TomlSer(toml_ser_err);
        let s2 = se2.to_string();
        assert!(s2.contains("TOML serialization"));
    }

    #[test]
    fn test_settings_toml_roundtrip() {
        let mut settings = SettingsStore::default();
        settings.general.theme = "Dark".to_string();
        settings.general.locale = Some("de".to_string());
        settings.general.animate_gifs = false;
        settings.window_position.left = 42;
        settings.window_position.top = 99;
        settings.window_position.maximized = true;
        settings.metadata_panel.is_expanded = true;
        settings.metadata_panel.panel_width = 400;
        settings.pinned_folders.paths = vec!["/home/test".to_string(), "/tmp/foo".to_string()];

        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let loaded: SettingsStore = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.general.theme, "Dark");
        assert_eq!(loaded.general.locale, Some("de".to_string()));
        assert!(!loaded.general.animate_gifs);
        assert_eq!(loaded.window_position.left, 42);
        assert_eq!(loaded.window_position.top, 99);
        assert!(loaded.window_position.maximized);
        assert!(loaded.metadata_panel.is_expanded);
        assert_eq!(loaded.metadata_panel.panel_width, 400);
        assert_eq!(loaded.pinned_folders.paths, vec!["/home/test", "/tmp/foo"]);
        assert!(
            !loaded
                .keybindings
                .move_to_folder
                .key
                .display_name()
                .is_empty()
        );
    }

    #[test]
    fn test_settings_save_load_file_roundtrip() {
        let dir = test_temp_subdir();
        let config_file = dir.join("config.toml");

        let mut settings = SettingsStore::default();
        settings.general.theme = "Slate".to_string();
        settings.general.animate_gifs = false;
        settings.general.folder_tree_width = 300;

        let toml_str = toml::to_string_pretty(&settings).unwrap();
        std::fs::write(&config_file, &toml_str).unwrap();

        let loaded_str = std::fs::read_to_string(&config_file).unwrap();
        let loaded: SettingsStore = toml::from_str(&loaded_str).unwrap();
        assert_eq!(loaded.general.theme, "Slate");
        assert!(!loaded.general.animate_gifs);
        assert_eq!(loaded.general.folder_tree_width, 300);
    }
}
