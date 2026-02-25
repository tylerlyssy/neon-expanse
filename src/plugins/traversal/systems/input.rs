//! Input polling systems — on-foot and vehicle.
//!
//! Systems run in `Update` and write normalised snapshots into
//! [`OnFootInputState`] / [`VehicleInputState`] components for consumption by
//! physics systems running in `FixedUpdate`. FR-027.
//!
//! In Bevy 0.18 `Gamepad` is a *component* on each gamepad entity.
//! `GamepadButton` and `GamepadAxis` are plain enums with no entity field.
//! Read buttons/axes via `Gamepad::just_pressed()`, `Gamepad::pressed()`, and
//! `Gamepad::get()`.

use bevy::{
    input::gamepad::{Gamepad, GamepadAxis, GamepadButton},
    prelude::*,
};

use crate::plugins::input::{config::InputConfig, resources::ActiveGamepad};
use crate::plugins::traversal::components::{OnFootInputState, PassengerOf, VehicleInputState};

// ── T017: poll_on_foot_input ──────────────────────────────────────────────────

/// Update system: reads keyboard/gamepad state and writes a normalised
/// [`OnFootInputState`] snapshot on every player entity that is not a passenger.
pub fn poll_on_foot_input(
    mut query: Query<&mut OnFootInputState, Without<PassengerOf>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    config: Res<InputConfig>,
    active_pad: Res<ActiveGamepad>,
) {
    let dz = config.deadzone_radius;
    let pad = active_pad.entity.and_then(|e| gamepads.get(e).ok());

    for mut state in &mut query {
        // ── Movement ─────────────────────────────────────────────────────────
        let gp_fwd = pad
            .map(|g| apply_dz(g.get(GamepadAxis::LeftStickY).unwrap_or(0.0), dz))
            .unwrap_or(0.0);
        let gp_right = pad
            .map(|g| apply_dz(g.get(GamepadAxis::LeftStickX).unwrap_or(0.0), dz))
            .unwrap_or(0.0);
        let kb_fwd =
            (keys.pressed(KeyCode::KeyW) as i32 - keys.pressed(KeyCode::KeyS) as i32) as f32;
        let kb_right =
            (keys.pressed(KeyCode::KeyD) as i32 - keys.pressed(KeyCode::KeyA) as i32) as f32;
        let raw_fwd = if kb_fwd.abs() > 0.0 { kb_fwd } else { gp_fwd };
        let raw_right = if kb_right.abs() > 0.0 {
            kb_right
        } else {
            gp_right
        };
        state.move_dir = Vec2::new(raw_right, raw_fwd).clamp_length_max(1.0);

        // ── Look ──────────────────────────────────────────────────────────────
        let look_h = pad
            .map(|g| apply_dz(g.get(GamepadAxis::RightStickX).unwrap_or(0.0), dz))
            .unwrap_or(0.0);
        let look_v = pad
            .map(|g| apply_dz(g.get(GamepadAxis::RightStickY).unwrap_or(0.0), dz))
            .unwrap_or(0.0);
        state.look_delta = Vec2::new(look_h, look_v);

        // ── Jump ──────────────────────────────────────────────────────────────
        let gp_jump = pad
            .map(|g| g.just_pressed(GamepadButton::South))
            .unwrap_or(false);
        state.jump_pressed = keys.just_pressed(KeyCode::Space) || gp_jump;

        // ── Sprint ────────────────────────────────────────────────────────────
        let gp_sprint = pad
            .map(|g| g.pressed(GamepadButton::LeftThumb))
            .unwrap_or(false);
        state.sprint_held = keys.pressed(KeyCode::ShiftLeft) || gp_sprint;

        // ── Enter/exit vehicle ────────────────────────────────────────────────
        let gp_enter = pad
            .map(|g| g.just_pressed(GamepadButton::North))
            .unwrap_or(false);
        state.enter_exit_pressed = keys.just_pressed(KeyCode::KeyF) || gp_enter;
    }
}

// ── T027: poll_vehicle_input ──────────────────────────────────────────────────

/// Update system: reads keyboard/gamepad state and writes a normalised
/// [`VehicleInputState`] snapshot on every occupied vehicle entity.
pub fn poll_vehicle_input(
    mut query: Query<&mut VehicleInputState>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    config: Res<InputConfig>,
    active_pad: Res<ActiveGamepad>,
) {
    let dz = config.deadzone_radius;
    let pad = active_pad.entity.and_then(|e| gamepads.get(e).ok());

    for mut state in &mut query {
        // Throttle (R2 / W)
        let gp_throt = pad
            .map(|g| apply_dz(g.get(GamepadAxis::RightZ).unwrap_or(0.0), dz).max(0.0))
            .unwrap_or(0.0);
        state.throttle = gp_throt.max(keys.pressed(KeyCode::KeyW) as u32 as f32);

        // Brake (L2 / S)
        let gp_brake = pad
            .map(|g| apply_dz(g.get(GamepadAxis::LeftZ).unwrap_or(0.0), dz).max(0.0))
            .unwrap_or(0.0);
        state.brake = gp_brake.max(keys.pressed(KeyCode::KeyS) as u32 as f32);

        // Steer
        let gp_steer = pad
            .map(|g| apply_dz(g.get(GamepadAxis::LeftStickX).unwrap_or(0.0), dz))
            .unwrap_or(0.0);
        let kb_steer =
            (keys.pressed(KeyCode::KeyD) as i32 - keys.pressed(KeyCode::KeyA) as i32) as f32;
        state.steer = if kb_steer.abs() > 0.0 {
            kb_steer
        } else {
            gp_steer
        };

        // Aircraft axes
        state.pitch = pad
            .map(|g| apply_dz(g.get(GamepadAxis::RightStickY).unwrap_or(0.0), dz))
            .unwrap_or(0.0);
        state.roll = pad
            .map(|g| apply_dz(g.get(GamepadAxis::RightStickX).unwrap_or(0.0), dz))
            .unwrap_or(0.0);

        // Enter/exit
        let gp_enter = pad
            .map(|g| g.just_pressed(GamepadButton::North))
            .unwrap_or(false);
        state.enter_exit_pressed = keys.just_pressed(KeyCode::KeyF) || gp_enter;
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Applies a symmetric dead-zone: returns `0.0` when `|raw| < dz`.
#[inline]
fn apply_dz(raw: f32, dz: f32) -> f32 {
    if raw.abs() < dz { 0.0 } else { raw }
}
