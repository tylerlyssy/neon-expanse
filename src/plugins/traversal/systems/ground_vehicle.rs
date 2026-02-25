//! Ground vehicle raycast-suspension physics system.
//!
//! Each `FixedUpdate` tick (before `PhysicsSystems::StepSimulation`):
//!   1. Per wheel: cast a downward ray from the wheel's world-space origin.
//!   2. If the ray hits ground within `ray_length`, compute spring compression
//!      and apply a spring+damper impulse upward at the wheel world-point.
//!   3. Apply drive force along the vehicle's forward axis (throttle).
//!   4. Apply braking force opposing linear velocity.
//!   5. Apply yaw torque for steering.

use crate::plugins::floating_origin::components::GlobalPosition;
use avian3d::dynamics::rigid_body::forces::ReadRigidBodyForces;
use avian3d::prelude::{Forces, RigidBody, SpatialQuery, SpatialQueryFilter, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::plugins::traversal::{
    components::{OccupiedBy, VehicleInputState, VehicleTag},
    config::GroundVehicleConfig,
};

/// Runs `FixedUpdate` before `PhysicsSystems::StepSimulation`.
///
/// Applies raycast-suspension spring/damper forces, drive force, braking force,
/// and steering yaw torque to every occupied ground vehicle. FR-010 / FR-011.
#[allow(clippy::type_complexity)]
pub fn ground_vehicle_system(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut vehicle_q: Query<
        (
            Entity,
            &GlobalPosition,
            &Transform,
            &GroundVehicleConfig,
            &VehicleInputState,
            Forces,
        ),
        (With<VehicleTag>, With<OccupiedBy>, With<RigidBody>),
    >,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (entity, global_pos, transform, cfg, input, mut forces) in &mut vehicle_q {
        let body_pos = global_pos.0.as_vec3();
        let rotation = transform.rotation;
        let forward = rotation * Vec3::NEG_Z;
        let velocity = forces.linear_velocity(); // Vec3

        // Exclude the vehicle itself from raycasts.
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);

        // ── Suspension (per wheel) ───────────────────────────────────────────
        let mut grounded_wheels: u32 = 0;

        for wheel in &cfg.wheels {
            // World space wheel origin.
            let wheel_world = body_pos + rotation * wheel.local_offset;

            // Cast a ray straight down from the wheel origin.
            if let Some(hit) =
                spatial_query.cast_ray(wheel_world, Dir3::NEG_Y, wheel.ray_length, true, &filter)
            {
                grounded_wheels += 1;
                let hit_dist = hit.distance;
                let compression = wheel.rest_length - hit_dist;

                // Relative velocity of the vehicle at the wheel attachment point.
                let wheel_vel_y = velocity.y; // simplified — ignore angular contribution

                // Spring + damper force (upward).
                let spring_force =
                    compression * wheel.spring_stiffness - wheel_vel_y * wheel.damper;
                let spring_force = spring_force.max(0.0); // Suspension can only push, not pull.

                // Apply as a force at the wheel world point (generates body torque for lean).
                let world_point = wheel_world - Vec3::Y * hit_dist;
                forces.apply_force_at_point(Vec3::Y * spring_force, world_point);
            }
        }

        // Only apply drive/brake/steer when at least one wheel is grounded.
        if grounded_wheels == 0 {
            continue;
        }

        let speed = velocity.dot(forward);

        // ── Drive force ──────────────────────────────────────────────────────
        if input.throttle > 0.0 && speed < cfg.max_speed_m_s {
            let drive = forward * cfg.engine_force_n * input.throttle;
            forces.apply_force(drive);
        }

        // ── Brake force ──────────────────────────────────────────────────────
        if input.brake > 0.0 {
            let horiz_vel = Vec3::new(velocity.x, 0.0, velocity.z);
            if horiz_vel.length_squared() > 0.001 {
                let brake = -horiz_vel.normalize() * cfg.brake_force_n * input.brake;
                forces.apply_force(brake);
            }
        }

        // ── Steering yaw torque ──────────────────────────────────────────────
        if input.steer.abs() > 0.01 {
            // Torque scales with speed so low-speed turns feel responsive.
            let speed_factor = (speed.abs() / cfg.max_speed_m_s).clamp(0.1, 1.0);
            let steer_rad = cfg.max_steer_angle_deg.to_radians();
            let yaw_torque = Vec3::Y * steer_rad * cfg.engine_force_n * input.steer * speed_factor;
            forces.apply_torque(yaw_torque);
        }
    }
}
