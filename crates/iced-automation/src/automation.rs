use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Deserialize;

use iced::advanced::mouse;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Operation, Outcome, Scrollable, TextInput};
use iced::widget::{canvas, column, container, stack, text};
use iced::{
    Alignment, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Task, Theme, Vector,
};

// --- FindBounds custom operation ---
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

// --- Automation target ---
#[derive(Debug, Clone)]
pub enum AutomationTarget {
    Widget(Id),
    Pixel(Point),
}

// --- Automation step ---
#[derive(Debug, Clone)]
pub struct AutomationStep<Message> {
    pub execution_delay: Duration,
    pub target: AutomationTarget,
    pub underlying_message: Option<Message>,
    pub keycap_label: Option<String>,
}

// --- Automation state ---
pub struct AutomationState<Message> {
    pub virtual_cursor: Point,
    pub current_pixel_target: Point,
    pub is_clicking: bool,
    pub active_keycap: Option<(String, Duration)>,
    pub script_index: usize,
    pub steps: Vec<AutomationStep<Message>>,
    pub step_timer: Instant,
    pub virtual_elapsed: Duration,
    pub step_elapsed: Duration,
    pub window_width: f32,
    pub window_height: f32,
    pub flow_name: String,
    pub completed: bool,
    pub pending_bounds_id: Option<Id>,
    pub step_ready: bool,
}

pub static VIRTUAL_CURSOR: OnceLock<Mutex<Point>> = OnceLock::new();

impl<Message> AutomationState<Message> {
    pub fn new(
        steps: Vec<AutomationStep<Message>>,
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

// --- Tick result ---
pub enum AutomationTickResult<Message> {
    Message(Message),
    FindBounds(Id),
}

// --- Tick handlers ---
pub fn handle_automation_tick<Message>(
    automation: &mut AutomationState<Message>,
    now: Instant,
) -> Option<AutomationTickResult<Message>>
where
    Message: Clone,
{
    let delta = now
        .duration_since(automation.step_timer)
        .min(Duration::from_millis(500));
    automation.step_timer = now;
    handle_automation_virtual_tick(automation, delta)
}

pub fn handle_automation_virtual_tick<Message>(
    automation: &mut AutomationState<Message>,
    delta: Duration,
) -> Option<AutomationTickResult<Message>>
where
    Message: Clone,
{
    automation.virtual_elapsed += delta;
    automation.step_elapsed += delta;

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

    if let Some((_, set_at)) = &automation.active_keycap
        && automation.virtual_elapsed.saturating_sub(*set_at) > Duration::from_millis(1800)
    {
        automation.active_keycap = None;
    }

    if automation.pending_bounds_id.is_some() {
        return None;
    }

    if automation.step_ready && !cursor_arrived {
        return None;
    }

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
            return Some(AutomationTickResult::Message(m));
        }
        return None;
    }

    if automation.script_index < automation.steps.len() {
        let step = &automation.steps[automation.script_index];
        if automation.step_elapsed >= step.execution_delay {
            match &step.target {
                AutomationTarget::Widget(id) => {
                    automation.pending_bounds_id = Some(id.clone());
                    return Some(AutomationTickResult::FindBounds(id.clone()));
                }
                AutomationTarget::Pixel(point) => {
                    automation.current_pixel_target = *point;
                }
            }
            automation.step_elapsed = Duration::ZERO;
            automation.step_ready = true;
        }
    } else {
        automation.completed = true;
    }

    None
}

pub fn handle_bounds_resolved<Message>(
    automation: &mut AutomationState<Message>,
    rect_opt: Option<Rectangle>,
) {
    automation.pending_bounds_id = None;

    if let Some(rect) = rect_opt {
        automation.current_pixel_target = Point::new(rect.center_x(), rect.center_y());
    }

    automation.step_ready = true;
}

// --- Cursor overlay (canvas) ---
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

// --- View wrapper ---
pub fn wrap_view<'a, Message>(
    base_view: Element<'a, Message, Theme, Renderer>,
    automation: &'a AutomationState<Message>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
{
    let mut layers: Vec<Element<'a, Message, Theme, Renderer>> = vec![base_view];

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

// --- JSON automation flow ---
#[derive(Debug, Deserialize, Clone)]
pub struct JsonAutomationFlow<Msg = serde_json::Value> {
    pub flow_name: String,
    pub steps: Vec<JsonAutomationStep<Msg>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonAutomationStep<Msg = serde_json::Value> {
    pub delay_ms: u64,
    pub target: JsonTarget,
    pub keycap_label: Option<String>,
    pub message: Msg,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum JsonTarget {
    Coordinate { x: f32, y: f32 },
    Widget { id: String },
}

impl<Msg> JsonAutomationFlow<Msg>
where
    Msg: serde::de::DeserializeOwned,
{
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let flow = serde_json::from_reader(file)?;
        Ok(flow)
    }

    pub fn to_automation_steps<Message, MapTarget, MapMessage, MapKeycap>(
        &self,
        mut map_target: MapTarget,
        mut map_message: MapMessage,
        mut map_keycap: MapKeycap,
    ) -> Vec<AutomationStep<Message>>
    where
        MapTarget: FnMut(&JsonTarget) -> AutomationTarget,
        MapMessage: FnMut(&Msg) -> Option<Message>,
        MapKeycap: FnMut(&Msg, &Option<Message>) -> Option<String>,
    {
        self.steps
            .iter()
            .map(|step| {
                let target = map_target(&step.target);
                let underlying_message = map_message(&step.message);
                let keycap_label = step
                    .keycap_label
                    .clone()
                    .or_else(|| map_keycap(&step.message, &underlying_message));

                AutomationStep {
                    execution_delay: Duration::from_millis(step.delay_ms),
                    target,
                    underlying_message,
                    keycap_label,
                }
            })
            .collect()
    }
}

pub fn try_tick<State, Message, Update, MapBounds>(
    automation_opt: &mut Option<AutomationState<Message>>,
    state: &mut State,
    instant: Instant,
    mut update: Update,
    map_bounds: MapBounds,
) -> Task<Message>
where
    Message: Clone + Send + 'static,
    Update: FnMut(&mut State, Message) -> Task<Message>,
    MapBounds: FnMut(Option<Rectangle>) -> Message + Send + 'static,
{
    if let Some(automation) = automation_opt
        && let Some(result) = handle_automation_tick(automation, instant)
    {
        match result {
            AutomationTickResult::Message(msg) => {
                return update(state, msg);
            }
            AutomationTickResult::FindBounds(id) => {
                return find_bounds_task(id).map(map_bounds);
            }
        }
    }
    Task::none()
}
pub fn build_automation_steps<Message>(
    flow: &JsonAutomationFlow<Message>,
    mut resolve_widget_id: impl FnMut(&str) -> String,
    mut format_keycap: impl FnMut(&Message) -> String,
) -> Vec<AutomationStep<Message>>
where
    Message: Clone + serde::de::DeserializeOwned,
{
    flow.to_automation_steps(
        |target| match target {
            JsonTarget::Coordinate { x, y } => AutomationTarget::Pixel(iced::Point::new(*x, *y)),
            JsonTarget::Widget { id } => {
                let resolved_id = resolve_widget_id(id.as_str());
                AutomationTarget::Widget(iced::advanced::widget::Id::new(Box::leak(
                    resolved_id.into_boxed_str(),
                )))
            }
        },
        |msg| Some(msg.clone()),
        |_, underlying_message| underlying_message.as_ref().map(&mut format_keycap),
    )
}
