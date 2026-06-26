use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataPanelSettings {
    #[serde(default)]
    pub is_expanded: bool,
    #[serde(default = "default_width")]
    pub panel_width: u16,
}

impl Default for MetadataPanelSettings {
    fn default() -> Self {
        Self {
            is_expanded: false,
            panel_width: default_width(),
        }
    }
}

fn default_width() -> u16 {
    300
}
