mod app;
mod message;
mod state;
mod subscriptions;
mod update;
mod view;
mod widgets;

use iced::window;
use media_sort_core::settings::store::SettingsStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .init();

    let discovered =
        media_sort_backend::media::mpv_context::MpvContext::query_supported_extensions();
    media_sort_core::media_type::MediaRegistry::init(discovered);

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

    iced::application(
        move || {
            let settings = SettingsStore::load().unwrap_or_default();
            let state = crate::state::AppState::new(settings.clone());
            let mut startup_path = None;

            if state.settings.general.reopen_last_opened_folder
                && let Some(ref last_path_str) = state.settings.general.last_opened_folder
            {
                let last_path = std::path::PathBuf::from(last_path_str);
                if last_path.exists() {
                    startup_path = Some(last_path);
                }
            }

            if startup_path.is_none()
                && let Some(pic_dir) = dirs::picture_dir()
                && pic_dir.exists()
            {
                startup_path = Some(pic_dir);
            }

            let task = {
                let loaded_settings =
                    crate::message::Message::SettingsLoaded(Box::new(Ok(settings.clone())));
                if let Some(path) = startup_path {
                    iced::Task::batch([
                        iced::Task::done(loaded_settings),
                        iced::Task::done(crate::message::Message::Folder(
                            crate::message::FolderMessage::Open(path),
                        )),
                    ])
                } else {
                    iced::Task::done(loaded_settings)
                }
            };

            (state, task)
        },
        app::update,
        app::view,
    )
    .title("Media Sort")
    .theme(app::theme)
    .subscription(app::subscription)
    .font(lucide_icons::LUCIDE_FONT_BYTES)
    .window(window_settings)
    .run()?;

    Ok(())
}
