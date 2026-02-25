//! Watercraft buoyancy and propulsion system.
//!
//! Each `FixedUpdate` tick (before `PhysicsSystems::StepSimulation`):
//!  1. Compute submerged depth — how far hull centre is below `sea_level_m`.
//!  2. Apply upward buoyancy force proportional to submerged volume.
//!  3. Apply water drag opposing linear velocity.
//!  4. Apply forward thrust scaled by throttle and clamped to `max_speed_m_s`.
//!  5. Apply yaw torque from steering input.

use crate::plugins::floating_origin::components::GlobalPosition;
use avian3d::dynamics::rigid_body::forces::ReadRigidBodyForces;
use avian3d::prelude::{Forces, RigidBody, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::plugins::traversal::{
    components::{OccupiedBy, VehicleInputState, VehicleTag},
    config::WatercraftConfig,
};

/// Water density (kg/m³) — salt water.
const WATER_DENSITY: f32 = 1025.0;
const GRAVITY: f32 = 9.81;

/// Runs `FixedUpdate` before `PhysicsSystems::StepSimulation`.
///
/// Processes buoyancy, drag, thrust, and steering for every occupied watercraft.
/// FR-014 / FR-015.
#[allow(clippy::type_complexity)]
pub fn watercraft_system(
    mut craft_q: Query<
        (
            &GlobalPosition,
            &Transform,
            &WatercraftConfig,
            &VehicleInputState,
            Forces,
        ),
        (With<VehicleTag>, With<OccupiedBy>, With<RigidBody>),
    >,
) {
    for (global_pos, transform, cfg, input, mut forces) in &mut craft_q {
        let pos_y = global_pos.0.y as f32;
        let velocity = forces.linear_velocity(); // Vec3 (avian f32 mode)
        let forward = transform.rotation * Vec3::NEG_Z;

        // ── Buoyancy ─────────────────────────────────────────────────────────
        // Depth of hull centre below sea level.  Positive = submerged.
        let depth_below_sea = cfg.sea_level_m - pos_y;
        // Submerged fraction [0, 1] — clamp at full volume submersion.
        let submerged_fraction = (depth_below_sea / cfg.buoyancy_volume_m3.cbrt()).clamp(0.0, 1.0);

        if submerged_fraction > 0.0 {
            let buoyancy = WATER_DENSITY * GRAVITY * cfg.buoyancy_volume_m3 * submerged_fraction;
            forces.apply_force(Vec3::Y * buoyancy);
        }

        // ── Water drag ────────────────────────────────────────────────────────
        // Linear drag opposes motion; applies only when partially submerged.
        if submerged_fraction > 0.0 {
            let drag = -velocity * cfg.water_drag * submerged_fraction;
            forces.apply_force(drag);
        }

        // ── Thrust ────────────────────────────────────────────────────────────
        let speed = velocity.dot(forward);
        if input.throttle > 0.0 && speed < cfg.max_speed_m_s {
            let thrust = forward * cfg.thrust_force_n * input.throttle;
            forces.apply_force(thrust);
        }

        // ── Steering yaw torque ───────────────────────────────────────────────
        if input.steer.abs() > 0.01 {
            let yaw = Vec3::Y * cfg.turn_torque_n_m * input.steer;
            forces.apply_torque(yaw);
        }
    }
}
