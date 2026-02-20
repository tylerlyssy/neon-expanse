//! LodPlugin — distance-based Level of Detail.

use bevy::prelude::*;

use super::{components::LodLevel, resources::LodSettings, systems::update_lod_levels};

/// Bevy plugin that manages LOD levels for all world entities.
///
/// Registers [`LodSettings`] with default thresholds and [`LodLevel`]
/// component. Updates all LOD levels every `PostUpdate` frame.
///
/// No visual LOD transitions occur during Core Foundation because no
/// terrain geometry exists yet. All entities start at level 3 (impostor)
/// and transition to lower levels as the Voxel World Engine introduces
/// renderable geometry.
pub struct LodPlugin;

impl Plugin for LodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LodSettings>()
            .register_type::<LodLevel>()
            .register_type::<LodSettings>()
            .add_systems(PostUpdate, update_lod_levels);
    }
}
