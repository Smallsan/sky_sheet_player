# Sky Sheet Player

A Rust application for playing "Sky: Children of the Light" song sheets with global hotkey controls and a user-friendly GUI.

## Features

- **Load and Play Song Files**: Import JSON-formatted song sheets for Sky: Children of the Light.
- **Global Hotkeys**: Control playback even when the application is not in focus.
- **Adjustable Playback Speed**: Speed up or slow down song playback to suit your needs.
- **Customizable Controls**: Personalize your hotkeys for a more comfortable experience.
- **Progress Tracking**: Visual progress bar to track how much of the song has been played.
- **Play/Pause/Stop Controls**: Full control over your song playback.
- **Random Timing Variations**: Adds subtle timing variations for more natural-sounding playback.

## Installation

### Requirements
- Rust and Cargo installed on your system

### Build from Source
1. Clone this repository
2. Run the following command in the project directory:
```
cargo build --release
```
3. The executable will be available in `target/release`

## Usage

1. Launch the application
2. Click "Select Song File" to choose a JSON song file (in .txt format)
3. Adjust the playback speed if needed using the slider or speed buttons
4. Click "Play" or use the global play hotkey (Space by default)
5. Control playback using the on-screen buttons or global hotkeys

### Default Hotkeys

- **Play/Pause**: Space
- **Stop**: Escape
- **Speed Up**: = (Equal)
- **Speed Down**: - (Minus)

These hotkeys can be customized in the application and your preferences will be saved for future sessions.

## Song File Format

The application expects song files in JSON format (usually with .txt extension) with the following structure:

```json
{
  "name": "Song Name",
  "bpm": 120,
  "bitsPerPage": 16,
  "pitchLevel": 0,
  "helpText": "Optional help text",
  "songNotes": [
    {
      "key": "A",
      "time": 100
    },
    ...
  ]
}
```

## Development

This project uses the following dependencies:
- eframe/egui for the GUI
- device_query for global hotkey monitoring
- enigo for keyboard simulation
- serde for JSON serialization/deserialization

## License

This project is open source and available under the MIT License.

## Acknowledgements

Thanks to the Sky: Children of the Light community for inspiring this project and creating sheet music for various songs.
