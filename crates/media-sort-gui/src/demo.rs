use std::path::PathBuf;

/// Called at the top of `main()`, before the iced application builder.
/// Returns `Some(Ok(()))` if headless export completed, or `Some(Err(...))`
/// on failure.  Returns `None` when no export env var is set.
pub fn try_headless_export() -> Option<Result<(), Box<dyn std::error::Error>>> {
    let export_path = std::env::var("MEDIA_SORT_DEMO_EXPORT").ok()?;
    let spec_path = std::env::var("MEDIA_SORT_DEMO_SPEC")
        .expect("MEDIA_SORT_DEMO_SPEC required for video export");
    let demo_root = init_demo_media();
    Some(crate::headless::export_demo_video(
        demo_root,
        &spec_path,
        &export_path,
    ))
}

/// Called inside the iced application factory closure.
/// Sets up automation state and returns the demo root as the startup path.
/// Returns `None` when `MEDIA_SORT_DEMO` is not set.
pub fn init(state: &mut crate::state::AppState) -> Option<PathBuf> {
    if std::env::var("MEDIA_SORT_DEMO").is_err() {
        return None;
    }

    let spec_path = std::env::var("MEDIA_SORT_DEMO_SPEC")
        .expect("MEDIA_SORT_DEMO_SPEC env var required when MEDIA_SORT_DEMO is set");

    let demo_root = std::env::temp_dir().join(format!("media_sort_demo_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&demo_root);

    if crate::automation::generate_placeholder_media(&demo_root).is_ok() {
        tracing::info!("Demo media generated at {:?}", demo_root);
    }

    let ww = state.settings.window_position.width as f32;
    let wh = state.settings.window_position.height as f32;

    let flow = crate::automation::JsonAutomationFlow::load_from_file(&spec_path)
        .expect("failed to load demo spec JSON");
    let steps = flow.to_automation_steps(&demo_root);

    state.automation = Some(crate::automation::AutomationState::new(
        steps,
        &flow.flow_name,
        ww,
        wh,
    ));
    state.demo_root_path = Some(demo_root.clone());

    Some(demo_root)
}

fn init_demo_media() -> PathBuf {
    let demo_root = std::env::temp_dir().join(format!("media_sort_demo_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&demo_root);
    if crate::automation::generate_placeholder_media(&demo_root).is_ok() {
        tracing::info!("Demo media generated at {:?}", demo_root);
    }
    demo_root
}
