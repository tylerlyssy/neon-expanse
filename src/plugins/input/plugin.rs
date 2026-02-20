//! NeonInputPlugin — PS4/PS5 controller + keyboard/mouse input system.

use bevy::prelude::*;

use super::systems::{on_gamepad_event, setup_input};

/// Bevy plugin that implements Neon Expanse input handling.
///
/// # Responsibilities
/// - Loads [`InputConfig`] from `assets/config/input.ron` (RON format,
///   moddable without recompile). Falls back to hardcoded defaults.
/// - Inserts [`ActiveGamepad`] resource tracking the connected controller.
/// - Auto-detects PS4/PS5 controller on connection and applies dead-zone
///   filtering from [`InputConfig`].
/// - Logs all connection/disconnection events with human-readable messages.
/// - Gracefully switches to keyboard/mouse fallback if controller disconnects.
pub struct NeonInputPlugin;

impl Plugin for NeonInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_input)
            .add_systems(Update, on_gamepad_event);
    }
}
