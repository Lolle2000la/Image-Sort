use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Enter,
    Tab,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Escape,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    MediaPlayPause,
    MediaPlay,
    MediaPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    AudioVolumeUp,
    AudioVolumeDown,
    AudioVolumeMute,
    Character(char),
}

impl Key {
    pub fn display_name(&self) -> &'static str {
        match self {
            Key::Enter => "Enter",
            Key::Tab => "Tab",
            Key::Space => "Space",
            Key::ArrowUp => "Up",
            Key::ArrowDown => "Down",
            Key::ArrowLeft => "Left",
            Key::ArrowRight => "Right",
            Key::Escape => "Esc",
            Key::Backspace => "Backspace",
            Key::Delete => "Delete",
            Key::Home => "Home",
            Key::End => "End",
            Key::PageUp => "PageUp",
            Key::PageDown => "PageDown",
            Key::F1 => "F1",
            Key::F2 => "F2",
            Key::F3 => "F3",
            Key::F4 => "F4",
            Key::F5 => "F5",
            Key::F6 => "F6",
            Key::F7 => "F7",
            Key::F8 => "F8",
            Key::F9 => "F9",
            Key::F10 => "F10",
            Key::F11 => "F11",
            Key::F12 => "F12",
            Key::MediaPlayPause => "MediaPlayPause",
            Key::MediaPlay => "MediaPlay",
            Key::MediaPause => "MediaPause",
            Key::MediaStop => "MediaStop",
            Key::MediaTrackNext => "MediaTrackNext",
            Key::MediaTrackPrevious => "MediaTrackPrevious",
            Key::AudioVolumeUp => "AudioVolumeUp",
            Key::AudioVolumeDown => "AudioVolumeDown",
            Key::AudioVolumeMute => "AudioVolumeMute",
            Key::Character(c) => key_character_display(*c),
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "Enter" => Key::Enter,
            "Tab" => Key::Tab,
            "Space" => Key::Space,
            "Up" => Key::ArrowUp,
            "Down" => Key::ArrowDown,
            "Left" => Key::ArrowLeft,
            "Right" => Key::ArrowRight,
            "Esc" => Key::Escape,
            "Backspace" => Key::Backspace,
            "Delete" => Key::Delete,
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            "F1" => Key::F1,
            "F2" => Key::F2,
            "F3" => Key::F3,
            "F4" => Key::F4,
            "F5" => Key::F5,
            "F6" => Key::F6,
            "F7" => Key::F7,
            "F8" => Key::F8,
            "F9" => Key::F9,
            "F10" => Key::F10,
            "F11" => Key::F11,
            "F12" => Key::F12,
            "MediaPlayPause" => Key::MediaPlayPause,
            "MediaPlay" => Key::MediaPlay,
            "MediaPause" => Key::MediaPause,
            "MediaStop" => Key::MediaStop,
            "MediaTrackNext" => Key::MediaTrackNext,
            "MediaTrackPrevious" => Key::MediaTrackPrevious,
            "AudioVolumeUp" => Key::AudioVolumeUp,
            "AudioVolumeDown" => Key::AudioVolumeDown,
            "AudioVolumeMute" => Key::AudioVolumeMute,
            _ if s.chars().count() == 1 => Key::Character(s.chars().next().unwrap()),
            _ => return None,
        })
    }
}

impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Key::Character(c) => {
                let mut buf = [0u8; 4];
                serializer.serialize_str(c.encode_utf8(&mut buf))
            }
            _ => serializer.serialize_str(self.display_name()),
        }
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Key::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("unknown key: {s}")))
    }
}

fn key_character_display(c: char) -> &'static str {
    match c {
        'A' => "A",
        'B' => "B",
        'C' => "C",
        'D' => "D",
        'E' => "E",
        'F' => "F",
        'G' => "G",
        'H' => "H",
        'I' => "I",
        'J' => "J",
        'K' => "K",
        'L' => "L",
        'M' => "M",
        'N' => "N",
        'O' => "O",
        'P' => "P",
        'Q' => "Q",
        'R' => "R",
        'S' => "S",
        'T' => "T",
        'U' => "U",
        'V' => "V",
        'W' => "W",
        'X' => "X",
        'Y' => "Y",
        'Z' => "Z",
        '0' => "0",
        '1' => "1",
        '2' => "2",
        '3' => "3",
        '4' => "4",
        '5' => "5",
        '6' => "6",
        '7' => "7",
        '8' => "8",
        '9' => "9",
        ' ' => "Space",
        _ => unreachable!("Key::Character can only be ascii alphanumeric or space"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyBinding {
    pub fn new(key: Key) -> Self {
        Self {
            key,
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
    #[serde(default = "default_reveal_in_file_manager_binding")]
    pub reveal_in_file_manager: KeyBinding,
}

fn default_reveal_in_file_manager_binding() -> KeyBinding {
    KeyBinding::new(Key::Character('L'))
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_to_folder: KeyBinding::new(Key::ArrowUp),
            copy_to_folder: KeyBinding::new(Key::ArrowUp).with_shift(),
            delete: KeyBinding::new(Key::ArrowDown),
            rename: KeyBinding::new(Key::Character('R')),
            go_left: KeyBinding::new(Key::ArrowLeft),
            go_right: KeyBinding::new(Key::ArrowRight),
            create_folder: KeyBinding::new(Key::Character('C')),
            folder_up: KeyBinding::new(Key::Character('W')),
            folder_left: KeyBinding::new(Key::Character('A')),
            folder_down: KeyBinding::new(Key::Character('S')),
            folder_right: KeyBinding::new(Key::Character('D')),
            undo: KeyBinding::new(Key::Character('Q')),
            redo: KeyBinding::new(Key::Character('E')),
            open_folder: KeyBinding::new(Key::Character('O')),
            open_selected_folder: KeyBinding::new(Key::Enter),
            pin: KeyBinding::new(Key::Character('P')),
            pin_selected: KeyBinding::new(Key::Character('F')),
            unpin: KeyBinding::new(Key::Character('U')),
            move_pinned_up: KeyBinding::new(Key::Character('W')).with_ctrl(),
            move_pinned_down: KeyBinding::new(Key::Character('S')).with_ctrl(),
            search_images: KeyBinding::new(Key::Character('I')),
            toggle_metadata_panel: KeyBinding::new(Key::Character('M')),
            reveal_in_file_manager: KeyBinding::new(Key::Character('L')),
        }
    }
}
