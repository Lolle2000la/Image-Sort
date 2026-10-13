use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinnedFoldersSettings {
    pub paths: Vec<String>,
}
