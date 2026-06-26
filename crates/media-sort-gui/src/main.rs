mod app;
mod message;
mod state;
mod update;

use media_sort_core::settings::store::SettingsStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let settings = SettingsStore::load().unwrap_or_default();

    iced::application("Media Sort", app::update, app::view)
        .theme(app::theme)
        .subscription(app::subscription)
        .run_with(move || {
            let state = crate::state::AppState::new(settings);
            (state, iced::Task::none())
        })?;

    Ok(())
}
