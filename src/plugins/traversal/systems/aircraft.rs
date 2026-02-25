//! Helicopter and fixed-wing aircraft systems.
//!
//! `helicopter_system` (T037):
//!   - Computes hover lift = mass * 9.81, scales by throttle / hover_throttle.
//!   - Applies lateral tilt forces from steer/pitch inputs.
//!   - Applies yaw torque from yaw axis.
//!
//! `fixed_wing_system` (T038):
//!   - Applies forward thrust = max_thrust * throttle.
//!   - If airspeed ≥ stall_speed: applies aerodynamic lift and drag.
//!   - Applies pitch, roll, yaw torques.
//!
//! Both run `FixedUpdate` before `PhysicsSystems::StepSimulation`.

use avian3d::dynamics::rigid_body::forces::ReadRigidBodyForces;
use avian3d::prelude::{Forces, RigidBody, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::plugins::traversal::{
    components::{OccupiedBy, VehicleInputState, VehicleTag},
    config::{FixedWingConfig, HelicopterConfig},
};

const GRAVITY: f32 = 9.81;
/// Sea-level air density (kg/m³).
const AIR_DENSITY: f32 = 1.225;

// ── T037: Helicopter ─────────────────────────────────────────────────────────

/// `FixedUpdate` (before `PhysicsSystems::StepSimulation`).
///
/// Hover lift, lateral tilt translation, and yaw for rotary-wing aircraft. FR-017.
#[allow(clippy::type_complexity)]
pub fn helicopter_system(
    mut heli_q: Query<
        (&Transform, &HelicopterConfig, &VehicleInputState, Forces),
        (With<VehicleTag>, With<OccupiedBy>, With<RigidBody>),
    >,
) {
    for (transform, cfg, input, mut forces) in &mut heli_q {
        let local_up = transform.rotation * Vec3::Y;
        let local_right = transform.rotation * Vec3::X;
        let local_forward = transform.rotation * Vec3::NEG_Z;

        // ── Lift ──────────────────────────────────────────────────────────────
        let hover_lift = cfg.mass_kg * GRAVITY;
        // Throttle linearly scales lift; at hover_throttle we match gravity.
        let lift_scale = if cfg.hover_throttle > 0.0 {
            input.throttle / cfg.hover_throttle
        } else {
            input.throttle
        };
        let lift_force = hover_lift * lift_scale;
        forces.apply_force(local_up * lift_force);

        // ── Lateral translation via body tilt ─────────────────────────────────
        // steer → left/right (roll tilt → lateral force)
        let lateral_force = local_right * cfg.tilt_sensitivity * input.steer * cfg.mass_kg;
        forces.apply_force(lateral_force);

        // pitch input → forward/backward translation
        let fwd_force = local_forward * cfg.tilt_sensitivity * (-input.pitch) * cfg.mass_kg;
        forces.apply_force(fwd_force);

        // ── Yaw torque ─────────────────────────────────────────────────────────
        if input.roll.abs() > 0.01 {
            // Roll axis on controller maps to yaw for helicopters.
            forces.apply_torque(Vec3::Y * cfg.yaw_torque_n_m * input.roll);
        }
    }
}

// ── T038: Fixed-wing ─────────────────────────────────────────────────────────

/// `FixedUpdate` (before `PhysicsSystems::StepSimulation`).
///
/// Forward thrust, aerodynamic lift and drag above stall speed, and 3-axis
/// attitude control torques for fixed-wing aircraft. FR-018.
#[allow(clippy::type_complexity)]
pub fn fixed_wing_system(
    mut plane_q: Query<
        (&Transform, &FixedWingConfig, &VehicleInputState, Forces),
        (With<VehicleTag>, With<OccupiedBy>, With<RigidBody>),
    >,
) {
    for (transform, cfg, input, mut forces) in &mut plane_q {
        let local_up = transform.rotation * Vec3::Y;
        let local_forward = transform.rotation * Vec3::NEG_Z;
        let airspeed = forces.linear_velocity().length();

        // ── Thrust ────────────────────────────────────────────────────────────
        let thrust = local_forward * cfg.max_thrust_n * input.throttle;
        forces.apply_force(thrust);

        // ── Aerodynamic forces (only above stall speed) ───────────────────────
        if airspeed >= cfg.stall_speed_m_s {
            let dynamic_pressure = 0.5 * AIR_DENSITY * airspeed * airspeed;

            // Lift — along local up (perpendicular to wings).
            let lift = dynamic_pressure * cfg.lift_coefficient * cfg.wing_area_m2;
            forces.apply_force(local_up * lift);

            // Drag — opposing velocity direction.
            if airspeed > 0.001 {
                let drag_dir = -(forces.linear_velocity() / airspeed);
                let drag = dynamic_pressure * cfg.drag_coefficient * cfg.wing_area_m2;
                forces.apply_force(drag_dir * drag);
            }
        }

        // ── Attitude control torques ──────────────────────────────────────────
        // Pitch (nose up/down) — right stick Y.
        if input.pitch.abs() > 0.01 {
            let pitch_axis = transform.rotation * Vec3::X; // local right = pitch axis
            forces.apply_torque(pitch_axis * cfg.max_pitch_torque_n_m * input.pitch);
        }

        // Roll (bank) — right stick X.
        if input.roll.abs() > 0.01 {
            forces.apply_torque(local_forward * cfg.max_roll_torque_n_m * input.roll);
        }

        // Yaw (rudder) — left stick X / steer.
        if input.steer.abs() > 0.01 {
            forces.apply_torque(Vec3::Y * cfg.max_yaw_torque_n_m * input.steer);
        }
    }
}
