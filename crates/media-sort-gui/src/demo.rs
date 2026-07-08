use std::path::PathBuf;

/// Called at the top of `main()`, before the iced application builder.
/// Returns `Some(Ok(()))` if headless export completed, or `Some(Err(...))`
/// on failure.  Returns `None` when no export env var is set.
pub fn try_headless_export(cli: &crate::Cli) -> Option<Result<(), Box<dyn std::error::Error>>> {
    if !cli.export {
        return None;
    }

    let export_path = std::path::PathBuf::from(&cli.demo_export);
    let spec_path = std::path::PathBuf::from(&cli.demo_spec);

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
                    crate::headless::export_demo_video(
                        demo_root,
                        &path.to_string_lossy(),
                        &output_video_path.to_string_lossy(),
                    )?;
                }
            }
            Ok(())
        } else {
            let demo_root = init_demo_media();
            crate::headless::export_demo_video(demo_root, &cli.demo_spec, &cli.demo_export)
        }
    };

    Some(run())
}

/// Called inside the iced application factory closure.
/// Sets up automation state and returns the demo root as the startup path.
/// Returns `None` when `MEDIA_SORT_DEMO` is not set.
pub fn init(cli: &crate::Cli, state: &mut crate::state::AppState) -> Option<PathBuf> {
    if !cli.demo {
        return None;
    }

    let spec_path = std::path::PathBuf::from(&cli.demo_spec);
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

    if crate::automation::generate_placeholder_media(&demo_root).is_ok() {
        tracing::info!("Demo media generated at {:?}", demo_root);
    }

    let ww = state.settings.window_position.width as f32;
    let wh = state.settings.window_position.height as f32;

    let flow = crate::automation::JsonAutomationFlow::load_from_file(&final_spec_path)
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
