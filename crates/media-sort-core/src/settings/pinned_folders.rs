use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PinnedFoldersSettings {
    pub paths: Vec<String>,
}
