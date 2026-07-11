use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced::advanced::graphics::core::renderer::{self, Headless};
use iced::advanced::mouse;
use iced::{Size, Theme};
use iced_test::core::Settings;
use iced_test::core::window;
use iced_test::emulator::{Emulator, Event, Mode};
use iced_test::futures::Subscription;
use iced_test::futures::futures::channel::mpsc;
use iced_test::futures::futures::{FutureExt, StreamExt};
use iced_test::program::Program;
use iced_test::runtime::user_interface;
use iced_test::runtime::{Task, UserInterface};

use crate::app;
use crate::automation::{self, AutomationState, JsonAutomationFlow};
use crate::message::{FolderMessage, Message};
use crate::state::AppState;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
const FPS: u32 = 60;

struct AppProgram {
    demo_root: std::path::PathBuf,
    settings: media_sort_core::settings::store::SettingsStore,
    completed: Arc<AtomicBool>,
    steps: Vec<automation::AutomationStep>,
    flow_name: String,
}

impl Program for AppProgram {
    type State = AppState;
    type Message = Message;
    type Theme = Theme;
    type Renderer = iced::Renderer;
    type Executor = iced_test::futures::backend::default::Executor;

    fn name() -> &'static str {
        "MediaSort"
    }

    fn settings(&self) -> Settings {
        let font = iced::Font::DEFAULT;
        iced_test::core::Settings {
            id: Some("mediasort".into()),
            fonts: vec![],
            default_font: font,
            default_text_size: iced::Pixels(16.0),
            antialiasing: false,
            vsync: false,
        }
    }

    fn window(&self) -> Option<window::Settings> {
        None
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        let mut state = AppState::new(self.settings.clone());

        state.automation = Some(AutomationState::new(
            self.steps.clone(),
            &self.flow_name,
            DEFAULT_WIDTH as f32,
            DEFAULT_HEIGHT as f32,
        ));
        state.demo_root_path = Some(self.demo_root.clone());

        let tasks = vec![
            Task::done(Message::Folder(FolderMessage::Open(self.demo_root.clone()))),
            Task::done(Message::SettingsLoaded(Box::new(Ok(self.settings.clone())))),
        ];

        (state, Task::batch(tasks))
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        let task = app::update(state, message);

        if state.automation.as_ref().is_some_and(|a| a.completed) {
            self.completed.store(true, Ordering::SeqCst);
        }

        task
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        _window: window::Id,
    ) -> iced::Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        app::view(state)
    }

    fn theme(&self, state: &Self::State, _window: window::Id) -> Option<Self::Theme> {
        Some(app::theme(state))
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        app::subscription(state)
    }
}

pub fn export_demo_video(
    demo_root: std::path::PathBuf,
    json_spec_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = media_sort_core::settings::store::SettingsStore::default();

    let completed = Arc::new(AtomicBool::new(false));

    let flow = JsonAutomationFlow::load_from_file(json_spec_path)?;
    let flow_name = flow.flow_name.clone();
    let steps = flow.to_automation_steps(&demo_root);

    let program = AppProgram {
        demo_root,
        settings: settings.clone(),
        completed: completed.clone(),
        steps,
        flow_name,
    };

    let delta = Duration::from_nanos(1_000_000_000 / FPS as u64);
    let size = Size::new(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32);

    // Register the Lucide icon font in the global text subsystem so
    // navigation arrows and other icons render correctly.
    {
        use std::borrow::Cow;
        if let Ok(mut font_system) = iced_wgpu::graphics::text::font_system().write() {
            font_system.load_font(Cow::Borrowed(lucide_icons::LUCIDE_FONT_BYTES));
        }
    }

    let (sender, mut receiver) = mpsc::channel(100);

    let mut emulator: Emulator<AppProgram> = Emulator::new(sender, &program, Mode::Immediate, size);

    // Drain boot events.
    loop {
        let event = iced_test::futures::futures::executor::block_on(receiver.next())
            .expect("emulator stopped");
        match event {
            Event::Action(action) => {
                emulator.perform(&program, action);
            }
            Event::Ready => break,
            Event::Failed(_) => {}
        }
    }

    // We use Emulator for state management (update/perform) and
    // emulator.view() to obtain the element tree.  Frames are rendered
    // with a separate renderer to avoid the iced_test bug where
    // Emulator::screenshot() consumes the cache without restoring it.
    let font = iced::Font::DEFAULT;
    let text_size = iced::Pixels(16.0);
    let mut renderer = <iced::Renderer as Headless>::new(font, text_size, None)
        .now_or_never()
        .flatten()
        .expect("create headless renderer");
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

    let mut frame_count = 0u64;
    if let Some(cell) = crate::automation::VIRTUAL_CURSOR.get() {
        if let Ok(mut guard) = cell.lock() {
            *guard = iced::Point::ORIGIN;
        }
    }
    let _ = crate::automation::VIRTUAL_CURSOR.set(std::sync::Mutex::new(iced::Point::ORIGIN));
    let unpadded_row = DEFAULT_WIDTH as usize * 4;
    let style = renderer::Style {
        text_color: iced::Color::WHITE,
    };

    struct DemoClipboard {
        content: Option<String>,
    }
    impl iced::advanced::Clipboard for DemoClipboard {
        fn read(&self, _kind: iced::advanced::clipboard::Kind) -> Option<String> {
            self.content.clone()
        }
        fn write(&mut self, _kind: iced::advanced::clipboard::Kind, contents: String) {
            self.content = Some(contents);
        }
    }
    let mut clipboard = DemoClipboard { content: None };
    let mut messages = Vec::new();

    while !completed.load(Ordering::SeqCst) {
        emulator.update(&program, Message::AutomationVirtualTick(delta));

        while let Ok(event) = receiver.try_recv() {
            match event {
                Event::Action(action) => {
                    emulator.perform(&program, action);
                }
                Event::Ready => {}
                Event::Failed(_) => {}
            }
        }

        let view = emulator.view(&program);
        let theme = emulator.theme(&program).unwrap_or(Theme::Dark);
        let cursor = crate::automation::VIRTUAL_CURSOR
            .get()
            .and_then(|cell| {
                cell.lock()
                    .ok()
                    .map(|guard| mouse::Cursor::Available(*guard))
            })
            .unwrap_or(mouse::Cursor::Unavailable);

        let mut ui =
            UserInterface::build(view, size, user_interface::Cache::default(), &mut renderer);

        let _ = ui.update(
            &[iced::Event::Window(iced::window::Event::RedrawRequested(
                std::time::Instant::now(),
            ))],
            cursor,
            &mut renderer,
            &mut clipboard,
            &mut messages,
        );
        messages.clear();

        let bg_color = theme.palette().background;
        ui.draw(&mut renderer, &theme, &style, cursor);

        let rgba = renderer.screenshot(Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT), 1.0, bg_color);

        let padded_row = rgba.len().div_ceil(DEFAULT_HEIGHT as usize);
        for row_chunk in rgba.chunks(padded_row) {
            ffmpeg_stdin.write_all(&row_chunk[..unpadded_row])?;
        }

        frame_count += 1;
    }

    while let Ok(event) = receiver.try_recv() {
        if let Event::Action(action) = event {
            emulator.perform(&program, action);
        }
    }

    drop(ffmpeg_stdin);
    ffmpeg.wait()?;

    tracing::info!("Exported {} frames to {}", frame_count, output_path);
    Ok(())
}
