//! Input resources.

use bevy::prelude::*;

/// Tracks the currently active gamepad, if any.
///
/// `entity = None` means no controller is connected and keyboard/mouse
/// fallback is active. Updated by [`crate::plugins::input::systems::on_gamepad_event`].
#[derive(Resource, Debug, Default)]
pub struct ActiveGamepad {
    /// The ECS entity carrying the [`Gamepad`] component, or `None`.
    pub entity: Option<Entity>,
}
