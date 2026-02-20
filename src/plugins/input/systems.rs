//! Input systems: gamepad connection handling and dead-zone configuration.

use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
use bevy::{ecs::message::MessageReader, input::gamepad::GamepadSettings, prelude::*};

use super::{config::InputConfig, resources::ActiveGamepad};

/// Startup system: inserts [`InputConfig`] (loaded from RON or defaults)
/// and [`ActiveGamepad`] resources, and logs keyboard fallback status.
pub fn setup_input(mut commands: Commands) {
    let config = super::config::load_input_config();
    commands.insert_resource(config);
    commands.insert_resource(ActiveGamepad::default());
    // No controller detected yet — will receive GamepadConnectionEvent if one is present.
    info!("No controller detected — keyboard/mouse fallback active");
}

/// Detects PS4/PS5 controller connection and disconnection events.
///
/// On connect: logs controller name, updates [`ActiveGamepad`], and
/// configures dead zones from [`InputConfig`].
/// On disconnect: logs fallback message, clears [`ActiveGamepad`].
pub fn on_gamepad_event(
    mut events: MessageReader<GamepadConnectionEvent>,
    mut active: ResMut<ActiveGamepad>,
    config: Res<InputConfig>,
    mut commands: Commands,
) {
    for event in events.read() {
        match &event.connection {
            GamepadConnection::Connected { name, .. } => {
                info!("PS4/PS5 controller connected: {name}");
                active.entity = Some(event.gamepad);

                // Apply dead-zone from InputConfig to this specific gamepad entity.
                let dz = config.deadzone_radius;
                let mut axis_settings = bevy::input::gamepad::AxisSettings::default();
                axis_settings.set_deadzone_lowerbound(-dz);
                axis_settings.set_deadzone_upperbound(dz);

                commands.entity(event.gamepad).insert(GamepadSettings {
                    default_axis_settings: axis_settings,
                    ..Default::default()
                });
            }
            GamepadConnection::Disconnected => {
                info!("Controller disconnected — keyboard/mouse fallback active");
                if active.entity == Some(event.gamepad) {
                    active.entity = None;
                }
            }
        }
    }
}
