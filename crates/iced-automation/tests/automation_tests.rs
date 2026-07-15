use iced_automation::{
    AutomationStep, AutomationTarget, JsonAutomationFlow, JsonAutomationStep, JsonTarget,
    build_automation_steps,
};
use std::time::Duration;

#[test]
fn test_build_automation_steps_basic() {
    let flow = JsonAutomationFlow {
        flow_name: "test".into(),
        steps: vec![JsonAutomationStep {
            delay_ms: 100,
            target: JsonTarget::Widget { id: "btn1".into() },
            keycap_label: Some("Click Me".into()),
            message: serde_json::Value::Null,
        }],
    };
    let steps = build_automation_steps(
        &flow,
        |_id| "resolved-btn1".into(),
        |_msg| "Click Me".into(),
    );
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].execution_delay, Duration::from_millis(100));
    assert_eq!(steps[0].keycap_label, Some("Click Me".to_string()));
}

#[test]
fn test_build_automation_steps_empty() {
    let flow: JsonAutomationFlow = JsonAutomationFlow {
        flow_name: "empty".into(),
        steps: vec![],
    };
    let steps: Vec<AutomationStep<serde_json::Value>> = build_automation_steps::<serde_json::Value>(
        &flow,
        |_id| "unused".into(),
        |_msg| "unused".into(),
    );
    assert!(steps.is_empty());
}

#[test]
fn test_build_automation_steps_coordinate_target() {
    let flow = JsonAutomationFlow {
        flow_name: "test".into(),
        steps: vec![JsonAutomationStep {
            delay_ms: 50,
            target: JsonTarget::Coordinate { x: 42.0, y: 99.0 },
            keycap_label: None,
            message: serde_json::Value::Null,
        }],
    };
    let steps = build_automation_steps(&flow, |_id| "unused".into(), |_msg| "unused".into());
    assert_eq!(steps.len(), 1);
    match &steps[0].target {
        AutomationTarget::Pixel(p) => {
            assert_eq!(p.x, 42.0);
            assert_eq!(p.y, 99.0);
        }
        _ => panic!("expected Pixel target"),
    }
}
