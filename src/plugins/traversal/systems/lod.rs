//! Vehicle LOD (Level-of-Detail) visibility system.
//!
//! Hides vehicle entities that are further than `LOD_MAX_DISTANCE` metres from
//! the camera.  Uses `GlobalPosition` (high-precision double coordinates) to
//! avoid floating-origin precision loss.

use bevy::prelude::*;

use crate::plugins::floating_origin::components::GlobalPosition;
use crate::plugins::traversal::{components::VehicleTag, systems::camera_follow::TraversalCamera};

/// Vehicles beyond this distance are hidden to save render budget.
const LOD_MAX_DISTANCE: f64 = 500.0;

/// `Update` — compare each vehicle's `GlobalPosition` to the camera's.
///
/// Sets `Visibility::Hidden` for vehicles beyond `LOD_MAX_DISTANCE`, and
/// `Visibility::Inherited` (rendering controlled by parent hierarchy) otherwise.
pub fn vehicle_lod_system(
    camera_q: Query<&GlobalPosition, With<TraversalCamera>>,
    mut vehicle_q: Query<(&GlobalPosition, &mut Visibility), With<VehicleTag>>,
) {
    let Ok(cam_pos) = camera_q.single() else {
        return;
    };

    let max_dist_sq = LOD_MAX_DISTANCE * LOD_MAX_DISTANCE;

    for (vehicle_pos, mut visibility) in &mut vehicle_q {
        let dist_sq = (vehicle_pos.0 - cam_pos.0).length_squared();
        *visibility = if dist_sq > max_dist_sq {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
