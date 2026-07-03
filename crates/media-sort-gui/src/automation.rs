use std::fmt;
use std::path::Path;
use std::time::{Duration, Instant};

use iced::advanced::mouse;
use iced::widget::{canvas, container, stack, text};
use iced::{Alignment, Color, Element, Length, Point, Rectangle, Renderer, Theme};

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};

// ── Cursor coordinate helpers ──────────────────────────────────────────

/// Build a relative coordinate pair.  `(x, y)` are fractions of the
/// window's width / height (0.0 … 1.0).  At runtime the automation
/// engine scales them to the actual pixel dimensions stored in
/// `AutomationState`.
fn rel(x: f32, y: f32) -> Point {
    Point::new(x, y)
}

// ── Automation step types ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutomationStep {
    pub execution_delay: Duration,
    /// Fractional cursor target (`0.0` – `1.0` of window width/height).
    pub cursor_target: Point,
    pub underlying_message: Option<Message>,
    pub keycap_label: Option<String>,
}

impl AutomationStep {
    pub fn new(
        execution_delay: Duration,
        cursor_target: Point,
        underlying_message: Option<Message>,
    ) -> Self {
        let keycap_label = underlying_message
            .as_ref()
            .map(format_message_for_keycaster);
        Self {
            execution_delay,
            cursor_target,
            underlying_message,
            keycap_label,
        }
    }

    #[allow(dead_code)]
    pub fn with_keycap(
        execution_delay: Duration,
        cursor_target: Point,
        underlying_message: Option<Message>,
        keycap_label: String,
    ) -> Self {
        Self {
            execution_delay,
            cursor_target,
            underlying_message,
            keycap_label: Some(keycap_label),
        }
    }
}

pub struct AutomationState {
    /// Pixel-space cursor position (used for rendering).
    pub virtual_cursor: Point,
    /// Current scaled pixel target.
    pub current_pixel_target: Point,
    pub is_clicking: bool,
    pub active_keycap: Option<(String, Instant)>,
    pub script_index: usize,
    pub steps: Vec<AutomationStep>,
    pub step_timer: Instant,
    pub window_width: f32,
    pub window_height: f32,
    #[allow(dead_code)]
    pub flow_name: String,
    pub completed: bool,
}

impl AutomationState {
    pub fn new(
        steps: Vec<AutomationStep>,
        flow_name: &str,
        window_width: f32,
        window_height: f32,
    ) -> Self {
        Self {
            virtual_cursor: Point::ORIGIN,
            current_pixel_target: Point::ORIGIN,
            is_clicking: false,
            active_keycap: None,
            script_index: 0,
            steps,
            step_timer: Instant::now(),
            window_width,
            window_height,
            flow_name: flow_name.to_string(),
            completed: false,
        }
    }

    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.script_index < self.steps.len() || !self.completed
    }

    fn scale_target(&self, fraction: Point) -> Point {
        Point::new(
            fraction.x * self.window_width,
            fraction.y * self.window_height,
        )
    }
}

pub fn handle_automation_tick(automation: &mut AutomationState, now: Instant) -> Option<Message> {
    let mut pending_message = None;

    // Smooth cursor movement toward the pixel-space target.
    let dx = automation.current_pixel_target.x - automation.virtual_cursor.x;
    let dy = automation.current_pixel_target.y - automation.virtual_cursor.y;

    if dx.abs() > 0.5 || dy.abs() > 0.5 {
        automation.virtual_cursor.x += dx * 0.18;
        automation.virtual_cursor.y += dy * 0.18;
    } else if automation.is_clicking {
        automation.is_clicking = false;
    }

    // Clear keycap label after timeout.
    if let Some((_, clear_timestamp)) = &automation.active_keycap
        && now.duration_since(*clear_timestamp) > Duration::from_millis(1800)
    {
        automation.active_keycap = None;
    }

    // Advance script.
    if automation.script_index < automation.steps.len() {
        let step = &automation.steps[automation.script_index];
        if automation.step_timer.elapsed() >= step.execution_delay {
            automation.current_pixel_target = automation.scale_target(step.cursor_target);
            automation.is_clicking = true;

            if let Some(ref label) = step.keycap_label {
                automation.active_keycap = Some((label.clone(), now));
            }

            pending_message = step.underlying_message.clone();
            automation.script_index += 1;
            automation.step_timer = Instant::now();
        }
    } else {
        automation.completed = true;
    }

    pending_message
}

