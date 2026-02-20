//! Traversal plugin — empty stub.
//!
//! This plugin will own: PlayerLocomotionComponent, VehicleComponent,
//! WatercraftComponent, AircraftComponent, and all movement systems.
//!
//! Populated in a spec following `002-voxel-planet-engine`.

use bevy::prelude::*;

/// Empty stub for the Traversal plugin.
///
/// This plugin will own: PlayerLocomotionComponent, VehicleComponent,
/// WatercraftComponent, AircraftComponent, and all movement systems.
///
/// Populated in a spec following `002-voxel-planet-engine`.
pub struct TraversalPlugin;

impl Plugin for TraversalPlugin {
    fn build(&self, _app: &mut App) {
        // Stub: no systems registered yet.
    }
}
