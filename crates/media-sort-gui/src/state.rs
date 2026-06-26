use media_sort_core::history::History;
use media_sort_core::settings::store::SettingsStore;

pub struct AppState {
    #[allow(dead_code)]
    pub history: History,
    pub settings: SettingsStore,
    pub current_folder: Option<std::path::PathBuf>,
    pub should_exit: bool,
}

impl AppState {
    pub fn new(settings: SettingsStore) -> Self {
        Self {
            history: History::new(),
            settings,
            current_folder: None,
            should_exit: false,
        }
    }
}
