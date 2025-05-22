use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::{App, egui};
use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use hotkey_utils::{HotkeyCapture, format_key_description, is_valid_hotkey};
use rand::Rng;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod hotkey_config;
mod hotkey_utils;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Note {
    key: String,
    time: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Song {
    name: String,
    #[serde(rename = "bpm")]
    bpm: u32,
    #[serde(rename = "bitsPerPage")]
    bits_per_page: u32,
    #[serde(rename = "pitchLevel")]
    pitch_level: i32,
    #[serde(rename = "helpText")]
    help_text: String,
    #[serde(rename = "songNotes")]
    song_notes: Vec<Note>,
}

#[derive(Default)]
struct AppState {
    song_path: Option<String>,
    speed: f32,
    is_playing: bool,
    is_paused: bool,
    status: String,
    progress: usize,
    total: usize,
    hotkeys: Hotkeys,
    show_help: bool,
    hotkey_capture: HotkeyCapture, // Track hotkey capture status
}

// Custom struct to hold hotkey settings
#[derive(Debug, Clone)]
struct Hotkeys {
    play_pause: Keycode,
    stop: Keycode,
    speed_up: Keycode,
    speed_down: Keycode,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            play_pause: Keycode::Space,
            stop: Keycode::Escape,
            speed_up: Keycode::Equal,   // + key
            speed_down: Keycode::Minus, // - key
        }
    }
}

pub struct SkySheetApp {
    state: Arc<Mutex<AppState>>,
    device_state: DeviceState,
    last_keys: Vec<Keycode>,
    last_hotkey_time: std::time::Instant,
}

impl Default for SkySheetApp {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState {
                speed: 1.0,
                ..Default::default()
            })),
            device_state: DeviceState::new(),
            last_keys: Vec::new(),
            last_hotkey_time: std::time::Instant::now(),
        }
    }
}