fn format_message_for_keycaster(msg: &Message) -> String {
    match msg {
        Message::Media(MediaMessage::GoRight) => "Right Arrow\nNext Image".into(),
        Message::Media(MediaMessage::GoLeft) => "Left Arrow\nPrevious Image".into(),
        Message::Media(MediaMessage::MoveActive) => "M\nMove to Folder".into(),
        Message::Media(MediaMessage::CopyActive) => "C\nCopy to Folder".into(),
        Message::Media(MediaMessage::SearchQueryChanged(_)) => "Type Query\nFilter Results".into(),
        Message::Media(MediaMessage::SearchFocused) => "Ctrl+F\nFocus Search".into(),
        Message::Media(MediaMessage::SelectEntry(_)) => "Click\nSelect Entry".into(),
        Message::Folder(FolderMessage::Open(_)) => "Enter\nOpen Folder".into(),
        Message::Folder(FolderMessage::ToggleExpand(_)) => "Space\nExpand Folder".into(),
        Message::Folder(FolderMessage::Selected(_)) => "Arrow Keys\nSelect Destination".into(),
        Message::Settings(SettingsMessage::Open) => "Ctrl+,\nSettings".into(),
        Message::Settings(SettingsMessage::ToggleDarkMode) => "Ctrl+D\nDark Mode".into(),
        Message::Settings(SettingsMessage::Close) => "Esc\nClose".into(),
        Message::Quit => "Ctrl+Q\nQuit".into(),
        _ => "Action".into(),
    }
}

// ── Cursor overlay (canvas) ────────────────────────────────────────────

struct CursorOverlay {
    position: Point,
    is_clicking: bool,
}

impl<Message> canvas::Program<Message, Theme, Renderer> for CursorOverlay {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let cursor_color = if self.is_clicking {
            Color::from_rgb(0.9, 0.2, 0.2)
        } else {
            Color::from_rgb(0.95, 0.95, 0.95)
        };

        let pointer_path = canvas::Path::new(|builder| {
            builder.move_to(self.position);
            builder.line_to(Point::new(self.position.x + 10.0, self.position.y + 24.0));
            builder.line_to(Point::new(self.position.x + 15.0, self.position.y + 21.0));
            builder.line_to(Point::new(self.position.x + 23.0, self.position.y + 34.0));
            builder.line_to(Point::new(self.position.x + 27.0, self.position.y + 31.0));
            builder.line_to(Point::new(self.position.x + 19.0, self.position.y + 18.0));
            builder.line_to(Point::new(self.position.x + 28.0, self.position.y + 14.0));
            builder.close();
        });

        frame.fill(&pointer_path, cursor_color);
        frame.stroke(
            &pointer_path,
            canvas::Stroke::default()
                .with_color(Color::BLACK)
                .with_width(1.5),
        );

        vec![frame.into_geometry()]
    }
}

// ── View wrapper ───────────────────────────────────────────────────────

pub fn wrap_view<'a>(
    base_view: Element<'a, Message>,
    automation: &'a AutomationState,
) -> Element<'a, Message> {
    let mut layers: Vec<Element<'a, Message>> = vec![base_view];

    if automation.completed {
        let completed_label = text("Demo Complete — Close this window")
            .size(22)
            .color(Color::from_rgb(0.85, 0.85, 1.0));
        let completed_box = container(container(completed_label).padding([18, 36]).style(
            |_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.05, 0.05, 0.08, 0.92,
                ))),
                border: iced::Border {
                    color: Color::from_rgb(0.2, 0.65, 0.2),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                ..container::Style::default()
            },
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);
        layers.push(completed_box.into());
    }

    if let Some((ref label, _)) = automation.active_keycap {
        let key_box = container(text(label).size(15).color(Color::from_rgb(0.88, 0.88, 1.0)))
            .padding([8, 14])
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.05, 0.05, 0.07, 0.88,
                ))),
                border: iced::Border {
                    color: Color::from_rgb(0.35, 0.35, 0.38),
                    width: 2.0,
                    radius: 6.0.into(),
                },
                ..container::Style::default()
            });
        let positioned = container(key_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::End)
            .padding([90, 30]);
        layers.push(positioned.into());
    }

    let cursor = canvas(CursorOverlay {
        position: automation.virtual_cursor,
        is_clicking: automation.is_clicking,
    })
    .width(Length::Fill)
    .height(Length::Fill);

    layers.push(cursor.into());
    stack(layers).into()
}

