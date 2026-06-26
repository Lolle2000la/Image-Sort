use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowPosition {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub screen_count: u32,
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self {
            left: 100,
            top: 100,
            width: 1000,
            height: 600,
            maximized: false,
            screen_count: 1,
        }
    }
}
