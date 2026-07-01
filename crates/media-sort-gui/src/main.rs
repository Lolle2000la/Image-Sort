mod app;
mod message;
mod state;
mod subscriptions;
mod update;
mod view;
mod widgets;

use iced::window;
use media_sort_core::settings::store::SettingsStore;
#[cfg(feature = "velopack")]
use serde::Deserialize;

use crate::message::Message;

#[cfg(feature = "velopack")]
const GITHUB_REPO_ID: u64 = 119281525;

#[cfg(feature = "velopack")]
#[derive(Deserialize)]
struct GithubRepoMetadata {
    html_url: String,
}

#[cfg(feature = "velopack")]
async fn fetch_canonical_repo_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("media-sort-gui-updater")
        .build()?;

    let url = format!("https://api.github.com/repositories/{}", GITHUB_REPO_ID);
    let metadata: GithubRepoMetadata = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(metadata.html_url)
}

pub fn run_background_update(
    settings: &media_sort_core::settings::general::GeneralSettings,
) -> iced::Task<Message> {
    #[cfg(not(feature = "velopack"))]
    {
        let _ = settings;
        iced::Task::none()
    }

    #[cfg(feature = "velopack")]
    {
        if !settings.check_for_updates_on_startup {
            return iced::Task::none();
        }

        let allow_prerelease =
            settings.install_prerelease_builds || env!("CARGO_PKG_VERSION").contains('-');

        iced::Task::perform(
            async move {
                let repo_url = fetch_canonical_repo_url()
                    .await
                    .map_err(|e| e.to_string())?;

                tokio::task::spawn_blocking(move || {
                    let source =
                        velopack::sources::GithubSource::new(&repo_url, None, allow_prerelease);
                    let um = velopack::UpdateManager::new(source, None, None)
                        .map_err(|e| e.to_string())?;

                    if let velopack::UpdateCheck::UpdateAvailable(updates) =
                        um.check_for_updates().map_err(|e| e.to_string())?
                    {
                        um.download_updates(&updates, None)
                            .map_err(|e| e.to_string())?;
                        um.apply_updates_and_restart(&*updates)
                            .map_err(|e| e.to_string())?;
                    }
                    Ok(())
                })
                .await
                .map_err(|e| e.to_string())?
            },
            crate::message::Message::UpdateCheckFinished,
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "velopack")]
    velopack::VelopackApp::build().run();

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

            let mut tasks = vec![];
            tasks.push(iced::Task::done(crate::message::Message::SettingsLoaded(
                Box::new(Ok(settings.clone())),
            )));

            if let Some(path) = startup_path {
                tasks.push(iced::Task::done(crate::message::Message::Folder(
                    crate::message::FolderMessage::Open(path),
                )));
            }

            tasks.push(run_background_update(&settings.general));

            (state, iced::Task::batch(tasks))
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
