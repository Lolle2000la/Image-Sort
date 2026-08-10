#[derive(Debug, Clone, Default)]
pub enum SettingsUiState {
    #[default]
    Hidden,
    Settings,
    Advanced,
    Keybindings {
        editing_keybinding: Option<usize>,
        waiting_for_key: bool,
    },
}
