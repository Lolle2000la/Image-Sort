use std::fmt;
use std::path::Path;
use std::time::{Duration, Instant};

use iced::advanced::mouse;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Operation, Outcome, Scrollable, TextInput};
use iced::widget::{canvas, column, container, stack, text};
use iced::{
    Alignment, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Task, Theme, Vector,
};

use crate::message::{FolderMessage, MediaMessage, Message, SettingsMessage};

// ── Widget ID constants shared with view code ───────────────────────────

pub mod widget_ids {
    use iced::widget;

    pub const SEARCH_INPUT: &str = "search_bar";

    pub fn search_input() -> widget::Id {
        widget::Id::new(SEARCH_INPUT)
    }

    pub fn folder_button(path: &std::path::Path) -> widget::Id {
        let s = path.display().to_string();
        widget::Id::new(Box::leak(s.into_boxed_str()))
    }

    pub fn media_card(index: usize) -> widget::Id {
        let s = format!("media_card_{}", index);
        widget::Id::new(Box::leak(s.into_boxed_str()))
    }

    pub fn settings_button() -> widget::Id {
        widget::Id::new("settings_btn")
    }

    pub fn move_button() -> widget::Id {
        widget::Id::new("move_btn")
    }

    pub fn close_settings_button() -> widget::Id {
        widget::Id::new("close_settings_btn")
    }

    pub fn prev_button() -> widget::Id {
        widget::Id::new("prev_btn")
    }

    pub fn next_button() -> widget::Id {
        widget::Id::new("next_btn")
    }

    pub fn dark_mode_toggle() -> widget::Id {
        widget::Id::new("dark_mode_toggle")
    }
}

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

impl AutomationTarget {
    #[allow(dead_code)]
    pub fn widget(id: Id) -> Self {
        AutomationTarget::Widget(id)
    }

    #[allow(dead_code)]
    pub fn pixel(x: f32, y: f32) -> Self {
        AutomationTarget::Pixel(Point::new(x, y))
    }
}

// ── Automation step ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutomationStep {
    pub execution_delay: Duration,
    pub target: AutomationTarget,
    pub underlying_message: Option<Message>,
    pub keycap_label: Option<String>,
}

impl AutomationStep {
    pub fn new(
        execution_delay: Duration,
        target: AutomationTarget,
        underlying_message: Option<Message>,
    ) -> Self {
        let keycap_label = underlying_message
            .as_ref()
            .map(format_message_for_keycaster);
        Self {
            execution_delay,
            target,
            underlying_message,
            keycap_label,
        }
    }
}

// ── Automation state ───────────────────────────────────────────────────

pub struct AutomationState {
    pub virtual_cursor: Point,
    pub current_pixel_target: Point,
    pub is_clicking: bool,
    pub active_keycap: Option<(String, Instant)>,
    pub script_index: usize,
    pub steps: Vec<AutomationStep>,
    pub step_timer: Instant,
    pub window_width: f32,
    pub window_height: f32,
    pub folder_tree_width: f32,
    pub metadata_panel_width: f32,
    pub metadata_expanded: bool,
    #[allow(dead_code)]
    pub flow_name: String,
    pub completed: bool,

    /// When `Some`, a FindBounds task is in-flight.  The tick handler
    /// won't advance until `handle_bounds_resolved` is called.
    pub pending_bounds_id: Option<Id>,
}

impl AutomationState {
    pub fn new(
        steps: Vec<AutomationStep>,
        flow_name: &str,
        window_width: f32,
        window_height: f32,
        folder_tree_width: u16,
        metadata_panel_width: u16,
        metadata_expanded: bool,
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
            folder_tree_width: folder_tree_width as f32,
            metadata_panel_width: metadata_panel_width as f32,
            metadata_expanded,
            flow_name: flow_name.to_string(),
            completed: false,
            pending_bounds_id: None,
        }
    }

    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }

    #[allow(dead_code)]
    pub fn update_layout(
        &mut self,
        folder_tree_width: u16,
        metadata_panel_width: u16,
        metadata_expanded: bool,
    ) {
        self.folder_tree_width = folder_tree_width as f32;
        self.metadata_panel_width = metadata_panel_width as f32;
        self.metadata_expanded = metadata_expanded;
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.script_index < self.steps.len() || !self.completed
    }
}

