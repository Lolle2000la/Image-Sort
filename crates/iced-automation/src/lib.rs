pub mod automation;
pub mod headless;

pub use automation::{
    AutomationState, AutomationStep, AutomationTarget, AutomationTickResult, JsonAutomationFlow,
    JsonAutomationStep, JsonTarget, VIRTUAL_CURSOR, build_automation_steps, find_bounds_task,
    handle_automation_tick, handle_automation_virtual_tick, handle_bounds_resolved, try_tick,
    wrap_view,
};
pub use headless::export_video;