// ── Demo kinds & script generation (parameterised on demo root) ────────

pub enum DemoKind {
    BasicNavigation,
    SortingWorkflow,
    SettingsTour,
    SearchAndFilter,
}

impl fmt::Display for DemoKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DemoKind::BasicNavigation => write!(f, "Basic Navigation"),
            DemoKind::SortingWorkflow => write!(f, "Sorting Workflow"),
            DemoKind::SettingsTour => write!(f, "Settings Tour"),
            DemoKind::SearchAndFilter => write!(f, "Search & Filter"),
        }
    }
}

pub fn generate_demo_script(kind: &DemoKind, demo_root: &Path) -> Vec<AutomationStep> {
    match kind {
        DemoKind::BasicNavigation => basic_navigation_script(demo_root),
        DemoKind::SortingWorkflow => sorting_workflow_script(demo_root),
        DemoKind::SettingsTour => settings_tour_script(demo_root),
        DemoKind::SearchAndFilter => search_and_filter_script(demo_root),
    }
}

// ── Individual demo scripts ────────────────────────────────────────────
//
// Layout assumptions for cursor coordinates (1920×1080 default):
//   top bar         y: 0.00 → 0.08
//   folder tree     x: 0.00 → 0.15
//   media grid      x: 0.15 → 0.85  y: 0.10 → 0.82
//   metadata panel  x: 0.85 → 1.00
//   search bar      y: 0.84 → 0.93
//   bottom bar      y: 0.93 → 1.00

fn basic_navigation_script(root: &Path) -> Vec<AutomationStep> {
    let images_dir = root.join("Images");

    vec![
        // 1. Open the Images folder directly.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.22, 0.15),
            Some(Message::Folder(FolderMessage::Open(images_dir.clone()))),
        ),
        // 2. Select first entry.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.22, 0.42),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        // 3. Go right twice.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.35, 0.42),
            Some(Message::Media(MediaMessage::GoRight)),
        ),
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.48, 0.42),
            Some(Message::Media(MediaMessage::GoRight)),
        ),
        // 4. Go left once.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.42, 0.42),
            Some(Message::Media(MediaMessage::GoLeft)),
        ),
        // 5. Quit.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.99, 0.02),
            Some(Message::Quit),
        ),
    ]
}

fn sorting_workflow_script(root: &Path) -> Vec<AutomationStep> {
    let unsorted_dir = root.join("Unsorted");
    let images_dir = root.join("Images");

    vec![
        // 1. Open the Unsorted folder.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.22, 0.15),
            Some(Message::Folder(FolderMessage::Open(unsorted_dir))),
        ),
        // 2. Select first image in the grid.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.22, 0.42),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        // 3. Select the Images folder as move destination (click folder tree).
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.07, 0.25),
            Some(Message::Folder(FolderMessage::Selected(images_dir))),
        ),
        // 4. Move the selected image.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.68, 0.04),
            Some(Message::Media(MediaMessage::MoveActive)),
        ),
        // 5. Select next image.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.35, 0.42),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        // 6. Move it too.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.68, 0.04),
            Some(Message::Media(MediaMessage::MoveActive)),
        ),
        // 7. Quit.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.99, 0.02),
            Some(Message::Quit),
        ),
    ]
}

