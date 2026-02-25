//! Camera follow system.
//!
//! Runs in `PostUpdate`, before `TransformSystem::TransformPropagate`.
//! Keeps a designated camera entity trailing the player so the floating-origin
//! plugin can update its `Transform` from `GlobalPosition`. FR-019 / FR-020.

use bevy::prelude::*;

use crate::plugins::floating_origin::components::GlobalPosition;
use crate::plugins::traversal::components::{PassengerOf, PlayerTag};

/// Marker component placed on the main traversal camera entity.
#[derive(Component, Debug, Default, Reflect)]
pub struct TraversalCamera;

// ── T019: camera_follow_system ────────────────────────────────────────────────

/// Post-update system: sets the camera's [`GlobalPosition`] to
/// `player_pos + (0, 1.5, 3.0)` every frame.
///
/// The floating-origin plugin converts `GlobalPosition` → `Transform` just after
/// this system runs, so no manual `Transform` writes are needed here. FR-019.
#[allow(clippy::type_complexity)]
pub fn camera_follow_system(
    player_q: Query<
        &GlobalPosition,
        (
            With<PlayerTag>,
            Without<PassengerOf>,
            Without<TraversalCamera>,
        ),
    >,
    mut camera_q: Query<(&mut GlobalPosition, &mut Transform), With<TraversalCamera>>,
) {
    let Ok(player_pos) = player_q.single() else {
        return;
    };

    // "Up" on the planet surface = direction away from the planet centre.
    let surface_normal = player_pos.0.normalize();

    // Third-person offset: 1.5 m above along surface normal, 3 m behind along
    // a surface-tangent "forward" (world -Z projected onto the tangent plane).
    let world_fwd = bevy::math::DVec3::NEG_Z;
    let tangent_fwd = (world_fwd - surface_normal * world_fwd.dot(surface_normal))
        .try_normalize()
        .unwrap_or(bevy::math::DVec3::X); // fallback at south pole

    let offset = surface_normal * 1.5 - tangent_fwd * 3.0;

    for (mut cam_pos, mut transform) in &mut camera_q {
        cam_pos.0 = player_pos.0 + offset;

        // Rotate the camera to look toward the player (direction = -offset).
        let look_dir = (-offset).normalize().as_vec3();
        let up = surface_normal.as_vec3();
        let up = if up.dot(look_dir).abs() > 0.99 {
            Vec3::X
        } else {
            up
        };
        *transform = Transform::IDENTITY.looking_to(look_dir, up);
    }
}
