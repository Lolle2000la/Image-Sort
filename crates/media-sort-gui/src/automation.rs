use std::path::Path;
use std::time::{Duration, Instant};

use serde::Deserialize;

use iced::advanced::mouse;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Operation, Outcome, Scrollable, TextInput};
use iced::widget::{canvas, column, container, stack, text};
use iced::{
    Alignment, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Task, Theme, Vector,
};

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};

// ── FindBounds custom operation ────────────────────────────────────────

struct FindBounds {
    target: Id,
    found: Option<Rectangle>,
}

impl Operation<Option<Rectangle>> for FindBounds {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Rectangle>>)) {
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

    fn finish(&self) -> Outcome<Option<Rectangle>> {
        Outcome::Some(self.found)
    }
}

pub fn find_bounds_task(target: Id) -> Task<Option<Rectangle>> {
    iced::advanced::widget::operate(FindBounds {
        target,
        found: None,
    })
}

// ── Automation target ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AutomationTarget {
    /// Resolve via FindBounds operation — precise runtime coordinates.
    Widget(Id),
    /// Hardcoded pixel position (fallback).
    Pixel(Point),
}

// ── Automation step ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutomationStep {
    pub execution_delay: Duration,
    pub target: AutomationTarget,
    pub underlying_message: Option<Message>,
    pub keycap_label: Option<String>,
}

// ── Automation state ───────────────────────────────────────────────────

pub struct AutomationState {
    pub virtual_cursor: Point,
    pub current_pixel_target: Point,
    pub is_clicking: bool,
    pub active_keycap: Option<(String, Duration)>,
    pub script_index: usize,
    pub steps: Vec<AutomationStep>,
    pub step_timer: Instant,
    /// Total virtual elapsed time (headless mode).
    pub virtual_elapsed: Duration,
    /// Time elapsed since the current step started (headless mode).
    pub step_elapsed: Duration,
    pub window_width: f32,
    pub window_height: f32,
    #[allow(dead_code)]
    pub flow_name: String,
    pub completed: bool,

    /// When `Some`, a FindBounds task is in-flight.  The tick handler
    /// won't advance until `handle_bounds_resolved` is called.
    pub pending_bounds_id: Option<Id>,

    /// Set to true once the current step's delay has elapsed and the
    /// cursor target is resolved.  The action fires when the cursor
    /// has also finished lerping to the target.
    pub step_ready: bool,
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
            pending_bounds_id: None,
            step_ready: false,
            virtual_elapsed: Duration::ZERO,
            step_elapsed: Duration::ZERO,
        }
    }

    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }
}

// ── Tick result ────────────────────────────────────────────────────────

pub enum AutomationTickResult {
    /// Fire the message via `update()`.
    Message(Box<Message>),
    /// Return this task (e.g. a FindBounds query).
    Task(Task<Message>),
}

// ── Tick handlers (real-time and virtual) ──────────────────────────────

/// Real-time tick: computes elapsed wall-clock time and delegates to the
/// virtual tick engine.
pub fn handle_automation_tick(
    automation: &mut AutomationState,
    now: Instant,
) -> Option<AutomationTickResult> {
    // Cap delta at 500 ms to avoid huge jumps if the app was suspended.
    let delta = now
        .duration_since(automation.step_timer)
        .min(Duration::from_millis(500));
    automation.step_timer = now;
    handle_automation_virtual_tick(automation, delta)
}

