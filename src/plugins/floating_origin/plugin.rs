//! FloatingOriginPlugin — DVec3 global coordinate system.

use bevy::{prelude::*, transform::TransformSystems};

use super::{
    components::{FloatingOrigin, GlobalPosition, TestOriginMarker},
    resources::OriginFrame,
    systems::{spawn_test_scene, sync_transforms, update_origin_frame},
};

/// Bevy plugin that implements the floating-origin coordinate system.
///
/// # What it does
/// - Registers [`GlobalPosition`], [`FloatingOrigin`], [`TestOriginMarker`]
///   components.
/// - Inserts [`OriginFrame`] resource (default: world origin).
/// - Spawns the camera + test sphere + point light at ~6,000 km in
///   `Startup`.
/// - Each frame:
///   - `PreUpdate`: updates [`OriginFrame`] from the [`FloatingOrigin`]
///     entity's [`GlobalPosition`].
///   - `PostUpdate` (before `TransformSystem::TransformPropagate`): syncs every
///     entity's `Transform.translation` from its `GlobalPosition`.
///
/// # Invariant
/// Any system that moves a world entity MUST write to [`GlobalPosition`],
/// never to [`Transform`]. `Transform` is owned by this plugin.
pub struct FloatingOriginPlugin;

impl Plugin for FloatingOriginPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<OriginFrame>()
            // Register component types for reflection (required for Bevy scenes / RON)
            .register_type::<GlobalPosition>()
            .register_type::<FloatingOrigin>()
            .register_type::<TestOriginMarker>()
            // Startup: spawn test scene
            .add_systems(Startup, spawn_test_scene)
            // PreUpdate: refresh origin frame
            .add_systems(PreUpdate, update_origin_frame)
            // PostUpdate: sync Transform from GlobalPosition (before propagation)
            .add_systems(
                PostUpdate,
                sync_transforms.before(TransformSystems::Propagate),
            );
    }
}
