use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::automation::{
    AutomationState, AutomationStateTrait, AutomationStyle, JsonAutomationFlow,
    build_automation_steps,
};

/// Describes how demo fixture files are staged.
pub struct FixtureSpec {
    /// Source directory containing fixture files to copy.
    pub source: PathBuf,
    /// Target directory. If `None`, a unique temp directory is created automatically.
    pub target: Option<PathBuf>,
}

/// Configuration for demo initialization.
pub struct DemoConfig {
    pub spec_path: PathBuf,
    /// If set, fixture files are copied from `source` to `target` (or a temp dir).
    /// The resulting directory is used as `$DEMO_ROOT` in the flow spec.
    pub fixture: Option<FixtureSpec>,
    pub variable_substitutions: HashMap<String, String>,
    pub window_width: f32,
    pub window_height: f32,
    pub style: AutomationStyle,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            spec_path: PathBuf::new(),
            fixture: None,
            variable_substitutions: HashMap::new(),
            window_width: 1920.0,
            window_height: 1080.0,
            style: AutomationStyle::default(),
        }
    }
}

/// Result of demo initialization: state ready to use, startup task, and demo media root.
pub struct DemoBootstrap<S, M> {
    pub state: S,
    pub task: iced::Task<M>,
    pub demo_root: PathBuf,
}

#[cfg(feature = "headless")]
pub struct ExportVideoConfig<Message> {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub output_path: PathBuf,
    pub tick_message: fn(std::time::Duration) -> Message,
    pub extra_fonts: Vec<std::borrow::Cow<'static, [u8]>>,
}

#[cfg(feature = "headless")]
impl<Message> ExportVideoConfig<Message> {
    pub fn standard(output_path: PathBuf) -> Self
    where
        Message: crate::automation::VirtualTickMessage,
    {
        Self {
            width: 1920,
            height: 1080,
            fps: 60,
            output_path,
            tick_message: Message::virtual_tick,
            extra_fonts: Vec::new(),
        }
    }
}

#[cfg(feature = "headless")]
impl<S, M> DemoBootstrap<S, M> {
    /// Convert this bootstrap into a `HeadlessApp` for video export.
    pub fn into_headless_app<Update, ThemeFn, Sub>(
        self,
        settings: iced_test::core::Settings,
        update: Update,
        view: impl for<'a> Fn(
            &'a S,
            iced::window::Id,
        ) -> iced::Element<'a, M, iced::Theme, iced::Renderer>
        + 'static,
        theme_fn: ThemeFn,
        subscription: Sub,
    ) -> crate::headless::HeadlessApp<S, M, Update, ThemeFn, Sub> {
        crate::headless::HeadlessApp::new(
            crate::headless::HeadlessBootConfig {
                settings,
                boot_state: self.state,
                boot_task: self.task,
            },
            update,
            view,
            theme_fn,
            subscription,
        )
    }
}

/// Trait that consumers implement to use `init_demo()`.
pub trait DemoApp: AutomationStateTrait<Self::Message> + Sized {
    type Message: Clone + Send + DeserializeOwned + 'static;
    type Settings: Default;

    /// Create a fresh application state from settings.
    fn new_app_state(settings: &Self::Settings) -> Self;

    /// Default settings for the application.
    fn default_settings() -> Self::Settings;

    /// Resolve a widget ID from the JSON spec into the app's widget ID convention.
    /// Called for each `Widget` target in the JSON flow. `fixture_root` is the
    /// directory where demo media files are staged.
    ///
    /// Default implementation passes through the raw JSON id unchanged.
    fn resolve_widget_id(_fixture_root: &Path, json_id: &str) -> String {
        json_id.to_string()
    }

    /// Human-readable keycap label for a message.
    fn format_keycap(message: &Self::Message) -> String;

    /// Startup messages to fire when the demo begins.
    fn bootstrap_messages(settings: &Self::Settings, demo_root: &Path) -> Vec<Self::Message>;
}

/// Initialise a demo application from a JSON flow spec and optional fixture directory.
///
/// Reads the spec file, substitutes variables, parses the JSON flow, copies
/// fixture files to a temp directory, builds automation steps, creates the
/// application state, and returns a `DemoBootstrap` ready to send to the application.
pub fn init_demo<A: DemoApp>(
    config: &DemoConfig,
) -> Result<DemoBootstrap<A, A::Message>, Box<dyn std::error::Error>> {
    let demo_root = match &config.fixture {
        Some(fixture) => {
            let target = fixture.target.clone().unwrap_or_else(|| {
                std::env::temp_dir().join(format!("demo_{}", std::process::id()))
            });
            std::fs::create_dir_all(&target)?;
            copy_dir_all(&fixture.source, &target)?;
            target
        }
        None => PathBuf::new(),
    };

    let mut spec_content = std::fs::read_to_string(&config.spec_path)?;
    spec_content = spec_content.replace("$DEMO_ROOT", &demo_root.to_string_lossy());
    for (var, val) in &config.variable_substitutions {
        spec_content = spec_content.replace(&format!("${}", var), val);
    }

    let flow: JsonAutomationFlow<A::Message> = serde_json::from_str(&spec_content)?;

    let fixture_root = demo_root.clone();
    let steps = build_automation_steps(
        &flow,
        |id| A::resolve_widget_id(&fixture_root, id),
        A::format_keycap,
    );

    let settings = A::default_settings();
    let mut state = A::new_app_state(&settings);

    *state.automation_mut() = Some(AutomationState::new(
        steps,
        &flow.flow_name,
        config.window_width,
        config.window_height,
        config.style.clone(),
    ));

    let messages = A::bootstrap_messages(&settings, &demo_root);
    let task = messages.into_iter().fold(iced::Task::none(), |acc, msg| {
        let t: iced::Task<A::Message> = iced::Task::done(msg);
        acc.chain(t)
    });

    Ok(DemoBootstrap {
        state,
        task,
        demo_root,
    })
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
