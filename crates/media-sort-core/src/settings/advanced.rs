use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdvancedSettings {
    /// Disables hardware video decoding. Hardware decoding is enabled by
    /// default; when enabled, video frames are rendered at their native
    /// (mpv default) resolution.
    #[serde(default)]
    pub disable_hardware_decoding: bool,
}
