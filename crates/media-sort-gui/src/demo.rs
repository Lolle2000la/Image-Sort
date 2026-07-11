use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use iced_automation::{AutomationStateTrait, DemoApp};

use crate::message::{FolderMessage, Message};
use crate::state::AppState;

impl DemoApp for AppState {
    type Message = Message;
    type Settings = media_sort_core::settings::store::SettingsStore;

    fn new_app_state(settings: &Self::Settings) -> Self {
        AppState::new(settings.clone())
    }

    fn default_settings() -> Self::Settings {
        media_sort_core::settings::store::SettingsStore::default()
    }

    fn resolve_widget_id(fixture_root: &Path, json_id: &str) -> String {
        if fixture_root.join(json_id).is_dir() {
            let folder_path = fixture_root.join(json_id);
            format!("folder_{}", folder_path.display())
        } else {
            json_id.to_string()
        }
    }

    fn format_keycap(message: &Self::Message) -> String {
        message.automation_keycap().unwrap_or("Action").to_string()
    }

    fn bootstrap_messages(settings: &Self::Settings, demo_root: &Path) -> Vec<Self::Message> {
        vec![
            Message::Folder(FolderMessage::Open(demo_root.to_path_buf())),
            Message::SettingsLoaded(Box::new(Ok(settings.clone()))),
        ]
    }
}

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const FPS: u32 = 60;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| manifest_dir.clone())
}

fn resolve_workspace_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        workspace_root().join(path)
    }
}

fn find_spec_path(spec_path: &Path) -> PathBuf {
    if spec_path.is_dir() {
        let default_flow = spec_path.join("sorting_workflow.json");
        if default_flow.exists() {
            return default_flow;
        }
        if let Ok(entries) = std::fs::read_dir(spec_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    return path;
                }
            }
        }
    }
    spec_path.to_path_buf()
}

pub fn try_headless_export(cli: &crate::Cli) -> Option<Result<(), Box<dyn std::error::Error>>> {
    if !cli.export {
        return None;
    }

    let export_path = resolve_workspace_path(&cli.demo_export);
    let spec_path = resolve_workspace_path(&cli.demo_spec);

    let run = || -> Result<(), Box<dyn std::error::Error>> {
        if spec_path.is_dir() {
            if !export_path.exists() {
                std::fs::create_dir_all(&export_path)?;
            }
            if !export_path.is_dir() {
                return Err("MEDIA_SORT_DEMO_EXPORT must be a directory because MEDIA_SORT_DEMO_SPEC is a directory".into());
            }

            let entries = std::fs::read_dir(&spec_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    let file_stem = path.file_stem().unwrap().to_string_lossy();
                    let output_video_path = export_path.join(format!("{}.mp4", file_stem));

                    tracing::info!(
                        "Rendering demo: {} -> {:?}",
                        path.display(),
                        output_video_path
                    );

                    export_demo_video(&path, &output_video_path)?;
                }
            }
            Ok(())
        } else {
            export_demo_video(&spec_path, &export_path)
        }
    };

    Some(run())
}

pub fn init(cli: &crate::Cli, state: &mut AppState) -> Option<PathBuf> {
    if !cli.demo {
        return None;
    }

    let spec_path = resolve_workspace_path(&cli.demo_spec);
    let final_spec_path = find_spec_path(&spec_path);

    let demo_root = std::env::temp_dir().join(format!("media_sort_demo_{}", std::process::id()));
    let mock_state_src = resolve_workspace_path("resources/MockState");

    let config = iced_automation::DemoConfig {
        spec_path: final_spec_path,
        fixture: Some(iced_automation::FixtureSpec {
            source: mock_state_src,
            target: Some(demo_root.clone()),
        }),
        variable_substitutions: std::collections::HashMap::new(),
        window_width: state.settings.window_position.width as f32,
        window_height: state.settings.window_position.height as f32,
        style: iced_automation::AutomationStyle::default(),
    };

    let bootstrap = iced_automation::init_demo::<AppState>(&config).expect("failed to init demo");

    state.automation = bootstrap.state.automation;

    Some(bootstrap.demo_root)
}

pub fn export_demo_video(
    json_spec_path: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let completed = Arc::new(AtomicBool::new(false));
    let mock_state_src = resolve_workspace_path("resources/MockState");

    let config = iced_automation::DemoConfig {
        spec_path: json_spec_path.to_path_buf(),
        fixture: Some(iced_automation::FixtureSpec {
            source: mock_state_src,
            target: None,
        }),
        variable_substitutions: std::collections::HashMap::new(),
        window_width: DEFAULT_WIDTH as f32,
        window_height: DEFAULT_HEIGHT as f32,
        style: iced_automation::AutomationStyle::default(),
    };

    let bootstrap = iced_automation::init_demo::<AppState>(&config).expect("failed to init demo");

    let completed_clone = completed.clone();
    let headless_app = bootstrap.into_headless_app(
        "MediaSort",
        iced_test::core::Settings {
            id: Some("mediasort".into()),
            fonts: vec![],
            default_font: iced::Font::DEFAULT,
            default_text_size: iced::Pixels(16.0),
            antialiasing: false,
            vsync: false,
        },
        move |state: &mut AppState, msg: Message| {
            let task = crate::app::update(state, msg);
            if state.is_automation_completed() {
                completed_clone.store(true, Ordering::SeqCst);
            }
            task
        },
        |state: &AppState, _window| crate::app::view(state),
        |state: &AppState, _window| Some(crate::app::theme(state)),
        |state: &AppState| crate::app::subscription(state),
    );

    let video_config = iced_automation::ExportVideoConfig {
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        fps: FPS,
        output_path: output_path.to_path_buf(),
        tick_message: Message::AutomationVirtualTick,
        extra_fonts: vec![std::borrow::Cow::Borrowed(lucide_icons::LUCIDE_FONT_BYTES)],
    };

    iced_automation::export_video(&headless_app, completed, &video_config)?;

    Ok(())
}
