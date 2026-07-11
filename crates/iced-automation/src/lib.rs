pub mod automation;
pub mod headless;

pub use automation::{
    AutomationMessage, AutomationMessageView, AutomationState, AutomationStateTrait,
    AutomationStep, AutomationTarget, AutomationTickResult, JsonAutomationFlow, JsonAutomationStep,
    JsonTarget, VIRTUAL_CURSOR, build_automation_steps, find_bounds_task, handle_automation_tick,
    handle_automation_virtual_tick, handle_bounds_resolved, intercept_update, try_tick,
    try_tick_state, wrap_view,
};
pub use headless::export_video;

pub use iced_automation_macros::{message, state};