// ── Tick result ────────────────────────────────────────────────────────

pub enum AutomationTickResult {
    /// Fire the message via `update()`.
    Message(Message),
    /// Return this task (e.g. a FindBounds query).
    Task(Task<Message>),
}

// ── Tick handler ───────────────────────────────────────────────────────

pub fn handle_automation_tick(
    automation: &mut AutomationState,
    now: Instant,
) -> Option<AutomationTickResult> {
    let dx = automation.current_pixel_target.x - automation.virtual_cursor.x;
    let dy = automation.current_pixel_target.y - automation.virtual_cursor.y;

    if dx.abs() > 0.5 || dy.abs() > 0.5 {
        automation.virtual_cursor.x += dx * 0.18;
        automation.virtual_cursor.y += dy * 0.18;
    } else if automation.is_clicking {
        automation.is_clicking = false;
    }

    if let Some((_, clear_timestamp)) = &automation.active_keycap
        && now.duration_since(*clear_timestamp) > Duration::from_millis(1800)
    {
        automation.active_keycap = None;
    }

    // Don't advance while waiting for bounds.
    if automation.pending_bounds_id.is_some() {
        return None;
    }

    if automation.script_index < automation.steps.len() {
        let step = &automation.steps[automation.script_index];
        if automation.step_timer.elapsed() >= step.execution_delay {
            match &step.target {
                AutomationTarget::Widget(id) => {
                    automation.pending_bounds_id = Some(id.clone());
                    automation.step_timer = Instant::now();
                    let task = find_bounds_task(id.clone()).map(Message::AutomationBounds);
                    return Some(AutomationTickResult::Task(task));
                }
                AutomationTarget::Pixel(point) => {
                    automation.current_pixel_target = *point;
                }
            }

            automation.is_clicking = true;

            if let Some(ref label) = step.keycap_label {
                automation.active_keycap = Some((label.clone(), now));
            }

            let msg = step.underlying_message.clone();
            automation.script_index += 1;
            automation.step_timer = Instant::now();

            if let Some(m) = msg {
                return Some(AutomationTickResult::Message(m));
            }
        }
    } else {
        automation.completed = true;
    }

    None
}

/// Called when a FindBounds query resolves.
pub fn handle_bounds_resolved(
    automation: &mut AutomationState,
    rect_opt: Option<Rectangle>,
) -> Option<Message> {
    let _pending = automation.pending_bounds_id.take()?;

    if let Some(rect) = rect_opt {
        automation.current_pixel_target = Point::new(rect.center_x(), rect.center_y());
    }

    // Fire the current step's message and advance.
    if automation.script_index < automation.steps.len() {
        let step = &automation.steps[automation.script_index];
        let msg = step.underlying_message.clone();
        if let Some(ref label) = step.keycap_label {
            automation.active_keycap = Some((label.clone(), Instant::now()));
        }
        automation.is_clicking = true;
        automation.script_index += 1;
        automation.step_timer = Instant::now();
        return msg;
    }

    None
}

// ── Keycaster ──────────────────────────────────────────────────────────

