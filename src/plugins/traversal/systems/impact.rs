//! Impact detection system — emits ImpactEvent on collision above threshold.
//!
//! Strategy (two-system approach):
//!  1. `cache_pre_impact_velocity` — runs *before* `PhysicsSystems::StepSimulation`,
//!     stores the current `LinearVelocity` magnitude into `PreImpactSpeed`.
//!  2. `impact_detection_system` — runs *after* `PhysicsSystems::StepSimulation`,
//!     reads `CollisionStart` messages; computes speed delta = `pre_speed - post_speed`;
//!     emits `ImpactEvent` when delta exceeds the vehicle's configured threshold. FR-011.

use avian3d::prelude::{CollisionEventsEnabled, CollisionStart, LinearVelocity};
use bevy::prelude::*;

use crate::plugins::traversal::{
    components::VehicleTag,
    config::{FixedWingConfig, GroundVehicleConfig, HelicopterConfig, WatercraftConfig},
    events::ImpactEvent,
};

/// Cache a vehicle's linear speed each `FixedUpdate` tick *before* the physics
/// step runs, so the impact system can compute the velocity delta afterwards.
#[derive(Component, Debug, Default)]
pub struct PreImpactSpeed(pub f32);

/// Cache the linear speed of every vehicle entity before the physics step.
/// Runs `FixedUpdate` **before** `PhysicsSystems::StepSimulation`.
pub fn cache_pre_impact_velocity(
    mut vehicle_q: Query<(&LinearVelocity, &mut PreImpactSpeed), With<VehicleTag>>,
) {
    for (lin_vel, mut cache) in &mut vehicle_q {
        cache.0 = lin_vel.0.length();
    }
}

// ── Helper: per-vehicle impact threshold ─────────────────────────────────────

fn impact_threshold(
    entity: Entity,
    ground_q: &Query<&GroundVehicleConfig>,
    water_q: &Query<&WatercraftConfig>,
    heli_q: &Query<&HelicopterConfig>,
    fixed_q: &Query<&FixedWingConfig>,
) -> Option<f32> {
    if let Ok(cfg) = ground_q.get(entity) {
        return Some(cfg.impact_threshold_m_s);
    }
    if let Ok(cfg) = heli_q.get(entity) {
        return Some(cfg.impact_threshold_m_s);
    }
    if let Ok(cfg) = fixed_q.get(entity) {
        return Some(cfg.impact_threshold_m_s);
    }
    // WatercraftConfig has no impact_threshold_m_s in the spec; skip.
    let _ = water_q;
    None
}

/// Read `CollisionStart` messages, compute speed delta at impact, and emit
/// `ImpactEvent` when the delta exceeds the vehicle's configured threshold.
/// Runs `FixedUpdate` **after** `PhysicsSystems::StepSimulation`.
pub fn impact_detection_system(
    mut collision_reader: MessageReader<CollisionStart>,
    vehicle_q: Query<(&LinearVelocity, &PreImpactSpeed), With<VehicleTag>>,
    ground_cfg_q: Query<&GroundVehicleConfig>,
    water_cfg_q: Query<&WatercraftConfig>,
    heli_cfg_q: Query<&HelicopterConfig>,
    fixed_cfg_q: Query<&FixedWingConfig>,
    mut impact_writer: MessageWriter<ImpactEvent>,
) {
    for collision in collision_reader.read() {
        // Check each body to see if it is a vehicle with a threshold.
        for &body in [collision.body1, collision.body2].iter().flatten() {
            let Some(threshold) =
                impact_threshold(body, &ground_cfg_q, &water_cfg_q, &heli_cfg_q, &fixed_cfg_q)
            else {
                continue;
            };

            let Ok((post_vel, pre_cache)) = vehicle_q.get(body) else {
                continue;
            };

            let post_speed = post_vel.0.length();
            let delta = (pre_cache.0 - post_speed).abs();

            if delta >= threshold {
                impact_writer.write(ImpactEvent {
                    entity: body,
                    impact_speed_m_s: delta,
                    contact_normal: Vec3::Y, // CollisionStart has no normal; use up as placeholder.
                });

                warn!(
                    entity = ?body,
                    delta_m_s = delta,
                    threshold,
                    "Vehicle impact detected"
                );
            }
        }
    }
}

/// Ensure new vehicle entities get the `PreImpactSpeed` cache component and
/// `CollisionEventsEnabled` so Avian fires `CollisionStart` messages for them.
pub fn setup_vehicle_collision_tracking(
    mut commands: Commands,
    new_vehicles: Query<Entity, (Added<VehicleTag>, Without<PreImpactSpeed>)>,
) {
    for entity in &new_vehicles {
        commands
            .entity(entity)
            .insert((PreImpactSpeed::default(), CollisionEventsEnabled));
    }
}
