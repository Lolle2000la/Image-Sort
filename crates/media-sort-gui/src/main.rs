mod app;
mod cache;
mod message;
mod state;
mod subscriptions;
mod theme;
mod update;
mod view;
mod widgets;

use iced::window;
use media_sort_core::settings::store::SettingsStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let settings = SettingsStore::load().unwrap_or_default();

    let window_settings = window::Settings {
        position: window::Position::Specific(iced::Point::new(
            settings.window_position.left as f32,
            settings.window_position.top as f32,
        )),
        size: iced::Size::new(
            settings.window_position.width as f32,
            settings.window_position.height as f32,
        ),
        ..window::Settings::default()
    };

    iced::application("Media Sort", app::update, app::view)
        .theme(app::theme)
        .subscription(app::subscription)
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .window(window_settings)
        .run_with(move || {
            let state = crate::state::AppState::new(settings);
            let mut startup_path = None;

            if state.settings.general.reopen_last_opened_folder {
                if let Some(ref last_path_str) = state.settings.general.last_opened_folder {
                    let last_path = std::path::PathBuf::from(last_path_str);
                    if last_path.exists() {
                        startup_path = Some(last_path);
                    }
                }
            }

            if startup_path.is_none() {
                if let Some(pic_dir) = dirs::picture_dir() {
                    if pic_dir.exists() {
                        startup_path = Some(pic_dir);
                    }
                }
            }

            let task = if let Some(path) = startup_path {
                iced::Task::done(crate::message::Message::OpenFolder(path))
            } else {
                iced::Task::none()
            };

            (state, task)
        })?;

    Ok(())
}
