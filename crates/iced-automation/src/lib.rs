pub mod automation;
pub mod demo_setup;

#[cfg(feature = "headless")]
pub mod headless;

pub use automation::{
    AutomationContext, AutomationMessage, AutomationState, AutomationStateTrait, AutomationStep,
    AutomationStyle, AutomationTarget, AutomationTickResult, JsonAutomationFlow,
    JsonAutomationStep, JsonTarget, VIRTUAL_CURSOR, build_automation_steps, find_bounds_task,
    handle_automation_message, handle_automation_tick, handle_automation_virtual_tick,
    handle_bounds_resolved, try_tick, try_tick_state, wrap_view,
};

pub use demo_setup::{DemoApp, DemoBootstrap, DemoConfig, init_demo};

#[cfg(feature = "headless")]
pub use headless::{HeadlessApp, export_video};

pub use iced_automation_macros::{message, state};
