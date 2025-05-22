use device_query::Keycode;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyCapture {
    None,
    WaitingForPlayPause,
    WaitingForStop,
    WaitingForSpeedUp,
    WaitingForSpeedDown,
}

impl Default for HotkeyCapture {
    fn default() -> Self {
        Self::None
    }
}

// Makes sure we don't use keys that are essential for the application
pub fn is_valid_hotkey(key: Keycode) -> bool {
    // Reserved system keys that shouldn't be used as hotkeys
    let reserved_keys = vec![
        Keycode::F1,
        Keycode::F2,
        Keycode::F3,
        Keycode::F4,
        Keycode::F5,
        Keycode::F6,
        Keycode::F7,
        Keycode::F8,
        Keycode::F9,
        Keycode::F10,
        Keycode::F11,
        Keycode::F12,
        Keycode::LAlt,
        Keycode::RAlt,
        Keycode::LControl,
        Keycode::RControl,
        Keycode::Tab,
        Keycode::CapsLock,
    ];

    !reserved_keys.contains(&key)
}

// Human-readable descriptions of keys
pub fn format_key_description(key: Keycode) -> String {
    match key {
        Keycode::Space => "Space".to_string(),
        Keycode::Escape => "Esc".to_string(),
        Keycode::Equal => "+".to_string(),
        Keycode::Minus => "-".to_string(),
        _ => format!("{:?}", key),
    }
}
