//! Debug overlay plugin.
//!
//! Registers developer tooling that is compiled and registered unconditionally
//! (both debug and release) but is toggled at runtime via keyboard shortcuts.
//!
//! | Key | Action                   |
//! |-----|--------------------------|
//! | F1  | Toggle global wireframe  |

use bevy::{pbr::wireframe::WireframeConfig, prelude::*};

/// Bevy plugin that wires up runtime debug overlays.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // WireframePlugin must be added before WireframeConfig is usable.
        app.add_plugins(bevy::pbr::wireframe::WireframePlugin::default())
            .add_systems(Update, toggle_wireframe_on_f1);
    }
}

/// `Update` system: toggle global wireframe when F1 is just-pressed.
fn toggle_wireframe_on_f1(keys: Res<ButtonInput<KeyCode>>, mut config: ResMut<WireframeConfig>) {
    if keys.just_pressed(KeyCode::F1) {
        config.global = !config.global;
        info!("Wireframe {}", if config.global { "ON" } else { "OFF" });
    }
}
