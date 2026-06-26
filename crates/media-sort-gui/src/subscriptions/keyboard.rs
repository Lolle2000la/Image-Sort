use iced::keyboard::{self, Key};
use iced::Subscription;

use crate::message::Message;

pub fn keyboard_subscription() -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        key_to_name(key).map(|key_name| {
            Message::KeyCaptured(
                key_name,
                modifiers.control(),
                modifiers.shift(),
                modifiers.alt(),
            )
        })
    })
}

fn key_to_name(key: Key) -> Option<String> {
    match &key {
        Key::Named(named) => {
            let name = match named {
                keyboard::key::Named::Enter => "Enter",
                keyboard::key::Named::Tab => "Tab",
                keyboard::key::Named::Space => "Space",
                keyboard::key::Named::ArrowUp => "Up",
                keyboard::key::Named::ArrowDown => "Down",
                keyboard::key::Named::ArrowLeft => "Left",
                keyboard::key::Named::ArrowRight => "Right",
                keyboard::key::Named::Escape => "Esc",
                keyboard::key::Named::Backspace => "Backspace",
                keyboard::key::Named::Delete => "Delete",
                keyboard::key::Named::Home => "Home",
                keyboard::key::Named::End => "End",
                keyboard::key::Named::PageUp => "PageUp",
                keyboard::key::Named::PageDown => "PageDown",
                keyboard::key::Named::F1 => "F1",
                keyboard::key::Named::F2 => "F2",
                keyboard::key::Named::F3 => "F3",
                keyboard::key::Named::F4 => "F4",
                keyboard::key::Named::F5 => "F5",
                keyboard::key::Named::F6 => "F6",
                keyboard::key::Named::F7 => "F7",
                keyboard::key::Named::F8 => "F8",
                keyboard::key::Named::F9 => "F9",
                keyboard::key::Named::F10 => "F10",
                keyboard::key::Named::F11 => "F11",
                keyboard::key::Named::F12 => "F12",
                _ => return None,
            };
            Some(name.to_string())
        }
        Key::Character(c) if !c.is_empty() => Some(c.to_uppercase()),
        _ => None,
    }
}

pub fn keybinding_list(
    state: &crate::state::AppState,
) -> Vec<(String, media_sort_core::settings::keybindings::KeyBinding)> {
    let kb = &state.settings.keybindings;
    vec![
        ("move_to_folder".into(), kb.move_to_folder.clone()),
        ("delete".into(), kb.delete.clone()),
        ("rename".into(), kb.rename.clone()),
        ("undo".into(), kb.undo.clone()),
        ("redo".into(), kb.redo.clone()),
        ("open_folder".into(), kb.open_folder.clone()),
        ("search_images".into(), kb.search_images.clone()),
        (
            "toggle_metadata_panel".into(),
            kb.toggle_metadata_panel.clone(),
        ),
        ("pin".into(), kb.pin.clone()),
        ("unpin".into(), kb.unpin.clone()),
    ]
}

pub fn update_keybinding(
    kb: &mut media_sort_core::settings::keybindings::KeyBindings,
    name: &str,
    key: &str,
    ctrl: bool,
    shift: bool,
    alt: bool,
) {
    let binding = media_sort_core::settings::keybindings::KeyBinding {
        key: key.to_string(),
        ctrl,
        shift,
        alt,
        meta: false,
    };
    match name {
        "move_to_folder" => kb.move_to_folder = binding,
        "delete" => kb.delete = binding,
        "rename" => kb.rename = binding,
        "undo" => kb.undo = binding,
        "redo" => kb.redo = binding,
        "open_folder" => kb.open_folder = binding,
        "search_images" => kb.search_images = binding,
        "toggle_metadata_panel" => kb.toggle_metadata_panel = binding,
        "pin" => kb.pin = binding,
        "unpin" => kb.unpin = binding,
        _ => {}
    }
}

pub fn format_keybinding(binding: &media_sort_core::settings::keybindings::KeyBinding) -> String {
    let mut parts = Vec::new();
    if binding.ctrl {
        parts.push("Ctrl");
    }
    if binding.shift {
        parts.push("Shift");
    }
    if binding.alt {
        parts.push("Alt");
    }
    parts.push(&binding.key);
    parts.join("+")
}
