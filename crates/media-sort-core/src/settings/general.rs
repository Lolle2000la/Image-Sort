use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_true")]
    pub check_for_updates_on_startup: bool,

    #[serde(default)]
    pub install_prerelease_builds: bool,

    #[serde(default = "default_true")]
    pub animate_gifs: bool,

    #[serde(default)]
    pub integration_with_windows: bool,

    #[serde(default = "default_true")]
    pub reopen_last_opened_folder: bool,

    #[serde(default)]
    pub last_opened_folder: Option<String>,

    #[serde(default)]
    pub locale: Option<String>,

    #[serde(default = "default_folder_tree_width")]
    pub folder_tree_width: u16,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            check_for_updates_on_startup: true,
            install_prerelease_builds: false,
            animate_gifs: true,
            integration_with_windows: false,
            reopen_last_opened_folder: true,
            last_opened_folder: None,
            locale: None,
            folder_tree_width: default_folder_tree_width(),
        }
    }
}

fn default_theme() -> String {
    "Light".to_string()
}

fn default_true() -> bool {
    true
}

fn default_folder_tree_width() -> u16 {
    240
}
