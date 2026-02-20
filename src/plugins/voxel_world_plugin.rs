//! Voxel World Engine plugin — empty stub.
//!
//! This plugin will own: chunk streaming, dual marching cubes mesher,
//! layered procedural generation (base sphere → fractal noise → erosion →
//! biome overlays), and the scaled-space / full-detail rendering switch.
//!
//! Populated in spec `002-voxel-planet-engine`.

use bevy::prelude::*;

/// Empty stub for the Voxel World Engine plugin.
///
/// This plugin will own: chunk streaming, dual marching cubes mesher,
/// layered procedural generation, biome overlays, and the
/// scaled-space / full-detail rendering switch.
///
/// Populated in spec `002-voxel-planet-engine`.
pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, _app: &mut App) {
        // Stub: no systems registered yet.
    }
}
