use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Operation, Scrollable, TextInput};
use iced::{Color, Point, Rectangle, Size, Theme, Vector};

use crate::app;
use crate::automation::{self, AutomationState};
use crate::message::{FolderMessage, MediaMessage, Message};
use crate::state::AppState;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const FPS: u32 = 60;

// ── Synchronous bounds lookup ──────────────────────────────────────────

struct HeadlessFindBounds {
    target: Id,
    found: Option<Rectangle>,
}

impl Operation for HeadlessFindBounds {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&self.target) {
            self.found = Some(bounds);
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if id == Some(&self.target) {
            self.found = Some(bounds);
        }
    }

    fn text_input(&mut self, id: Option<&Id>, bounds: Rectangle, _state: &mut dyn TextInput) {
        if id == Some(&self.target) {
            self.found = Some(bounds);
        }
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, _text: &str) {
        if id == Some(&self.target) {
            self.found = Some(bounds);
        }
    }
}

// ── Synchronous image loading (avoids async Task dependency) ───────────

fn load_selected_image(state: &mut AppState) {
    let Some(idx) = state.selected_index else {
        return;
    };
    let filtered = state.filtered_media_entries();
    let Some(entry) = filtered.get(idx) else {
        return;
    };
    let path = entry.path.clone();
    if let Ok(img) = media_sort_backend::media::image_decoder::load_image(&path) {
        use image::GenericImageView;
        let (w, h) = img.dimensions();
        let rgba = img.to_rgba8().into_raw();
        let handle = iced::widget::image::Handle::from_rgba(w, h, rgba);
        state.selected_image = Some((path.clone(), handle.clone()));
        state.image_cache.push(path, handle);
    }
}

fn sync_load_first_entry(state: &mut AppState) {
    if state.media_entries.is_empty() {
        return;
    }
    let path = state.media_entries[0].path.clone();
    if let Ok(img) = media_sort_backend::media::image_decoder::load_image(&path) {
        use image::GenericImageView;
        let (w, h) = img.dimensions();
        let rgba = img.to_rgba8().into_raw();
        let handle = iced::widget::image::Handle::from_rgba(w, h, rgba);
        state.image_cache.push(path, handle);
    }
}

// ── Video export ───────────────────────────────────────────────────────

pub fn export_demo_video(
    mut state: AppState,
    mut automation: AutomationState,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let size = Size::new(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32);
    automation.update_window_size(size.width, size.height);
    let delta = Duration::from_nanos(1_000_000_000 / FPS as u64);

    // Register the Lucide icon font so navigation arrows and other
    // icons render correctly in the headless text subsystem.
    {
        use std::borrow::Cow;
        if let Ok(mut font_system) = iced_wgpu::graphics::text::font_system().write() {
            font_system.load_font(Cow::Borrowed(lucide_icons::LUCIDE_FONT_BYTES));
        }
    }

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
        // 1. Step automation in virtual time.
        let mut msg_to_process = None;
        let mut needs_bounds_resolution = false;

        if let Some(result) = automation::handle_automation_virtual_tick(&mut automation, delta) {
            use automation::AutomationTickResult;
            match result {
                AutomationTickResult::Message(msg) => {
                    msg_to_process = Some(msg);
                }
                AutomationTickResult::Task(_) => {
                    needs_bounds_resolution = true;
                }
            }
        }

        // 2. Resolve bounds and extract the target rect.
        let found_rect: Option<Rectangle> = if needs_bounds_resolution {
            let view = app::view(&state);
            let view = automation::wrap_view(view, &automation);
            let inner = wgpu_renderer.take().expect("renderer consumed");
            let mut composite = iced::Renderer::Primary(inner);

            let mut ui = iced_runtime::UserInterface::build(view, size, cache, &mut composite);

            let rect = if let Some(ref target_id) = automation.pending_bounds_id {
                let mut op = HeadlessFindBounds {
                    target: target_id.clone(),
                    found: None,
                };
                ui.operate(&composite, &mut op);
                op.found
            } else {
                None
            };

            cache = ui.into_cache();
            let iced::Renderer::Primary(inner) = composite else {
                unreachable!()
            };
            wgpu_renderer = Some(inner);
            rect
        } else {
            None
        };
        // ── view/UI dropped — safe to mutate automation now ──

        if let Some(rect) = found_rect {
            automation.current_pixel_target = Point::new(rect.center_x(), rect.center_y());
        }

        if needs_bounds_resolution {
            automation.pending_bounds_id = None;

            if automation.script_index < automation.steps.len() {
                let step = &automation.steps[automation.script_index];
                if let Some(ref label) = step.keycap_label {
                    automation.active_keycap = Some((label.clone(), automation.virtual_elapsed));
                }
                automation.is_clicking = true;
                automation.script_index += 1;
                automation.step_elapsed = Duration::ZERO;
                msg_to_process = step.underlying_message.clone();
            }
        }

        // 3. Process message side effects (async tasks simulated).
        if let Some(msg) = msg_to_process {
            let _ = app::update(&mut state, msg.clone());

            match &msg {
                Message::Folder(FolderMessage::Open(_)) => {
                    sync_load_first_entry(&mut state);
                    state.selected_index = Some(0);
                    load_selected_image(&mut state);
                }
                Message::Media(
                    MediaMessage::GoRight | MediaMessage::GoLeft | MediaMessage::SelectEntry(_),
                ) => {
                    load_selected_image(&mut state);
                }
                Message::Media(MediaMessage::MoveActive) => {
                    state.selected_index = Some(0);
                    load_selected_image(&mut state);
                }
                _ => {}
            }
        }

        // Synchronously generate thumbnails for visible entries (the
        // normal async Tasks can't run without the iced runtime).
        {
            let paths: Vec<_> = state
                .filtered_media_entries()
                .iter()
                .filter(|e| !state.thumbnail_cache.contains(&e.path))
                .map(|e| e.path.clone())
                .collect();
            for path in paths {
                if let Ok(bytes) = crate::subscriptions::prefetch::generate_thumbnail(&path)
                    && !bytes.is_empty()
                {
                    let handle = iced::widget::image::Handle::from_bytes(bytes);
                    state.thumbnail_cache.push(path, handle);
                }
            }
        }

        // 4. Build view and render frame.
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
