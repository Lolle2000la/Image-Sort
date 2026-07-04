use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use iced::{Color, Size, Theme};

use crate::app;
use crate::automation::{self, AutomationState};
use crate::state::AppState;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const FPS: u32 = 60;

pub fn export_demo_video(
    mut state: AppState,
    mut automation: AutomationState,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = Size::new(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32);
    automation.update_window_size(size.width, size.height);
    let delta = Duration::from_nanos(1_000_000_000 / FPS as u64);

    // ── Headless wgpu ──────────────────────────────────────────────
    let instance = iced_wgpu::wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(
        &iced_wgpu::wgpu::RequestAdapterOptions {
            power_preference: iced_wgpu::wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        },
    ))
    .expect("GPU adapter");

    let (device, queue) =
        pollster::block_on(adapter.request_device(&iced_wgpu::wgpu::DeviceDescriptor {
            label: Some("headless-exporter"),
            required_features: iced_wgpu::wgpu::Features::empty(),
            required_limits: iced_wgpu::wgpu::Limits::default(),
            memory_hints: iced_wgpu::wgpu::MemoryHints::Performance,
            experimental_features: iced_wgpu::wgpu::ExperimentalFeatures::disabled(),
            trace: iced_wgpu::wgpu::Trace::Off,
        }))
        .expect("GPU device");

    let format = iced_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb;
    let engine = iced_wgpu::Engine::new(
        &adapter,
        device,
        queue,
        format,
        None,
        iced_wgpu::graphics::Shell::headless(),
    );

    let mut wgpu_renderer = Some(iced_wgpu::Renderer::new(
        engine,
        iced::Font::DEFAULT,
        iced::Pixels(16.0),
    ));

    let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
        Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
        1.0,
    );

    let theme = Theme::Dark;
    let style = iced::advanced::graphics::core::renderer::Style {
        text_color: Color::WHITE,
    };

    // ── FFmpeg ─────────────────────────────────────────────────────
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &format!("{}x{}", DEFAULT_WIDTH, DEFAULT_HEIGHT),
            "-r",
            &FPS.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            output_path,
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut ffmpeg_stdin = ffmpeg.stdin.take().unwrap();
    let mut cache = iced_runtime::user_interface::Cache::default();
    let mut frame_count = 0u64;

    // ── Render loop ────────────────────────────────────────────────
    while !automation.completed {
        if let Some(result) = automation::handle_automation_virtual_tick(&mut automation, delta) {
            use automation::AutomationTickResult;
            match result {
                AutomationTickResult::Message(msg) => {
                    let _ = app::update(&mut state, msg);
                }
                AutomationTickResult::Task(_) => {
                    // Widget bounds queries can't resolve without the
                    // live iced runtime.  Fire the step's message now
                    // and advance so the script doesn't stall.
                    if automation.script_index < automation.steps.len() {
                        let msg = automation.steps[automation.script_index]
                            .underlying_message
                            .clone();
                        automation.script_index += 1;
                        automation.step_elapsed = std::time::Duration::ZERO;
                        automation.pending_bounds_id = None;
                        if let Some(m) = msg {
                            let _ = app::update(&mut state, m);
                        }
                    }
                }
            }
        }

        let view = app::view(&state);
        let view = automation::wrap_view(view, &automation);

        let inner = wgpu_renderer.take().expect("renderer consumed");
        let mut composite = iced::Renderer::Primary(inner);

        let mut ui = iced_runtime::UserInterface::build(view, size, cache, &mut composite);

        ui.draw(
            &mut composite,
            &theme,
            &style,
            iced::advanced::mouse::Cursor::Unavailable,
        );

        let iced::Renderer::Primary(mut inner) = composite else {
            unreachable!()
        };

        cache = ui.into_cache();

        let frame_data = inner.screenshot(&viewport, Color::from_rgb(0.08, 0.08, 0.1));

        let alignment = 256usize;
        let unpadded_row = DEFAULT_WIDTH as usize * 4;
        let padded_row = unpadded_row.div_ceil(alignment) * alignment;

        for row_chunk in frame_data.chunks(padded_row) {
            ffmpeg_stdin.write_all(&row_chunk[..unpadded_row])?;
        }

        wgpu_renderer = Some(inner);
        frame_count += 1;
    }

    drop(ffmpeg_stdin);
    ffmpeg.wait()?;

    tracing::info!(
        "Exported {} frames ({:.1} s) to {}",
        frame_count,
        automation.virtual_elapsed.as_secs_f32(),
        output_path
    );
    Ok(())
}
