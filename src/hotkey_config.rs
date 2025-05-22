use device_query::Keycode;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::Hotkeys;

#[derive(Debug, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub play_pause: String,
    pub stop: String,
    pub speed_up: String,
    pub speed_down: String,
}

impl From<&Hotkeys> for HotkeyConfig {
    fn from(hotkeys: &Hotkeys) -> Self {
        Self {
            play_pause: format!("{:?}", hotkeys.play_pause),
            stop: format!("{:?}", hotkeys.stop),
            speed_up: format!("{:?}", hotkeys.speed_up),
            speed_down: format!("{:?}", hotkeys.speed_down),
        }
    }
}

pub fn save_hotkeys(hotkeys: &Hotkeys) -> Result<(), String> {
    let config = HotkeyConfig::from(hotkeys);
    let config_dir =
        dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;
    let app_config_dir = config_dir.join("sky_sheet_player");

    // Create directory if it doesn't exist
    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_path = app_config_dir.join("hotkeys.json");
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize hotkey config: {}", e))?;

    let mut file =
        File::create(config_path).map_err(|e| format!("Failed to create config file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

pub fn load_hotkeys() -> Result<Hotkeys, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;
    let config_path = config_dir.join("sky_sheet_player").join("hotkeys.json");

    if !config_path.exists() {
        return Ok(Hotkeys::default());
    }

    let mut file =
        File::open(config_path).map_err(|e| format!("Failed to open config file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: HotkeyConfig = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    // Convert string keys to Keycode enums
    let play_pause = parse_keycode(&config.play_pause).unwrap_or(Keycode::Space);
    let stop = parse_keycode(&config.stop).unwrap_or(Keycode::Escape);
    let speed_up = parse_keycode(&config.speed_up).unwrap_or(Keycode::Equal);
    let speed_down = parse_keycode(&config.speed_down).unwrap_or(Keycode::Minus);

    Ok(Hotkeys {
        play_pause,
        stop,
        speed_up,
        speed_down,
    })
}

fn parse_keycode(key_str: &str) -> Option<Keycode> {
    // Manual mapping of keycode strings to Keycode enum variants
    match key_str.trim() {
        "Space" => Some(Keycode::Space),
        "Escape" => Some(Keycode::Escape),
        "Equal" => Some(Keycode::Equal),
        "Minus" => Some(Keycode::Minus),
        "Key1" => Some(Keycode::Key1),
        "Key2" => Some(Keycode::Key2),
        "Key3" => Some(Keycode::Key3),
        "Key4" => Some(Keycode::Key4),
        "Key5" => Some(Keycode::Key5),
        "Key6" => Some(Keycode::Key6),
        "Key7" => Some(Keycode::Key7),
        "Key8" => Some(Keycode::Key8),
        "Key9" => Some(Keycode::Key9),
        "Key0" => Some(Keycode::Key0),
        "A" => Some(Keycode::A),
        "B" => Some(Keycode::B),
        "C" => Some(Keycode::C),
        "D" => Some(Keycode::D),
        "E" => Some(Keycode::E),
        "F" => Some(Keycode::F),
        "G" => Some(Keycode::G),
        "H" => Some(Keycode::H),
        "I" => Some(Keycode::I),
        "J" => Some(Keycode::J),
        "K" => Some(Keycode::K),
        "L" => Some(Keycode::L),
        "M" => Some(Keycode::M),
        "N" => Some(Keycode::N),
        "O" => Some(Keycode::O),
        "P" => Some(Keycode::P),
        "Q" => Some(Keycode::Q),
        "R" => Some(Keycode::R),
        "S" => Some(Keycode::S),
        "T" => Some(Keycode::T),
        "U" => Some(Keycode::U),
        "V" => Some(Keycode::V),
        "W" => Some(Keycode::W),
        "X" => Some(Keycode::X),
        "Y" => Some(Keycode::Y),
        "Z" => Some(Keycode::Z),
        _ => None,
    }
}
