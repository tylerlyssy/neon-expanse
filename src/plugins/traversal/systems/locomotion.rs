//! On-foot locomotion system using MoveAndSlide.
//!
//! Runs in `FixedUpdate`, before `PhysicsSystems::StepSimulation`.
//! Applies gravity, jump impulse, walking/swimming velocity, then calls
//! `MoveAndSlide::move_and_slide` and writes the resulting absolute position
//! back into [`GlobalPosition`]. FR-002 / FR-003 / FR-004 / FR-005.
//!
//! ## Precision note
//! `GlobalPosition` stores `DVec3` (f64) for floating-origin accuracy.
//! Avian3d is compiled in f32 mode (`Vector = Vec3`), so we convert:
//! - Input to avian: `global_pos.0.as_vec3()` (DVec3 → Vec3)
//! - Output from avian: `output.position.as_dvec3()` (Vec3 → DVec3)

use avian3d::prelude::{
    Collider, MoveAndSlide, MoveAndSlideConfig, MoveAndSlideHitResponse, SpatialQueryFilter,
};
use bevy::prelude::*;

use crate::plugins::floating_origin::components::GlobalPosition;
use crate::plugins::traversal::components::{
    OnFootInputState, PassengerOf, PlayerLocomotionState, PlayerTag,
};
use crate::plugins::traversal::config::PlayerLocomotionConfig;

// ── T018 / T018a / T018b: locomotion_system ───────────────────────────────────

/// Fixed-update system: applies gravity/buoyancy, jump impulse, and
/// `MoveAndSlide` to every on-foot player entity.
#[allow(clippy::type_complexity)]
pub fn locomotion_system(
    mut player_q: Query<
        (
            Entity,
            &mut GlobalPosition,
            &mut PlayerLocomotionState,
            &OnFootInputState,
            &Collider,
            &Transform,
        ),
        (With<PlayerTag>, Without<PassengerOf>),
    >,
    config: Res<PlayerLocomotionConfig>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (entity, mut global_pos, mut loco_state, input, collider, transform) in &mut player_q {
        // ── T018a/T018b: swimming detection ──────────────────────────────────
        let sea_y = config.sea_level_m as f64;
        loco_state.is_swimming = global_pos.0.y < sea_y;

        // ── Desired horizontal speed ──────────────────────────────────────────
        let speed = if loco_state.is_swimming {
            config.swim_speed_m_s
        } else if input.sprint_held {
            config.sprint_speed_m_s
        } else {
            config.walk_speed_m_s
        };
        let wish_vel_xz = Vec3::new(input.move_dir.x, 0.0, -input.move_dir.y) * speed;

        // ── Vertical velocity ─────────────────────────────────────────────────
        if loco_state.is_swimming {
            let depth = (sea_y - global_pos.0.y) as f32;
            let buoy = depth * 8.0;
            loco_state.vertical_velocity =
                (loco_state.vertical_velocity + buoy * dt).clamp(-10.0, 5.0);
            loco_state.is_grounded = false;
        } else {
            loco_state.vertical_velocity += config.gravity_m_s2 * dt;
            if input.jump_pressed && loco_state.is_grounded {
                loco_state.vertical_velocity = config.jump_impulse_m_s;
                loco_state.jumped_this_frame = true;
            } else {
                loco_state.jumped_this_frame = false;
            }
        }

        // ── Compose velocity (f32 — avian's space) ────────────────────────────
        let velocity = wish_vel_xz + Vec3::Y * loco_state.vertical_velocity;

        // ── MoveAndSlide ──────────────────────────────────────────────────────
        let pos_f32 = global_pos.0.as_vec3(); // DVec3 → Vec3
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);

        let output = move_and_slide.move_and_slide(
            collider,
            pos_f32,
            transform.rotation, // already f32 Quat
            velocity,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &filter,
            |_hit| MoveAndSlideHitResponse::Accept,
        );

        // ── Write position back (Vec3 → DVec3) ───────────────────────────────
        global_pos.0 = output.position.as_dvec3();

        // ── Update grounded flag ──────────────────────────────────────────────
        if !loco_state.is_swimming {
            let is_falling = loco_state.vertical_velocity < -0.1;
            if is_falling && output.projected_velocity.y.abs() < 0.5 {
                loco_state.vertical_velocity = 0.0;
                loco_state.is_grounded = true;
            } else {
                loco_state.is_grounded = loco_state.vertical_velocity.abs() < 0.05;
            }
        }

        // Clamp at sea surface when swimming
        if loco_state.is_swimming {
            let max_y = sea_y + config.capsule_height_m as f64 * 0.5;
            if global_pos.0.y > max_y {
                global_pos.0.y = max_y;
            }
        }
    }
}
