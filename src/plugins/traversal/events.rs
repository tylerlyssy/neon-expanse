//! Traversal ECS messages (buffered events).
//!
//! In Bevy 0.18, buffered event delivery uses `Message` + `MessageWriter`/`MessageReader`.
//! `Event` is reserved for observer-based (immediate) triggers.

use bevy::prelude::*;

/// Emitted when a vehicle collides with an obstacle at or above its configured
/// `impact_threshold_m_s`. FR-012 / FR-019.
#[derive(Message, Debug)]
pub struct ImpactEvent {
    /// The vehicle entity that sustained the impact.
    pub entity: Entity,
    /// Magnitude of the velocity change at impact (m/s).
    pub impact_speed_m_s: f32,
    /// Outward contact normal in world space (direction of the surface hit). FR-012.
    pub contact_normal: Vec3,
}

/// Emitted when a player requests to enter a vehicle.
/// Consumed by `apply_enter_vehicle` to perform the component lifecycle. FR-007.
#[derive(Message, Debug)]
pub struct EnterVehicleEvent {
    /// The player entity initiating entry.
    pub player: Entity,
    /// The target vehicle entity.
    pub vehicle: Entity,
}

/// Emitted when a player requests to exit a vehicle.
/// Consumed by `apply_exit_vehicle` to perform the component lifecycle. FR-008.
#[derive(Message, Debug)]
pub struct ExitVehicleEvent {
    /// The player entity exiting.
    pub player: Entity,
    /// The vehicle entity being vacated.
    pub vehicle: Entity,
}