impl App for SkySheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set custom visuals for a prettier UI
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 255));
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 48);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 68);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(55, 55, 85);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(65, 65, 95);
        visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(70, 70, 100);
        ctx.set_visuals(visuals);

        // Check for global hotkeys
        let keys = self.device_state.get_keys();
        let state_clone = Arc::clone(&self.state);
        let mut state = state_clone.lock().unwrap();

        // Check for hotkey capture mode
        if state.hotkey_capture != HotkeyCapture::None && !keys.is_empty() {
            // Get first key press that wasn't in the previous frame
            for key in &keys {
                if !self.last_keys.contains(key) && is_valid_hotkey(*key) {
                    // Assign the key to the appropriate hotkey
                    match state.hotkey_capture {
                        HotkeyCapture::WaitingForPlayPause => {
                            state.hotkeys.play_pause = *key;
                            state.status = format!(
                                "Play/Pause hotkey set to: {}",
                                format_key_description(*key)
                            );
                        }
                        HotkeyCapture::WaitingForStop => {
                            state.hotkeys.stop = *key;
                            state.status =
                                format!("Stop hotkey set to: {}", format_key_description(*key));
                        }
                        HotkeyCapture::WaitingForSpeedUp => {
                            state.hotkeys.speed_up = *key;
                            state.status =
                                format!("Speed Up hotkey set to: {}", format_key_description(*key));
                        }
                        HotkeyCapture::WaitingForSpeedDown => {
                            state.hotkeys.speed_down = *key;
                            state.status = format!(
                                "Speed Down hotkey set to: {}",
                                format_key_description(*key)
                            );
                        }
                        _ => {}
                    }

                    // Reset capture mode
                    state.hotkey_capture = HotkeyCapture::None;
                    break;
                }
            }
        }

        // Only process global hotkeys if we're not capturing a new hotkey
        if state.hotkey_capture == HotkeyCapture::None {
            // Add a cooldown period to prevent accidental double-triggers
            let now = std::time::Instant::now();
            let cooldown_period = std::time::Duration::from_millis(200);
            let can_trigger_hotkey = now.duration_since(self.last_hotkey_time) >= cooldown_period;

            // Only process a key if it's new (wasn't pressed in the last frame)
            for key in &keys {
                if !self.last_keys.contains(key) && can_trigger_hotkey {
                    self.last_hotkey_time = now;
                    match *key {
                        k if k == state.hotkeys.play_pause => {
                            if state.is_playing {
                                state.is_paused = !state.is_paused;
                                if state.is_paused {
                                    state.status = "Paused".to_string();
                                } else {
                                    state.status = "Playing...".to_string();
                                }
                            } else if state.song_path.is_some() {
                                // Start playing
                                state.is_playing = true;
                                state.status = "Starting playback...".to_string();
                                let state_arc = Arc::clone(&self.state);
                                std::thread::spawn(move || {
                                    play_song_gui(state_arc);
                                });
                            }
                        }
                        k if k == state.hotkeys.stop => {
                            if state.is_playing {
                                state.is_playing = false;
                                state.is_paused = false;
                                state.status = "Stopped".to_string();
                            }
                        }
                        k if k == state.hotkeys.speed_up => {
                            state.speed += 0.1;
                            if state.speed > 2.0 {
                                state.speed = 2.0;
                            }
                            state.status = format!("Speed: {:.1}x", state.speed);
                        }
                        k if k == state.hotkeys.speed_down => {
                            state.speed -= 0.1;
                            if state.speed < 0.5 {
                                state.speed = 0.5;
                            }
                            state.status = format!("Speed: {:.1}x", state.speed);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.last_keys = keys;

        // Draw the UI with an improved layout and theme
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Sky Sheet Player");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if state.show_help {
                            "Hide Help"
                        } else {
                            "Show Help"
                        })
                        .clicked()
                    {
                        state.show_help = !state.show_help;
                    }
                });
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if state.show_help {
                // Help section
                ui.group(|ui| {
                    ui.heading("Hotkeys (work even when not focused)");

                    // Hotkey configuration section
                    ui.horizontal(|ui| {
                        ui.label("Play/Pause:");
                        ui.label(format_key_description(state.hotkeys.play_pause));
                        if ui.button("Change").clicked() {
                            state.hotkey_capture = HotkeyCapture::WaitingForPlayPause;
                            state.status = "Press any key to set Play/Pause hotkey...".to_string();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Stop:");
                        ui.label(format_key_description(state.hotkeys.stop));
                        if ui.button("Change").clicked() {
                            state.hotkey_capture = HotkeyCapture::WaitingForStop;
                            state.status = "Press any key to set Stop hotkey...".to_string();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Speed Up:");
                        ui.label(format_key_description(state.hotkeys.speed_up));
                        if ui.button("Change").clicked() {
                            state.hotkey_capture = HotkeyCapture::WaitingForSpeedUp;
                            state.status = "Press any key to set Speed Up hotkey...".to_string();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Speed Down:");
                        ui.label(format_key_description(state.hotkeys.speed_down));
                        if ui.button("Change").clicked() {
                            state.hotkey_capture = HotkeyCapture::WaitingForSpeedDown;
                            state.status = "Press any key to set Speed Down hotkey...".to_string();
                        }
                    });

                    ui.add_space(10.0);
                    ui.heading("How to Use");
                    ui.label(
                        "1. Click 'Select Song File' and choose a .txt file with JSON song data",
                    );
                    ui.label("2. Adjust speed with the slider or hotkeys if needed");
                    ui.label("3. Click 'Play' or press the play hotkey");
                    ui.label("4. Use the pause/stop buttons or hotkeys to control playback");
                });
                ui.add_space(10.0);
            }

            // File selection
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("📂 Select Song File").clicked() {
                        if let Some(path) =
                            FileDialog::new().add_filter("Text", &["txt"]).pick_file()
                        {
                            state.song_path = Some(path.display().to_string());
                            state.status = "Song loaded!".to_string();
                        }
                    }
                    if let Some(ref path) = state.song_path {
                        ui.label(format!("Selected: {}", path));
                    } else {
                        ui.label("No file selected");
                    }
                });
            });

            ui.add_space(10.0);

            // Playback controls
            ui.group(|ui| {
                ui.vertical(|ui| {
                    // Main playback controls in a row
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);

                        let btn_size = egui::Vec2::new(60.0, 40.0);

                        if !state.is_playing {
                            if ui
                                .add(
                                    egui::Button::new("▶️ Play")
                                        .min_size(btn_size)
                                        .fill(egui::Color32::from_rgb(50, 180, 100)),
                                )
                                .clicked()
                            {
                                let state_arc = Arc::clone(&self.state);
                                state.is_playing = true;
                                state.status = "Starting playback...".to_string();
                                std::thread::spawn(move || {
                                    play_song_gui(state_arc);
                                });
                            }
                        } else {
                            if state.is_paused {
                                if ui
                                    .add(
                                        egui::Button::new("▶️ Resume")
                                            .min_size(btn_size)
                                            .fill(egui::Color32::from_rgb(50, 180, 100)),
                                    )
                                    .clicked()
                                {
                                    state.is_paused = false;
                                    state.status = "Resuming...".to_string();
                                }
                            } else {
                                if ui
                                    .add(
                                        egui::Button::new("⏸️ Pause")
                                            .min_size(btn_size)
                                            .fill(egui::Color32::from_rgb(180, 180, 50)),
                                    )
                                    .clicked()
                                {
                                    state.is_paused = true;
                                    state.status = "Paused".to_string();
                                }
                            }

                            ui.add_space(10.0);

                            if ui
                                .add(
                                    egui::Button::new("⏹️ Stop")
                                        .min_size(btn_size)
                                        .fill(egui::Color32::from_rgb(180, 50, 50)),
                                )
                                .clicked()
                            {
                                state.is_playing = false;
                                state.is_paused = false;
                                state.status = "Stopped".to_string();
                            }
                        }

                        ui.add_space(20.0);

                        // Add a vertical separator
                        ui.separator();

                        ui.add_space(20.0);

                        // Speed control with fancy buttons
                        ui.vertical(|ui| {
                            ui.label("Speed:");
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new("−")
                                            .min_size(egui::Vec2::new(30.0, 30.0)),
                                    )
                                    .clicked()
                                {
                                    state.speed -= 0.1;
                                    if state.speed < 0.5 {
                                        state.speed = 0.5;
                                    }
                                }

                                ui.add(egui::Label::new(format!("{:.1}x", state.speed)));

                                if ui
                                    .add(
                                        egui::Button::new("+")
                                            .min_size(egui::Vec2::new(30.0, 30.0)),
                                    )
                                    .clicked()
                                {
                                    state.speed += 0.1;
                                    if state.speed > 2.0 {
                                        state.speed = 2.0;
                                    }
                                }
                            });
                        });
                    });

                    ui.add_space(5.0);

                    // Speed slider below the buttons
                    ui.add(
                        egui::Slider::new(&mut state.speed, 0.5..=2.0)
                            .text("Speed")
                            .show_value(false),
                    );
                });
            });

            ui.add_space(10.0);

            // Status and progress
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong("Status: ");
                    ui.label(&state.status);
                });
                if state.total > 0 {
                    ui.add_space(5.0);
                    ui.add(
                        egui::ProgressBar::new(state.progress as f32 / state.total as f32)
                            .text(format!("{}/{} notes", state.progress, state.total)),
                    );
                }
            });
        });
    }
}