fn format_message_for_keycaster(msg: &Message) -> String {
    match msg {
        Message::Media(MediaMessage::GoRight) => "Right Arrow\nNext Image".into(),
        Message::Media(MediaMessage::GoLeft) => "Left Arrow\nPrevious Image".into(),
        Message::Media(MediaMessage::MoveActive) => "M\nMove to Folder".into(),
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

fn step_widget(delay_ms: u64, id: Id, message: Option<Message>) -> AutomationStep {
    AutomationStep::new(
        Duration::from_millis(delay_ms),
        AutomationTarget::Widget(id),
        message,
    )
}

// ── Individual demo scripts ────────────────────────────────────────────

fn basic_navigation_script(root: &Path) -> Vec<AutomationStep> {
    let images_dir = root.join("Images");
    vec![
        step_widget(
            1500,
            widget_ids::folder_button(&images_dir),
            Some(Message::Folder(FolderMessage::Open(images_dir.clone()))),
        ),
        step_widget(
            2500,
            widget_ids::media_card(0),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        step_widget(
            1500,
            widget_ids::next_button(),
            Some(Message::Media(MediaMessage::GoRight)),
        ),
        step_widget(
            1500,
            widget_ids::next_button(),
            Some(Message::Media(MediaMessage::GoRight)),
        ),
        step_widget(
            1500,
            widget_ids::prev_button(),
            Some(Message::Media(MediaMessage::GoLeft)),
        ),
        step_widget(
            1500,
            widget_ids::close_settings_button(),
            Some(Message::Quit),
        ),
    ]
}

fn sorting_workflow_script(root: &Path) -> Vec<AutomationStep> {
    let unsorted_dir = root.join("Unsorted");
    let images_dir = root.join("Images");
    vec![
        step_widget(
            1500,
            widget_ids::folder_button(&unsorted_dir),
            Some(Message::Folder(FolderMessage::Open(unsorted_dir))),
        ),
        step_widget(
            2500,
            widget_ids::media_card(0),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        step_widget(
            1500,
            widget_ids::folder_button(&images_dir),
            Some(Message::Folder(FolderMessage::Selected(images_dir))),
        ),
        step_widget(
            1500,
            widget_ids::move_button(),
            Some(Message::Media(MediaMessage::MoveActive)),
        ),
        step_widget(
            2000,
            widget_ids::media_card(0),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        step_widget(
            1500,
            widget_ids::move_button(),
            Some(Message::Media(MediaMessage::MoveActive)),
        ),
        step_widget(
            2000,
            widget_ids::close_settings_button(),
            Some(Message::Quit),
        ),
    ]
}

fn settings_tour_script(root: &Path) -> Vec<AutomationStep> {
    let images_dir = root.join("Images");
    vec![
        step_widget(
            1500,
            widget_ids::folder_button(&images_dir),
            Some(Message::Folder(FolderMessage::Open(images_dir))),
        ),
        step_widget(
            1500,
            widget_ids::settings_button(),
            Some(Message::Settings(SettingsMessage::Open)),
        ),
        step_widget(
            2000,
            widget_ids::dark_mode_toggle(),
            Some(Message::Settings(SettingsMessage::ToggleDarkMode)),
        ),
        step_widget(
            2000,
            widget_ids::dark_mode_toggle(),
            Some(Message::Settings(SettingsMessage::ToggleDarkMode)),
        ),
        step_widget(
            1500,
            widget_ids::close_settings_button(),
            Some(Message::Settings(SettingsMessage::Close)),
        ),
        step_widget(
            1500,
            widget_ids::close_settings_button(),
            Some(Message::Quit),
        ),
    ]
}

fn search_and_filter_script(root: &Path) -> Vec<AutomationStep> {
    let images_dir = root.join("Images");
    vec![
        step_widget(
            1500,
            widget_ids::folder_button(&images_dir),
            Some(Message::Folder(FolderMessage::Open(images_dir))),
        ),
        step_widget(
            2000,
            widget_ids::search_input(),
            Some(Message::Media(MediaMessage::SearchFocused)),
        ),
        step_widget(
            1500,
            widget_ids::search_input(),
            Some(Message::Media(MediaMessage::SearchQueryChanged(
                "landscape".into(),
            ))),
        ),
        step_widget(
            2000,
            widget_ids::media_card(0),
            Some(Message::Media(MediaMessage::SelectEntry(0))),
        ),
        step_widget(
            2000,
            widget_ids::search_input(),
            Some(Message::Media(MediaMessage::SearchQueryChanged(
                String::new(),
            ))),
        ),
        step_widget(
            1500,
            widget_ids::close_settings_button(),
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