fn settings_tour_script(root: &Path) -> Vec<AutomationStep> {
    let images_dir = root.join("Images");

    vec![
        // 1. Open a folder so we see content.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.22, 0.15),
            Some(Message::Folder(FolderMessage::Open(images_dir))),
        ),
        // 2. Open settings (top-right button area).
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.97, 0.04),
            Some(Message::Settings(SettingsMessage::Open)),
        ),
        // 3. Toggle dark mode.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.5, 0.5),
            Some(Message::Settings(SettingsMessage::ToggleDarkMode)),
        ),
        // 4. Toggle back.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.5, 0.5),
            Some(Message::Settings(SettingsMessage::ToggleDarkMode)),
        ),
        // 5. Close settings.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.5, 0.85),
            Some(Message::Settings(SettingsMessage::Close)),
        ),
        // 6. Quit.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.99, 0.02),
            Some(Message::Quit),
        ),
    ]
}

fn search_and_filter_script(root: &Path) -> Vec<AutomationStep> {
    let images_dir = root.join("Images");

    vec![
        // 1. Open the Images folder.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.22, 0.15),
            Some(Message::Folder(FolderMessage::Open(images_dir))),
        ),
        // 2. Focus the search bar.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.45, 0.88),
            Some(Message::Media(MediaMessage::SearchFocused)),
        ),
        // 3. Type a search query to filter.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.45, 0.88),
            Some(Message::Media(MediaMessage::SearchQueryChanged(
                "landscape".into(),
            ))),
        ),
        // 4. Select first filtered result.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.22, 0.42),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        // 5. Clear search.
        AutomationStep::new(
            Duration::from_millis(2000),
            rel(0.45, 0.88),
            Some(Message::Media(MediaMessage::SearchQueryChanged(
                String::new(),
            ))),
        ),
        // 6. Quit.
        AutomationStep::new(
            Duration::from_millis(1500),
            rel(0.99, 0.02),
            Some(Message::Quit),
        ),
    ]
}

// ── Placeholder media generation ───────────────────────────────────────

pub fn generate_placeholder_media(root: &Path) -> std::io::Result<()> {
    let images_dir = root.join("Images");
    let unsorted_dir = root.join("Unsorted");
    std::fs::create_dir_all(&images_dir)?;
    std::fs::create_dir_all(&unsorted_dir)?;

    let colors: &[(u8, u8, u8, &str)] = &[
        (220, 50, 50, "sunset_landscape"),
        (50, 150, 220, "ocean_view"),
        (60, 180, 75, "forest_path"),
        (240, 200, 50, "sunflower_field"),
        (180, 100, 200, "lavender_garden"),
        (255, 140, 0, "autumn_leaves"),
        (100, 200, 180, "mountain_lake"),
        (200, 50, 100, "rose_garden"),
        (80, 80, 200, "night_sky"),
        (50, 180, 150, "tropical_beach"),
    ];

    for (i, &(r, g, b, name)) in colors.iter().enumerate() {
        let img = create_placeholder_image(r, g, b, name, 400, 300);
        let path = images_dir.join(format!("{:02}_{}.png", i, name));
        img.save(&path).ok();

        // Copy every third image into Unsorted.
        if i % 3 == 0 {
            let unsorted_path = unsorted_dir.join(format!("{:02}_{}.png", i, name));
            let _ = std::fs::copy(&path, &unsorted_path);
        }
    }

    let extra_names = &["city_skyline", "desert_dunes", "waterfall", "meadow"];
    for (i, name) in extra_names.iter().enumerate() {
        let (r, g, b) = match i % 4 {
            0 => (100, 100, 100),
            1 => (210, 180, 140),
            2 => (100, 180, 220),
            3 => (150, 220, 100),
            _ => unreachable!(),
        };
        let img = create_placeholder_image(r, g, b, name, 400, 300);
        let path = unsorted_dir.join(format!("{}.png", name));
        img.save(&path).ok();
    }

    Ok(())
}

