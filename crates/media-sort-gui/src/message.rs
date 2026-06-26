use std::time::Instant;

use media_sort_core::settings::store::SettingsStore;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    Tick(Instant),
    SettingsLoaded(Box<Result<SettingsStore, String>>),
    Quit,
}
