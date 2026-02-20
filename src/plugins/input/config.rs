//! Input configuration — loaded from `assets/config/input.ron`.

use std::collections::HashMap;

use bevy::{
    input::gamepad::{GamepadAxis, GamepadButton},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Complete input configuration for Neon Expanse.
///
/// Loaded from `assets/config/input.ron` at startup. Falls back to
/// [`InputConfig::default()`] if the file is absent or malformed —
/// the application must never crash due to a missing config file.
///
/// # Modding
/// Mods may supply a replacement `assets/config/input.ron` to fully
/// remap all controls without recompiling.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Dead-zone radius applied to all analog axes.
    /// Inputs within this radius from centre produce a 0.0 action value.
    pub deadzone_radius: f32,
    /// Maps named action strings (e.g. `"MoveForward"`) to gamepad axis types.
    pub axis_map: HashMap<String, GamepadAxis>,
    /// Maps named action strings to gamepad button types.
    pub button_map: HashMap<String, GamepadButton>,
    /// Maps named action strings to keyboard key fallbacks.
    pub keyboard_map: HashMap<String, KeyCode>,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut axis_map = HashMap::new();
        axis_map.insert("MoveForward".into(), GamepadAxis::LeftStickY);
        axis_map.insert("MoveRight".into(), GamepadAxis::LeftStickX);
        axis_map.insert("LookVertical".into(), GamepadAxis::RightStickY);
        axis_map.insert("LookHorizontal".into(), GamepadAxis::RightStickX);

        let mut button_map = HashMap::new();
        button_map.insert("PrimaryAction".into(), GamepadButton::South);
        button_map.insert("Jump".into(), GamepadButton::South);
        button_map.insert("Sprint".into(), GamepadButton::LeftThumb);

        let mut keyboard_map = HashMap::new();
        keyboard_map.insert("MoveForward".into(), KeyCode::KeyW);
        keyboard_map.insert("MoveRight".into(), KeyCode::KeyD);
        keyboard_map.insert("Jump".into(), KeyCode::Space);
        keyboard_map.insert("Sprint".into(), KeyCode::ShiftLeft);
        keyboard_map.insert("PrimaryAction".into(), KeyCode::KeyE);

        Self {
            deadzone_radius: 0.15,
            axis_map,
            button_map,
            keyboard_map,
        }
    }
}

/// Loads [`InputConfig`] from `assets/config/input.ron`.
///
/// Called synchronously during `Startup`. Uses `std::fs::read_to_string`
/// because the config must be available before the first frame —
/// Bevy's async asset system cannot guarantee this timing.
///
/// Falls back to [`InputConfig::default()`] on any I/O or parse error
/// and emits a `warn!` so developers notice.
pub fn load_input_config() -> InputConfig {
    load_input_config_from("assets/config/input.ron")
}

/// Loads [`InputConfig`] from the given file path.
///
/// Shared implementation for [`load_input_config`] and tests.
/// Falls back to [`InputConfig::default()`] on any I/O or parse error.
pub fn load_input_config_from(path: &str) -> InputConfig {
    match std::fs::read_to_string(path) {
        Ok(contents) => match ron::from_str(&contents) {
            Ok(config) => {
                info!("Input config loaded from {path}");
                config
            }
            Err(e) => {
                warn!("Failed to parse {path}: {e} — using defaults");
                InputConfig::default()
            }
        },
        Err(_) => {
            info!("No {path} found — using default input bindings");
            InputConfig::default()
        }
    }
}