fn play_song_gui(state_arc: Arc<Mutex<AppState>>) {
    // We'll use this function to safely get a lock and handle errors
    let mut get_lock = || state_arc.lock().unwrap();

    // Initial setup - get file path and speed
    let (path, speed, total_notes) = {
        let mut state = get_lock();
        state.is_playing = true;
        state.status = "Playing...".to_string();

        // Get path
        let path = match &state.song_path {
            Some(p) => p.clone(),
            None => {
                state.status = "No song file selected!".to_string();
                state.is_playing = false;
                return;
            }
        };

        let speed = state.speed;
        (path, speed, 0)
    };

    // Read the song file
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            let mut state = get_lock();
            state.status = format!("Failed to open file: {}", e);
            state.is_playing = false;
            return;
        }
    };

    // Read file contents
    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        let mut state = get_lock();
        state.status = format!("Failed to read file: {}", e);
        state.is_playing = false;
        return;
    }

    // Parse JSON
    let song = match serde_json::from_str::<Vec<Song>>(&contents) {
        Ok(songs) if !songs.is_empty() => songs[0].clone(),
        _ => {
            let mut state = get_lock();
            state.status =
                "Invalid song format! JSON must contain at least one Song object.".to_string();
            state.is_playing = false;
            return;
        }
    };

    // Initialize keyboard emulator
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            let mut state = get_lock();
            state.status = format!("Failed to initialize keyboard: {}", e);
            state.is_playing = false;
            return;
        }
    };

    // Set up RNG and timing
    let mut rng = rand::rng();
    let start_time = Instant::now();

    // Update total note count
    {
        let mut state = get_lock();
        state.total = song.song_notes.len();
        state.progress = 0;
    }

    // Play each note
    for (index, note) in song.song_notes.iter().enumerate() {
        // Check if we need to stop or pause
        let should_play = {
            let mut state = get_lock();

            // Check if playback should stop
            if !state.is_playing {
                state.status = "Stopped".to_string();
                return;
            }

            // Update progress
            state.progress = index + 1;

            // Handle pause if needed
            if state.is_paused {
                state.status = "Paused".to_string();
                drop(state); // Release lock while paused

                // Wait until we're unpaused or stopped
                loop {
                    thread::sleep(Duration::from_millis(100));

                    let state = get_lock();
                    if !state.is_playing {
                        return; // Stop playback
                    }

                    if !state.is_paused {
                        break; // Resume playback
                    }

                    drop(state); // Release lock for next iteration
                }

                // Set status to playing again
                let mut state = get_lock();
                state.status = "Playing...".to_string();
            }

            true
        };

        if !should_play {
            return;
        }

        // Calculate timing
        let adjusted_time = (note.time as f32 / speed) as u64;
        let target_time = Duration::from_millis(adjusted_time);
        let elapsed = start_time.elapsed();

        // Wait until the right moment to play this note
        if elapsed < target_time {
            thread::sleep(target_time - elapsed);
        }

        // Play the note if we have a valid keyboard mapping
        if let Some(key) = map_key(&note.key) {
            // Determine note characteristics
            let is_important = index % 4 == 0;
            let is_melodic_peak = index > 0
                && index < song.song_notes.len() - 1
                && note.time > song.song_notes[index - 1].time
                && (index == song.song_notes.len() - 1
                    || note.time > song.song_notes[index + 1].time);

            // Set note duration based on importance
            let base_hold = if is_important {
                55
            } else if is_melodic_peak {
                50
            } else {
                35
            };

            // Add a small variation to hold duration for a more natural sound
            let variation = rng.random_range(-5..=5);
            let hold_duration = Duration::from_millis((base_hold + variation) as u64);

            // Press and release the key
            let _ = enigo.key(Key::Unicode(key), Press);
            thread::sleep(hold_duration);
            let _ = enigo.key(Key::Unicode(key), Release);

            // Brief articulation gap between notes
            let gap = if is_important { 5 } else { 10 };
            thread::sleep(Duration::from_millis(gap));
        }
    }

    // Song finished
    let mut state = get_lock();
    state.status = "Song finished!".to_string();
    state.is_playing = false;
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([650.0, 550.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Sky Sheet Player",
        options,
        Box::new(|_cc| Ok(Box::new(SkySheetApp::default()))),
    );
}

fn map_key(key_str: &str) -> Option<char> {
    if let Some(key_num) = key_str.strip_prefix("1Key") {
        if let Ok(num) = key_num.parse::<u32>() {
            return match num {
                0 => Some('y'),
                1 => Some('u'),
                2 => Some('i'),
                3 => Some('o'),
                4 => Some('p'),
                5 => Some('h'),
                6 => Some('j'),
                7 => Some('k'),
                8 => Some('l'),
                9 => Some(';'),
                10 => Some('n'),
                11 => Some('m'),
                12 => Some('.'),
                13 => Some(','),
                14 => Some('/'),
                _ => None,
            };
        }
    }
    None
}
