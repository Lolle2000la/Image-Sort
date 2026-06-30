use iced::keyboard::{self, Key};
use iced::Subscription;

use crate::message::Message;

pub fn keyboard_subscription() -> Subscription<Message> {
    iced::event::listen_with(|event, status, _window| {
        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                let is_exit_key = matches!(
                    key,
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)
                        | iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                );

                if status == iced::event::Status::Captured && !is_exit_key {
                    return None;
                }

                key_to_name(key).map(|key_name| {
                    Message::KeyCaptured(
                        key_name,
                        modifiers.control(),
                        modifiers.shift(),
                        modifiers.alt(),
                    )
                })
            }
            _ => None,
        }
    })
}

pub(crate) fn key_to_name(key: Key) -> Option<String> {
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
                keyboard::key::Named::MediaPlayPause => "MediaPlayPause",
                keyboard::key::Named::MediaPlay => "MediaPlay",
                keyboard::key::Named::MediaPause => "MediaPause",
                keyboard::key::Named::MediaStop => "MediaStop",
                keyboard::key::Named::MediaTrackNext => "MediaTrackNext",
                keyboard::key::Named::MediaTrackPrevious => "MediaTrackPrevious",
                keyboard::key::Named::AudioVolumeUp => "AudioVolumeUp",
                keyboard::key::Named::AudioVolumeDown => "AudioVolumeDown",
                keyboard::key::Named::AudioVolumeMute => "AudioVolumeMute",
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
        ("go_left".into(), kb.go_left.clone()),
        ("go_right".into(), kb.go_right.clone()),
        ("create_folder".into(), kb.create_folder.clone()),
        ("folder_up".into(), kb.folder_up.clone()),
        ("folder_left".into(), kb.folder_left.clone()),
        ("folder_down".into(), kb.folder_down.clone()),
        ("folder_right".into(), kb.folder_right.clone()),
        ("undo".into(), kb.undo.clone()),
        ("redo".into(), kb.redo.clone()),
        ("open_folder".into(), kb.open_folder.clone()),
        ("open_selected_folder".into(), kb.open_selected_folder.clone()),
        ("pin".into(), kb.pin.clone()),
        ("pin_selected".into(), kb.pin_selected.clone()),
        ("unpin".into(), kb.unpin.clone()),
        ("move_pinned_up".into(), kb.move_pinned_up.clone()),
        ("move_pinned_down".into(), kb.move_pinned_down.clone()),
        ("search_images".into(), kb.search_images.clone()),
        (
            "toggle_metadata_panel".into(),
            kb.toggle_metadata_panel.clone(),
        ),
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
        "go_left" => kb.go_left = binding,
        "go_right" => kb.go_right = binding,
        "create_folder" => kb.create_folder = binding,
        "folder_up" => kb.folder_up = binding,
        "folder_left" => kb.folder_left = binding,
        "folder_down" => kb.folder_down = binding,
        "folder_right" => kb.folder_right = binding,
        "undo" => kb.undo = binding,
        "redo" => kb.redo = binding,
        "open_folder" => kb.open_folder = binding,
        "open_selected_folder" => kb.open_selected_folder = binding,
        "pin" => kb.pin = binding,
        "pin_selected" => kb.pin_selected = binding,
        "unpin" => kb.unpin = binding,
        "move_pinned_up" => kb.move_pinned_up = binding,
        "move_pinned_down" => kb.move_pinned_down = binding,
        "search_images" => kb.search_images = binding,
        "toggle_metadata_panel" => kb.toggle_metadata_panel = binding,
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

#[cfg(test)]
mod tests {
    use super::*;
    use media_sort_core::settings::keybindings::{KeyBinding, KeyBindings};
    use smol_str::SmolStr;

    #[test]
    fn test_key_to_name_named_keys() {
        assert_eq!(
            key_to_name(iced::keyboard::Key::Named(
                iced::keyboard::key::Named::Enter
            )),
            Some("Enter".into())
        );
        assert_eq!(
            key_to_name(iced::keyboard::Key::Named(
                iced::keyboard::key::Named::Space
            )),
            Some("Space".into())
        );
        assert_eq!(
            key_to_name(iced::keyboard::Key::Named(
                iced::keyboard::key::Named::ArrowUp
            )),
            Some("Up".into())
        );
        assert_eq!(
            key_to_name(iced::keyboard::Key::Named(
                iced::keyboard::key::Named::Escape
            )),
            Some("Esc".into())
        );
        assert_eq!(
            key_to_name(iced::keyboard::Key::Named(iced::keyboard::key::Named::F1)),
            Some("F1".into())
        );
        assert_eq!(
            key_to_name(iced::keyboard::Key::Named(iced::keyboard::key::Named::F12)),
            Some("F12".into())
        );
    }

    #[test]
    fn test_key_to_name_character() {
        assert_eq!(
            key_to_name(iced::keyboard::Key::Character(SmolStr::new("a"))),
            Some("A".into())
        );
        assert_eq!(
            key_to_name(iced::keyboard::Key::Character(SmolStr::new("z"))),
            Some("Z".into())
        );
    }

    #[test]
    fn test_key_to_name_unknown() {
        assert_eq!(key_to_name(iced::keyboard::Key::Unidentified), None);
        assert_eq!(
            key_to_name(iced::keyboard::Key::Character(SmolStr::new(""))),
            None
        );
    }

    #[test]
    fn test_format_keybinding_plain() {
        let kb = KeyBinding::new("A");
        assert_eq!(format_keybinding(&kb), "A");
    }

    #[test]
    fn test_format_keybinding_ctrl() {
        let kb = KeyBinding::new("X").with_ctrl();
        assert_eq!(format_keybinding(&kb), "Ctrl+X");
    }

    #[test]
    fn test_format_keybinding_shift() {
        let mut kb = KeyBinding::new("A");
        kb.shift = true;
        assert_eq!(format_keybinding(&kb), "Shift+A");
    }

    #[test]
    fn test_format_keybinding_ctrl_shift() {
        let kb = KeyBinding::new("Z").with_ctrl().with_shift();
        assert_eq!(format_keybinding(&kb), "Ctrl+Shift+Z");
    }

    #[test]
    fn test_format_keybinding_all_modifiers() {
        let kb = KeyBinding::new("Delete")
            .with_ctrl()
            .with_shift()
            .with_alt();
        assert_eq!(format_keybinding(&kb), "Ctrl+Shift+Alt+Delete");
    }

    #[test]
    fn test_update_keybinding_known_name() {
        let mut kb = KeyBindings::default();
        update_keybinding(&mut kb, "undo", "Z", true, false, false);
        assert_eq!(kb.undo.key, "Z");
        assert!(kb.undo.ctrl);
        assert!(!kb.undo.shift);
        assert!(!kb.undo.alt);
    }

    #[test]
    fn test_update_keybinding_unknown_name() {
        let mut kb = KeyBindings::default();
        let saved = kb.redo.key.clone();
        update_keybinding(&mut kb, "nonexistent_action", "X", false, false, false);
        assert_eq!(kb.redo.key, saved);
    }

    #[test]
    fn test_keybinding_list_length() {
        use crate::state::AppState;
        use media_sort_core::settings::store::SettingsStore;

        let state = AppState::new(SettingsStore::default());
        let list = keybinding_list(&state);
        assert_eq!(list.len(), 21);
    }

    #[test]
    fn test_keybinding_list_has_known_actions() {
        use crate::state::AppState;
        use media_sort_core::settings::store::SettingsStore;

        let state = AppState::new(SettingsStore::default());
        let list = keybinding_list(&state);
        let names: Vec<&str> = list.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"undo"));
        assert!(names.contains(&"redo"));
        assert!(names.contains(&"delete"));
        assert!(names.contains(&"rename"));
        assert!(names.contains(&"move_to_folder"));
    }
}
