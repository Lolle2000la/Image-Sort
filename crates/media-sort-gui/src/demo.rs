use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};
use crate::state::AppState;

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

/// Called at the top of `main()`, before the iced application builder.
/// Returns `Some(Ok(()))` if headless export completed, or `Some(Err(...))`
/// on failure.  Returns `None` when no export env var is set.
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

                    let demo_root = init_demo_media();
                    export_demo_video(
                        demo_root,
                        &path.to_string_lossy(),
                        &output_video_path.to_string_lossy(),
                    )?;
                }
            }
            Ok(())
        } else {
            let demo_root = init_demo_media();
            export_demo_video(
                demo_root,
                &spec_path.to_string_lossy(),
                &export_path.to_string_lossy(),
            )
        }
    };

    Some(run())
}

fn build_automation_flow(
    spec_path: &str,
    demo_root: &Path,
) -> (
    iced_automation::JsonAutomationFlow<Message>,
    Vec<iced_automation::AutomationStep<Message>>,
) {
    let spec_content = std::fs::read_to_string(spec_path).expect("failed to read spec file");
    let resolved_content = spec_content.replace("$DEMO_ROOT", &demo_root.to_string_lossy());
    let flow: iced_automation::JsonAutomationFlow<Message> =
        serde_json::from_str(&resolved_content).expect("failed to parse spec JSON");
    let steps = build_steps(&flow, demo_root);
    (flow, steps)
}

