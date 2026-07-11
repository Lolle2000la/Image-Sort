use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced::advanced::graphics::core::renderer::Headless;
use iced_test::emulator::{Emulator, Event, Mode};
use iced_test::futures::futures::channel::mpsc;
use iced_test::futures::futures::{FutureExt, StreamExt};
use iced_test::program::Program;
use iced_test::runtime::UserInterface;

#[allow(clippy::too_many_arguments)]
pub fn export_video<P>(
    program: &P,
    completed: Arc<AtomicBool>,
    width: u32,
    height: u32,
    fps: u32,
    output_path: &str,
    tick_message: impl Fn(Duration) -> P::Message,
    extra_fonts: Vec<std::borrow::Cow<'static, [u8]>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Program<Renderer = iced::Renderer, Theme = iced::Theme> + 'static,
    P::Message: Clone + std::fmt::Debug + Send + Sync,
{
    let delta = Duration::from_nanos(1_000_000_000 / fps as u64);
    let size = iced::Size::new(width as f32, height as f32);

    for font_bytes in extra_fonts {
        if let Ok(mut font_system) = iced_wgpu::graphics::text::font_system().write() {
            font_system.load_font(font_bytes);
        }
    }

    let (sender, mut receiver) = mpsc::channel(100);

    let mut emulator: Emulator<P> = Emulator::new(sender, program, Mode::Immediate, size);

    // Drain boot events.
    loop {
        let event = iced_test::futures::futures::executor::block_on(receiver.next())
            .expect("emulator stopped");
        match event {
            Event::Action(action) => {
                emulator.perform(program, action);
            }
            Event::Ready => break,
            Event::Failed(_) => {}
        }
    }

    let font = iced::Font::DEFAULT;
    let text_size = iced::Pixels(16.0);
    let mut renderer = <iced::Renderer as Headless>::new(font, text_size, None)
        .now_or_never()
        .flatten()
        .expect("create headless renderer");

    let mut ffmpeg = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &format!("{}x{}", width, height),
            "-r",
            &fps.to_string(),
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
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    let mut ffmpeg_stdin = ffmpeg.stdin.take().unwrap();

    let mut frame_count = 0u64;
    if let Some(cell) = crate::automation::VIRTUAL_CURSOR.get()
        && let Ok(mut guard) = cell.lock()
    {
        *guard = iced::Point::ORIGIN;
    }
    let _ = crate::automation::VIRTUAL_CURSOR.set(std::sync::Mutex::new(iced::Point::ORIGIN));
    let unpadded_row = width as usize * 4;
    let style = iced::advanced::graphics::core::renderer::Style {
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
        emulator.update(program, tick_message(delta));

        while let Ok(event) = receiver.try_recv() {
            if let Event::Action(action) = event {
                emulator.perform(program, action);
            }
        }

        let view = emulator.view(program);
        let theme = emulator.theme(program).unwrap_or(iced::Theme::Dark);
        let cursor = crate::automation::VIRTUAL_CURSOR
            .get()
            .and_then(|cell| {
                cell.lock()
                    .ok()
                    .map(|guard| iced::advanced::mouse::Cursor::Available(*guard))
            })
            .unwrap_or(iced::advanced::mouse::Cursor::Unavailable);

        let mut ui = UserInterface::build(
            view,
            size,
            iced_test::runtime::user_interface::Cache::default(),
            &mut renderer,
        );

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

        let rgba = renderer.screenshot(iced::Size::new(width, height), 1.0, bg_color);

        let padded_row = rgba.len().div_ceil(height as usize);
        for row_chunk in rgba.chunks(padded_row) {
            use std::io::Write;
            ffmpeg_stdin.write_all(&row_chunk[..unpadded_row])?;
        }

        frame_count += 1;
    }

    while let Ok(event) = receiver.try_recv() {
        if let Event::Action(action) = event {
            emulator.perform(program, action);
        }
    }

    drop(ffmpeg_stdin);
    ffmpeg.wait()?;

    tracing::info!("Exported {} frames to {}", frame_count, output_path);
    Ok(())
}
