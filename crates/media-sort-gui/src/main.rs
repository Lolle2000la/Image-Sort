#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
#[cfg(feature = "demo")]
mod automation;
#[cfg(feature = "demo")]
mod demo;
#[cfg(feature = "demo")]
mod headless;
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

#[cfg(feature = "velopack")]
pub async fn check_for_update_async(
    settings: &media_sort_core::settings::general::GeneralSettings,
) -> Result<Option<velopack::UpdateInfo>, String> {
    let repo_url = fetch_canonical_repo_url()
        .await
        .map_err(|e| e.to_string())?;
    let allow_prerelease =
        settings.install_prerelease_builds || env!("CARGO_PKG_VERSION").contains('-');

    tokio::task::spawn_blocking(move || {
        let source = velopack::sources::GithubSource::new(&repo_url, None, allow_prerelease);
        let um = velopack::UpdateManager::new(source, None, None).map_err(|e| e.to_string())?;
        match um.check_for_updates().map_err(|e| e.to_string())? {
            velopack::UpdateCheck::UpdateAvailable(update) => Ok(Some(*update)),
            _ => Ok(None),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(feature = "velopack")]
pub async fn download_and_apply_async(info: velopack::UpdateInfo) -> Result<(), String> {
    let repo_url = fetch_canonical_repo_url()
        .await
        .map_err(|e| e.to_string())?;

    tokio::task::spawn_blocking(move || {
        let source = velopack::sources::GithubSource::new(&repo_url, None, false);
        let um = velopack::UpdateManager::new(source, None, None).map_err(|e| e.to_string())?;
        um.download_updates(&info, None)
            .map_err(|e| e.to_string())?;
        um.apply_updates_and_restart(&info)
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn initialize_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
            })
            .unwrap_or("Unknown panic payload");

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let crash_log = format!(
            "--- MEDIA SORT CRASH LOG ---\n\
             Location: {}\n\
             Cause: {}\n\
             Backtrace:\n{}\n",
            location,
            payload,
            std::backtrace::Backtrace::force_capture(),
        );

        if let Some(mut log_path) = dirs::data_local_dir() {
            log_path.push("media-sort");
            log_path.push("crashes");
            let _ = std::fs::create_dir_all(&log_path);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string());
            log_path.push(format!("crash_{}.log", ts));
            let _ = std::fs::write(&log_path, &crash_log);
        }

        default_hook(panic_info);
    }));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "velopack")]
    {
        let mut app = velopack::VelopackApp::build();
        #[cfg(target_os = "windows")]
        {
            app = app.on_before_uninstall_fast_callback(|_version| {
                let _ = media_sort_backend::platform::windows_shell::unregister();
            });
        }
        app.run();
    }

    initialize_panic_hook();

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

    #[cfg(feature = "demo")]
    if let Some(result) = crate::demo::try_headless_export() {
        return result;
    }

    let icon =
        window::icon::from_file_data(include_bytes!("../../../packaging/windows/icon.ico"), None)
            .ok();

    let window_settings = window::Settings {
        position: window::Position::Specific(iced::Point::new(
            settings.window_position.left as f32,
            settings.window_position.top as f32,
        )),
        size: iced::Size::new(
            settings.window_position.width as f32,
            settings.window_position.height as f32,
        ),
        icon,
        #[allow(unused_mut)]
        platform_specific: {
            let mut ps = window::settings::PlatformSpecific::default();
            #[cfg(target_os = "linux")]
            {
                ps.application_id = String::from("MediaSort");
            }
            ps
        },
        ..window::Settings::default()
    };

    iced::application(
        move || {
            let settings = SettingsStore::load().unwrap_or_default();
            let state = crate::state::AppState::new(settings.clone());
            #[cfg(feature = "demo")]
            let mut state = state;
            let mut startup_path = None;

            #[cfg(feature = "demo")]
            if let Some(root) = crate::demo::init(&mut state) {
                startup_path = Some(root);
            }

            if let Some(arg_path) = std::env::args()
                .nth(1)
                .map(std::path::PathBuf::from)
                .filter(|p| p.is_dir())
            {
                startup_path = Some(arg_path);
            }

            if startup_path.is_none()
                && state.settings.general.reopen_last_opened_folder
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

            #[cfg(feature = "velopack")]
            if state.settings.general.check_for_updates_on_startup {
                tasks.push(iced::Task::done(crate::message::Message::Update(
                    crate::message::UpdateMessage::CheckForUpdates,
                )));
            }

            (state, iced::Task::batch(tasks))
        },
        app::update,
        app::view,
    )
    .settings(iced::Settings {
        id: Some(String::from("MediaSort")),
        ..iced::Settings::default()
    })
    .title("Media Sort")
    .theme(app::theme)
    .subscription(app::subscription)
    .font(lucide_icons::LUCIDE_FONT_BYTES)
    .window(window_settings)
    .run()?;

    Ok(())
}
