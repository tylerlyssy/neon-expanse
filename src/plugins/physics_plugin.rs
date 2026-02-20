//! Physics plugin wrapper for Neon Expanse.
//!
//! Wraps [`avian3d::prelude::PhysicsPlugins`] with logging and
//! integrates it into the plugin registration order. All avian3d
//! APIs are available to any crate/plugin via `use avian3d::prelude::*`.

use avian3d::prelude::*;
use bevy::prelude::*;

/// Bevy plugin that activates the Avian3d physics simulation.
///
/// Physics runs in `FixedPostUpdate` at 64 Hz (inherited from
/// `Time::<Fixed>` set by [`CorePlugin`]). Gravity defaults to
/// `Vec3::NEG_Y * 9.81 m/s²` and affects only `RigidBody::Dynamic`
/// entities — `RigidBody::Static` bodies are completely unaffected.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        info!("Neon Expanse physics initialized (avian3d, 64 Hz)");
    }
}
