use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyBinding {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub move_to_folder: KeyBinding,
    pub copy_to_folder: KeyBinding,
    pub delete: KeyBinding,
    pub rename: KeyBinding,
    pub go_left: KeyBinding,
    pub go_right: KeyBinding,
    pub create_folder: KeyBinding,
    pub folder_up: KeyBinding,
    pub folder_left: KeyBinding,
    pub folder_down: KeyBinding,
    pub folder_right: KeyBinding,
    pub undo: KeyBinding,
    pub redo: KeyBinding,
    pub open_folder: KeyBinding,
    pub open_selected_folder: KeyBinding,
    pub pin: KeyBinding,
    pub pin_selected: KeyBinding,
    pub unpin: KeyBinding,
    pub move_pinned_up: KeyBinding,
    pub move_pinned_down: KeyBinding,
    pub search_images: KeyBinding,
    pub toggle_metadata_panel: KeyBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_to_folder: KeyBinding::new("Up"),
            copy_to_folder: KeyBinding::new("Up").with_shift(),
            delete: KeyBinding::new("Down"),
            rename: KeyBinding::new("R"),
            go_left: KeyBinding::new("Left"),
            go_right: KeyBinding::new("Right"),
            create_folder: KeyBinding::new("C"),
            folder_up: KeyBinding::new("W"),
            folder_left: KeyBinding::new("A"),
            folder_down: KeyBinding::new("S"),
            folder_right: KeyBinding::new("D"),
            undo: KeyBinding::new("Q"),
            redo: KeyBinding::new("E"),
            open_folder: KeyBinding::new("O"),
            open_selected_folder: KeyBinding::new("Enter"),
            pin: KeyBinding::new("P"),
            pin_selected: KeyBinding::new("F"),
            unpin: KeyBinding::new("U"),
            move_pinned_up: KeyBinding::new("W").with_ctrl(),
            move_pinned_down: KeyBinding::new("S").with_ctrl(),
            search_images: KeyBinding::new("I"),
            toggle_metadata_panel: KeyBinding::new("M"),
        }
    }
}