fn create_placeholder_image(
    r: u8,
    g: u8,
    b: u8,
    label: &str,
    width: u32,
    height: u32,
) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let fx = x as f64 / width as f64;
        let fy = y as f64 / height as f64;

        let shade = fx * 0.5 + fy * 0.5;
        let stripe = ((x as f64 / 20.0).sin() * 0.05) + ((y as f64 / 20.0).cos() * 0.05);

        let rr = (r as f64 * (0.7 + 0.3 * shade) * (1.0 + stripe)).clamp(0.0, 255.0) as u8;
        let gg = (g as f64 * (0.7 + 0.3 * shade) * (1.0 + stripe)).clamp(0.0, 255.0) as u8;
        let bb = (b as f64 * (0.7 + 0.3 * shade) * (1.0 + stripe)).clamp(0.0, 255.0) as u8;

        let border = x < 4 || x >= width - 4 || y < 4 || y >= height - 4;
        if border {
            *pixel = image::Rgba([40, 40, 40, 255]);
        } else {
            *pixel = image::Rgba([rr, gg, bb, 255]);
        }
    }

    draw_label(&mut img, label, (255, 255, 255, 230));
    img
}

fn draw_label(img: &mut image::RgbaImage, label: &str, color: (u8, u8, u8, u8)) {
    let label_bytes = label.as_bytes();
    let char_w: i32 = 7;
    let char_h: i32 = 8;
    let total_w = label_bytes.len() as i32 * char_w;
    let start_x = (img.width() as i32 - total_w) / 2;
    let start_y = (img.height() as i32 - char_h) / 2;

    let char_map: &[(&[u8], &[u8])] = &[
        (
            b"a",
            &[
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            b"b",
            &[
                0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
            ],
        ),
        (
            b"c",
            &[
                0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
            ],
        ),
        (
            b"d",
            &[
                0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
            ],
        ),
        (
            b"e",
            &[
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ],
        ),
        (
            b"f",
            &[
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
        ),
        (
            b"g",
            &[
                0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            b"h",
            &[
                0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            b"i",
            &[
                0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
        ),
        (
            b"j",
            &[
                0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
            ],
        ),
        (
            b"k",
            &[
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ],
        ),
        (
            b"l",
            &[
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ],
        ),
        (
            b"m",
            &[
                0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            b"n",
            &[
                0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            b"o",
            &[
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            b"p",
            &[
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
        ),
        (
            b"q",
            &[
                0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
            ],
        ),
        (
            b"r",
            &[
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ],
        ),
        (
            b"s",
            &[
                0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
        ),
        (
            b"t",
            &[
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
        ),
        (
            b"u",
            &[
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            b"v",
            &[
                0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
            ],
        ),
        (
            b"w",
            &[
                0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
            ],
        ),
        (
            b"x",
            &[
                0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001,
            ],
        ),
        (
            b"y",
            &[
                0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
        ),
        (
            b"z",
            &[
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
            ],
        ),
        (
            b"_",
            &[
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
            ],
        ),
        (
            b"0",
            &[
                0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
            ],
        ),
        (
            b"1",
            &[
                0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
        ),
        (
            b"2",
            &[
                0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
            ],
        ),
        (
            b"3",
            &[
                0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
        ),
        (
            b"4",
            &[
                0b10010, 0b10010, 0b10010, 0b11111, 0b00010, 0b00010, 0b00010,
            ],
        ),
        (
            b"5",
            &[
                0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
            ],
        ),
        (
            b"6",
            &[
                0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            b"7",
            &[
                0b11111, 0b00001, 0b00010, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
        ),
        (
            b"8",
            &[
                0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            b"9",
            &[
                0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
            ],
        ),
        (
            b" ",
            &[
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        ),
    ];

    for (ci, &ch) in label_bytes.iter().enumerate() {
        let cx = start_x + ci as i32 * char_w;
        let lower = ch.to_ascii_lowercase();
        if let Some(&(_, bitmap)) = char_map.iter().find(|(c, _)| c[0] == lower) {
            for (row, &row_bits) in bitmap.iter().enumerate() {
                for col in 0..5 {
                    if (row_bits >> (4 - col)) & 1 == 1 {
                        let px = cx + 1 + col;
                        let py = start_y + row as i32;
                        if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32
                        {
                            img.put_pixel(
                                px as u32,
                                py as u32,
                                image::Rgba([color.0, color.1, color.2, color.3]),
                            );
                        }
                    }
                }
            }
        }
    }
}
