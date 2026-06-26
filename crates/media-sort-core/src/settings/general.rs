use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default)]
    pub dark_mode: bool,

    #[serde(default = "default_true")]
    pub check_for_updates_on_startup: bool,

    #[serde(default)]
    pub install_prerelease_builds: bool,

    #[serde(default = "default_true")]
    pub animate_gifs: bool,

    #[serde(default = "default_true")]
    pub animate_gif_thumbnails: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            check_for_updates_on_startup: true,
            install_prerelease_builds: false,
            animate_gifs: true,
            animate_gif_thumbnails: true,
        }
    }
}

fn default_true() -> bool {
    true
}