/// Virtual-time tick: advances the automation by `delta`, animates the
/// cursor toward the current target, then fires the step's action once
/// the cursor arrives.
pub fn handle_automation_virtual_tick(
    automation: &mut AutomationState,
    delta: Duration,
) -> Option<AutomationTickResult> {
    automation.virtual_elapsed += delta;
    automation.step_elapsed += delta;

    // Smooth cursor lerp toward the resolved pixel target.
    let dx = automation.current_pixel_target.x - automation.virtual_cursor.x;
    let dy = automation.current_pixel_target.y - automation.virtual_cursor.y;
    let cursor_arrived = dx.abs() <= 0.5 && dy.abs() <= 0.5;

    if !cursor_arrived {
        automation.virtual_cursor.x += dx * 0.18;
        automation.virtual_cursor.y += dy * 0.18;
        if automation.is_clicking {
            automation.is_clicking = false;
        }
    }

    // Clear keycap label after virtual timeout.
    if let Some((_, set_at)) = &automation.active_keycap
        && automation.virtual_elapsed.saturating_sub(*set_at) > Duration::from_millis(1800)
    {
        automation.active_keycap = None;
    }

    // Don't advance while waiting for bounds resolution.
    if automation.pending_bounds_id.is_some() {
        return None;
    }

    // If the step is ready but the cursor hasn't arrived yet, keep
    // animating and wait.
    if automation.step_ready && !cursor_arrived {
        return None;
    }

    // --- step_ready && cursor_arrived → fire the action ----------------

    if automation.step_ready && cursor_arrived {
        let step = &automation.steps[automation.script_index];
        automation.is_clicking = true;
        if let Some(ref label) = step.keycap_label {
            automation.active_keycap = Some((label.clone(), automation.virtual_elapsed));
        }
        let msg = step.underlying_message.clone();
        automation.script_index += 1;
        automation.step_elapsed = Duration::ZERO;
        automation.step_ready = false;
        automation.step_timer = Instant::now();
        if let Some(m) = msg {
            return Some(AutomationTickResult::Message(Box::new(m)));
        }
        return None;
    }

    // --- resolve the current step's target (delay has elapsed) --------

    if automation.script_index < automation.steps.len() {
        let step = &automation.steps[automation.script_index];
        if automation.step_elapsed >= step.execution_delay {
            match &step.target {
                AutomationTarget::Widget(id) => {
                    automation.pending_bounds_id = Some(id.clone());
                    let task = find_bounds_task(id.clone()).map(Message::AutomationBounds);
                    return Some(AutomationTickResult::Task(task));
                }
                AutomationTarget::Pixel(point) => {
                    automation.current_pixel_target = *point;
                }
            }
            automation.step_elapsed = Duration::ZERO;
            automation.step_ready = true;
            // Don't fire yet — let the cursor animate to the target first.
        }
    } else {
        automation.completed = true;
    }

    None
}

/// Called when a FindBounds query resolves.  Sets the cursor target
/// position and marks the step ready so the next tick fires the action
/// once the cursor arrives.
pub fn handle_bounds_resolved(
    automation: &mut AutomationState,
    rect_opt: Option<Rectangle>,
) -> Option<Message> {
    let _pending = automation.pending_bounds_id.take()?;

    if let Some(rect) = rect_opt {
        automation.current_pixel_target = Point::new(rect.center_x(), rect.center_y());
    }

    automation.step_ready = true;
    None
}

// ── Keycaster ──────────────────────────────────────────────────────────

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

        let p = self.position;
        let pointer_path = canvas::Path::new(|builder| {
            builder.move_to(p);
            builder.line_to(Point::new(p.x + 10.0, p.y + 24.0));
            builder.line_to(Point::new(p.x + 15.0, p.y + 21.0));
            builder.line_to(Point::new(p.x + 23.0, p.y + 34.0));
            builder.line_to(Point::new(p.x + 27.0, p.y + 31.0));
            builder.line_to(Point::new(p.x + 19.0, p.y + 18.0));
            builder.line_to(Point::new(p.x + 28.0, p.y + 14.0));
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
        let hud = container(
            column![
                text("Automation Active")
                    .size(11)
                    .color(Color::from_rgb(0.5, 0.5, 0.6)),
                text(label).size(16).color(Color::from_rgb(0.9, 0.9, 0.95)),
            ]
            .spacing(6),
        )
        .padding([12, 18])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.08, 0.08, 0.1, 0.95,
            ))),
            border: iced::Border {
                color: Color::from_rgb(0.25, 0.45, 0.85),
                width: 1.5,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..container::Style::default()
        });

        let cursor_in_br = automation.virtual_cursor.x > automation.window_width * 0.66
            && automation.virtual_cursor.y > automation.window_height * 0.66;
        let (align_x, align_y) = if cursor_in_br {
            (Alignment::Start, Alignment::End)
        } else {
            (Alignment::End, Alignment::End)
        };

        let positioned = container(hud)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(align_x)
            .align_y(align_y)
            .padding(24);

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

// ── Demo kinds & script generation ─────────────────────────────────────

// ── JSON automation flow (serde deserialization) ───────────────────────

#[derive(Debug, Deserialize)]
pub struct JsonAutomationFlow {
    pub flow_name: String,
    pub steps: Vec<JsonAutomationStep>,
}