/// Called inside the iced application factory closure.
/// Sets up automation state and returns the demo root as the startup path.
/// Returns `None` when `MEDIA_SORT_DEMO` is not set.
pub fn init(cli: &crate::Cli, state: &mut crate::state::AppState) -> Option<PathBuf> {
    if !cli.demo {
        return None;
    }

    let spec_path = resolve_workspace_path(&cli.demo_spec);
    let final_spec_path = if spec_path.is_dir() {
        // Look for sorting_workflow.json or any json file in the directory
        let default_flow = spec_path.join("sorting_workflow.json");
        if default_flow.exists() {
            default_flow
        } else {
            let mut found = None;
            if let Ok(entries) = std::fs::read_dir(&spec_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                        found = Some(path);
                        break;
                    }
                }
            }
            found.unwrap_or(spec_path)
        }
    } else {
        spec_path
    };

    let demo_root = std::env::temp_dir().join(format!("media_sort_demo_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&demo_root);

    let mock_state_src = resolve_workspace_path("resources/MockState");
    copy_dir_all(&mock_state_src, &demo_root).expect("failed to copy MockState");
    tracing::info!("Concrete MockState copied to {:?}", demo_root);

    let ww = state.settings.window_position.width as f32;
    let wh = state.settings.window_position.height as f32;

    let (flow, steps) = build_automation_flow(&final_spec_path.to_string_lossy(), &demo_root);

    state.automation = Some(iced_automation::AutomationState::new(
        steps,
        &flow.flow_name,
        ww,
        wh,
        iced_automation::AutomationStyle::default(),
    ));

    Some(demo_root)
}

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn init_demo_media() -> PathBuf {
    let demo_root = std::env::temp_dir().join(format!("media_sort_demo_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&demo_root);

    let mock_state_src = resolve_workspace_path("resources/MockState");
    copy_dir_all(&mock_state_src, &demo_root).expect("failed to copy MockState");
    tracing::info!("Concrete MockState copied to {:?}", demo_root);
    demo_root
}

fn format_message_for_keycaster(msg: &Message) -> String {
    match msg {
        Message::Media(MediaMessage::GoRight) => "Right Arrow\nNext Image".into(),
        Message::Media(MediaMessage::GoLeft) => "Left Arrow\nPrevious Image".into(),
        Message::Media(MediaMessage::MoveActive) => "M\nMove to Folder".into(),
        Message::Media(MediaMessage::CopyActive) => "Ctrl+C\nCopy to Folder".into(),
        Message::Media(MediaMessage::TriggerRename) => "F2\nRename".into(),
        Message::Media(MediaMessage::SearchQueryChanged(_)) => "Type Query\nFilter Results".into(),
        Message::Media(MediaMessage::SearchFocused) => "Ctrl+F\nFocus Search".into(),
        Message::Media(MediaMessage::SelectEntry(_)) => "Click\nSelect Entry".into(),
        Message::Folder(FolderMessage::Open(_)) => "Enter\nOpen Folder".into(),
        Message::Folder(FolderMessage::ToggleExpand(_)) => "Space\nExpand Folder".into(),
        Message::Folder(FolderMessage::Selected(..)) => "Arrow Keys\nSelect Destination".into(),
        Message::Settings(SettingsMessage::Open) => "Ctrl+,\nSettings".into(),
        Message::Settings(SettingsMessage::SetTheme(_)) => "Ctrl+D\nChange Theme".into(),
        Message::Settings(SettingsMessage::Close) => "Esc\nClose".into(),
        Message::Quit => "Ctrl+Q\nQuit".into(),
        _ => "Action".into(),
    }
}

fn build_steps(
    flow: &iced_automation::JsonAutomationFlow<Message>,
    test_root: &Path,
) -> Vec<iced_automation::AutomationStep<Message>> {
    iced_automation::build_automation_steps(
        flow,
        |id| {
            if test_root.join(id).is_dir() {
                let folder_path = test_root.join(id);
                format!("folder_{}", folder_path.display())
            } else {
                id.to_string()
            }
        },
        format_message_for_keycaster,
    )
}

fn create_demo_state(
    settings: &media_sort_core::settings::store::SettingsStore,
    demo_root: &Path,
    steps: &[iced_automation::AutomationStep<Message>],
    flow_name: &str,
) -> (AppState, iced::Task<Message>) {
    let mut state = AppState::new(settings.clone());

    state.automation = Some(iced_automation::AutomationState::new(
        steps.to_vec(),
        flow_name,
        DEFAULT_WIDTH as f32,
        DEFAULT_HEIGHT as f32,
        iced_automation::AutomationStyle::default(),
    ));

    let tasks = vec![
        iced::Task::done(Message::Folder(crate::message::FolderMessage::Open(
            demo_root.to_path_buf(),
        ))),
        iced::Task::done(Message::SettingsLoaded(Box::new(Ok(settings.clone())))),
    ];

    (state, iced::Task::batch(tasks))
}

pub fn export_demo_video(
    demo_root: std::path::PathBuf,
    json_spec_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = media_sort_core::settings::store::SettingsStore::default();
    let completed = Arc::new(AtomicBool::new(false));

    let (flow, steps) = build_automation_flow(json_spec_path, &demo_root);

    let flow_name = flow.flow_name.clone();
    let (boot_state, boot_task) = create_demo_state(&settings, &demo_root, &steps, &flow_name);

    let completed_clone = completed.clone();
    let headless_app = iced_automation::HeadlessApp::new(
        "MediaSort",
        iced_test::core::Settings {
            id: Some("mediasort".into()),
            fonts: vec![],
            default_font: iced::Font::DEFAULT,
            default_text_size: iced::Pixels(16.0),
            antialiasing: false,
            vsync: false,
        },
        boot_state,
        boot_task,
        move |state: &mut AppState, msg: Message| {
            let task = crate::app::update(state, msg);
            if state.automation.as_ref().is_some_and(|a| a.completed) {
                completed_clone.store(true, Ordering::SeqCst);
            }
            task
        },
        |state: &AppState, _window| crate::app::view(state),
        |state: &AppState, _window| Some(crate::app::theme(state)),
        |state: &AppState| crate::app::subscription(state),
    );

    let extra_fonts = vec![std::borrow::Cow::Borrowed(lucide_icons::LUCIDE_FONT_BYTES)];

    iced_automation::export_video(
        &headless_app,
        completed,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        FPS,
        output_path,
        Message::AutomationVirtualTick,
        extra_fonts,
    )?;

    Ok(())
}
