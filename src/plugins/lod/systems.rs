//! LOD update system.

use bevy::prelude::*;

use crate::plugins::floating_origin::{components::GlobalPosition, resources::OriginFrame};

use super::{components::LodLevel, resources::LodSettings};

/// Computes the [`LodLevel`] for every entity with a [`GlobalPosition`]
/// based on its distance from the current [`OriginFrame`].
///
/// Runs in `PostUpdate`. No visible effect until terrain geometry
/// (introduced in the Voxel World Engine spec) begins querying `LodLevel`.
pub fn update_lod_levels(
    origin: Res<OriginFrame>,
    settings: Res<LodSettings>,
    mut query: Query<(&GlobalPosition, &mut LodLevel)>,
) {
    for (gpos, mut level) in query.iter_mut() {
        // Safe narrowing: distance at planetary scale fits comfortably in f32
        // (max ~20,000 km = 2×10⁷ m; f32 max ~3.4×10³⁸ m).
        let dist = (gpos.0 - origin.position).length() as f32;
        let new_level = if dist < settings.thresholds[0] {
            0
        } else if dist < settings.thresholds[1] {
            1
        } else if dist < settings.thresholds[2] {
            2
        } else {
            3
        };
        if level.0 != new_level {
            level.0 = new_level;
        }
    }
}