#[derive(Debug, Deserialize)]
pub struct JsonAutomationStep {
    pub delay_ms: u64,
    pub target: JsonTarget,
    pub keycap_label: Option<String>,
    pub message: JsonMessage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JsonTarget {
    Coordinate { x: f32, y: f32 },
    Widget { id: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum JsonMessage {
    #[serde(rename = "go_right")]
    GoRight,
    #[serde(rename = "go_left")]
    GoLeft,
    #[serde(rename = "move_active")]
    MoveActive,
    #[serde(rename = "copy_active")]
    CopyActive,
    #[serde(rename = "trigger_rename")]
    TriggerRename,
    #[serde(rename = "open_folder")]
    OpenFolder { relative_path: String },
    #[serde(rename = "select_entry")]
    SelectEntry { index: usize },
    #[serde(rename = "open_settings")]
    OpenSettings,
    #[serde(rename = "toggle_dark_mode")]
    ToggleDarkMode,
    #[serde(rename = "close_settings")]
    CloseSettings,
    #[serde(rename = "search_query")]
    SearchQuery { query: String },
    #[serde(rename = "focus_search")]
    FocusSearch,
    #[serde(rename = "folder_selected")]
    FolderSelected { relative_path: String },
    #[serde(rename = "quit")]
    Quit,
}

impl JsonAutomationFlow {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let flow = serde_json::from_reader(file)?;
        Ok(flow)
    }

    pub fn to_automation_steps(&self, test_root: &Path) -> Vec<AutomationStep> {
        self.steps
            .iter()
            .map(|step| {
                let cursor_target = match &step.target {
                    JsonTarget::Coordinate { x, y } => AutomationTarget::Pixel(Point::new(*x, *y)),
                    JsonTarget::Widget { id } => {
                        let final_id = if test_root.join(id).is_dir() {
                            let folder_path = test_root.join(id);
                            format!("folder_{}", folder_path.display())
                        } else {
                            id.clone()
                        };
                        AutomationTarget::Widget(Id::new(Box::leak(final_id.into_boxed_str())))
                    }
                };

                let underlying_message = match &step.message {
                    JsonMessage::GoRight => Some(Message::Media(MediaMessage::GoRight)),
                    JsonMessage::GoLeft => Some(Message::Media(MediaMessage::GoLeft)),
                    JsonMessage::MoveActive => Some(Message::Media(MediaMessage::MoveActive)),
                    JsonMessage::CopyActive => Some(Message::Media(MediaMessage::CopyActive)),
                    JsonMessage::TriggerRename => Some(Message::Media(MediaMessage::TriggerRename)),
                    JsonMessage::OpenFolder { relative_path } => {
                        let path = test_root.join(relative_path);
                        Some(Message::Folder(FolderMessage::Open(path)))
                    }
                    JsonMessage::SelectEntry { index } => {
                        Some(Message::Media(MediaMessage::SelectEntry(*index)))
                    }
                    JsonMessage::OpenSettings => Some(Message::Settings(SettingsMessage::Open)),
                    JsonMessage::ToggleDarkMode => Some(Message::Settings(
                        SettingsMessage::SetTheme("Dark".to_string()),
                    )),
                    JsonMessage::CloseSettings => Some(Message::Settings(SettingsMessage::Close)),
                    JsonMessage::SearchQuery { query } => Some(Message::Media(
                        MediaMessage::SearchQueryChanged(query.clone()),
                    )),
                    JsonMessage::FocusSearch => Some(Message::Media(MediaMessage::SearchFocused)),
                    JsonMessage::FolderSelected { relative_path } => {
                        let path = test_root.join(relative_path);
                        Some(Message::Folder(FolderMessage::Selected(path, 0)))
                    }
                    JsonMessage::Quit => Some(Message::Quit),
                };

                let keycap_label = step.keycap_label.clone().or_else(|| {
                    underlying_message
                        .as_ref()
                        .map(format_message_for_keycaster)
                });

                AutomationStep {
                    execution_delay: Duration::from_millis(step.delay_ms),
                    target: cursor_target,
                    underlying_message,
                    keycap_label,
                }
            })
            .collect()
    }
}

// ── try_tick hook (decoupled from app.rs) ──────────────────────────────

pub fn try_tick(state: &mut crate::state::AppState, instant: Instant) -> Task<Message> {
    if let Some(ref mut automation) = state.automation
        && let Some(result) = handle_automation_tick(automation, instant)
    {
        match result {
            AutomationTickResult::Message(msg) => {
                return crate::app::update(state, *msg);
            }
            AutomationTickResult::Task(task) => {
                return task;
            }
        }
    }
    Task::none()
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
        let img = create_placeholder_image(r, g, b, 400, 300);
        let path = images_dir.join(format!("{:02}_{}.png", i, name));
        img.save(&path).ok();

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
        let img = create_placeholder_image(r, g, b, 400, 300);
        let path = unsorted_dir.join(format!("{}.png", name));
        img.save(&path).ok();
    }

    Ok(())
}

fn create_placeholder_image(r: u8, g: u8, b: u8, width: u32, height: u32) -> image::RgbaImage {
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

    img
}
