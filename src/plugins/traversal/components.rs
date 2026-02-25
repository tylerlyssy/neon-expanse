//! Traversal ECS components.
//!
//! All types here implement [`Component`] and [`Reflect`]. They are registered
//! in [`TraversalPlugin::build`] so they appear in the Bevy inspector.

use bevy::prelude::*;

// ── Marker components ────────────────────────────────────────────────────────

/// Marks the player entity. Used to differentiate it from vehicle or camera entities.
#[derive(Component, Debug, Default, Reflect)]
pub struct PlayerTag;

/// Marker component placed on a UI entity that displays the enter/exit hint.
#[derive(Component, Debug, Default, Reflect)]
pub struct EnterExitHintMarker;

// ── Player runtime state ─────────────────────────────────────────────────────

/// Dynamic locomotion state updated each `FixedUpdate`. FR-002 / FR-004.
#[derive(Component, Debug, Default, Reflect)]
pub struct PlayerLocomotionState {
    /// Accumulated vertical velocity (m/s) — gravity added per tick, zeroed on ground contact.
    pub vertical_velocity: f32,
    /// `true` when the slope beneath the player is at or below `max_slope_angle_deg`.
    pub is_grounded: bool,
    /// `true` for exactly one `FixedUpdate` tick after a jump is initiated.
    pub jumped_this_frame: bool,
    /// `true` when the capsule is more than 50% submerged below `sea_level_m`.
    /// Replaces walking controls with swimming controls and enables buoyancy. FR-004.
    pub is_swimming: bool,
}

// ── Input state components ────────────────────────────────────────────────────

/// Per-frame on-foot input snapshot written by `poll_on_foot_input` in `Update`. FR-027.
#[derive(Component, Debug, Default, Reflect)]
pub struct OnFootInputState {
    /// Normalised movement direction in XZ plane (world space).
    pub move_dir: Vec2,
    /// Normalised look delta from right stick or mouse (horizontal, vertical).
    pub look_delta: Vec2,
    /// Jump button pressed this frame.
    pub jump_pressed: bool,
    /// Sprint button held this frame.
    pub sprint_held: bool,
    /// Enter/exit vehicle button pressed this frame.
    pub enter_exit_pressed: bool,
}

/// Per-frame vehicle input snapshot written by `poll_vehicle_input` in `Update`. FR-027.
#[derive(Component, Debug, Default, Reflect)]
pub struct VehicleInputState {
    /// Throttle axis [0.0, 1.0].
    pub throttle: f32,
    /// Brake axis [0.0, 1.0].
    pub brake: f32,
    /// Steering axis [-1.0, 1.0] (left stick X for ground/water; rudder yaw for aircraft).
    pub steer: f32,
    /// Pitch input [-1.0, 1.0] (right stick Y for aircraft).
    pub pitch: f32,
    /// Roll input [-1.0, 1.0] (right stick X for aircraft).
    pub roll: f32,
    /// Enter/exit vehicle button pressed this frame.
    pub enter_exit_pressed: bool,
}

// ── Vehicle classification ────────────────────────────────────────────────────

/// Discriminates the physics type of a vehicle entity. Used by system run conditions.
#[derive(Component, Debug, Reflect)]
pub enum VehicleTag {
    /// Wheeled land vehicle using raycast suspension.
    GroundVehicle,
    /// Floating watercraft using manual buoyancy.
    Watercraft,
    /// Rotary-wing aircraft.
    Helicopter,
    /// Fixed-wing aircraft.
    FixedWing,
}

// ── Occupancy components ──────────────────────────────────────────────────────

/// Added to the **player** entity while they occupy a vehicle.
/// Absence → on foot; presence → in vehicle. FR-007.
#[derive(Component, Debug, Reflect)]
pub struct PassengerOf(pub Entity);

/// Added to the **vehicle** entity while a player is inside.
/// Absence → parked; presence → occupied. FR-007.
#[derive(Component, Debug, Reflect)]
pub struct OccupiedBy(pub Entity);

// ── Proximity ────────────────────────────────────────────────────────────────

/// Tracks which player entities are currently within `interact_radius_m` of
/// this vehicle. Updated each `Update` frame by `update_enter_exit_prompts`. FR-006.
#[derive(Component, Debug, Default, Reflect)]
pub struct EnterExitProximity {
    /// Player entities currently within enter-radius.
    #[reflect(ignore)]
    pub nearby_players: Vec<Entity>,
}
