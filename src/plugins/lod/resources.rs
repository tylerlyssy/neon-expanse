//! LOD settings resource.

use bevy::prelude::*;

/// Global LOD distance thresholds in metres.
///
/// `thresholds[0]` = High→Medium boundary
/// `thresholds[1]` = Medium→Low boundary
/// `thresholds[2]` = Low→Impostor boundary
///
/// Values are calibrated for a 12,742 km diameter planet with the
/// viewer near the surface. They will be refined in the Voxel World
/// Engine spec.
#[derive(Resource, Debug, Clone, Reflect)]
pub struct LodSettings {
    /// Distance thresholds (metres) for level transitions.
    pub thresholds: [f32; 3],
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            thresholds: [1_000.0, 10_000.0, 100_000.0],
        }
    }
}
