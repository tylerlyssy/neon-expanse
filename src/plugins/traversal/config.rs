//! Traversal RON-serialisable config structs.
//!
//! All values that govern movement, physics, and vehicle behaviour are
//! expressed here and read from `assets/` at startup. Hardcoding any
//! tuneable value in Rust source is forbidden (Constitution Principle VIII).

use bevy::math::Vec3;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ── Player ──────────────────────────────────────────────────────────────────

/// All on-foot movement parameters.
///
/// Loaded from `assets/player/locomotion.ron` at startup.
/// Falls back to [`Default`] with a `warn!` if the file is absent or invalid.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct PlayerLocomotionConfig {
    /// Walking speed (m/s). FR-002.
    pub walk_speed_m_s: f32,
    /// Sprinting speed (m/s). FR-002.
    pub sprint_speed_m_s: f32,
    /// Vertical impulse applied on jump (m/s). FR-002.
    pub jump_impulse_m_s: f32,
    /// Gravity acceleration applied per FixedUpdate tick (m/s²). Negative = down.
    pub gravity_m_s2: f32,
    /// Steepest walkable slope angle (degrees). FR-003.
    pub max_slope_angle_deg: f32,
    /// Distance at which enter/exit prompt appears (metres). FR-006.
    pub interact_radius_m: f32,
    /// Player capsule radius (metres).
    pub capsule_radius_m: f32,
    /// Player capsule total height (metres).
    pub capsule_height_m: f32,
    /// Horizontal swimming speed cap (m/s). FR-004 / FR-005.
    pub swim_speed_m_s: f32,
    /// Y-axis height of the flat ocean surface used for swim/buoyancy detection (metres).
    pub sea_level_m: f32,
}

impl Default for PlayerLocomotionConfig {
    fn default() -> Self {
        Self {
            walk_speed_m_s: 5.0,
            sprint_speed_m_s: 10.0,
            jump_impulse_m_s: 6.0,
            gravity_m_s2: -20.0,
            max_slope_angle_deg: 45.0,
            interact_radius_m: 3.0,
            capsule_radius_m: 0.35,
            capsule_height_m: 1.8,
            swim_speed_m_s: 3.0,
            sea_level_m: 0.0,
        }
    }
}

// ── Wheel -──────────────────────────────────────────────────────────────────

/// Per-wheel raycast-suspension parameters. FR-010.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct WheelConfig {
    /// Offset from vehicle body origin in local space (metres).
    pub local_offset: Vec3,
    /// Length of the downward suspension ray (metres).
    pub ray_length: f32,
    /// Natural (unloaded) suspension length (metres).
    pub rest_length: f32,
    /// Spring stiffness (N/m).
    pub spring_stiffness: f32,
    /// Damper coefficient (N·s/m).
    pub damper: f32,
}

// ── Vehicle configs ──────────────────────────────────────────────────────────

/// Ground vehicle physics parameters. FR-010 / FR-011.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct GroundVehicleConfig {
    /// Vehicle mass (kg).
    pub mass_kg: f32,
    /// Maximum forward speed (m/s).
    pub max_speed_m_s: f32,
    /// Peak drive force (N).
    pub engine_force_n: f32,
    /// Peak braking force (N).
    pub brake_force_n: f32,
    /// Maximum front-axle steer angle (degrees).
    pub max_steer_angle_deg: f32,
    /// Suspension wheel configurations (exactly 4 for MVP).
    pub wheels: Vec<WheelConfig>,
    /// Velocity-delta threshold (m/s) above which `ImpactEvent` is emitted. FR-011.
    pub impact_threshold_m_s: f32,
}

/// Watercraft physics parameters. FR-014 / FR-015.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct WatercraftConfig {
    /// Vessel mass (kg).
    pub mass_kg: f32,
    /// Maximum forward speed (m/s).
    pub max_speed_m_s: f32,
    /// Forward thrust force (N).
    pub thrust_force_n: f32,
    /// Yaw torque for steering (N·m).
    pub turn_torque_n_m: f32,
    /// Approximate submerged hull volume for buoyancy (m³).
    pub buoyancy_volume_m3: f32,
    /// Linear water drag coefficient.
    pub water_drag: f32,
    /// Y-axis height of the flat ocean surface (metres).
    pub sea_level_m: f32,
    /// Desired waterline depth — hull sits this many metres below sea level.
    pub draft_m: f32,
}

/// Helicopter physics parameters. FR-017.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct HelicopterConfig {
    /// Mass (kg).
    pub mass_kg: f32,
    /// Maximum vertical ascent speed (m/s).
    pub max_ascent_m_s: f32,
    /// Maximum lateral translation speed (m/s).
    pub max_lateral_m_s: f32,
    /// Throttle fraction required to hover (0.0–1.0).
    pub hover_throttle: f32,
    /// Body-tilt degrees per unit of stick input.
    pub tilt_sensitivity: f32,
    /// Yaw torque (N·m).
    pub yaw_torque_n_m: f32,
    /// Velocity-delta threshold (m/s) above which `ImpactEvent` is emitted. FR-019.
    pub impact_threshold_m_s: f32,
}

/// Fixed-wing aircraft physics parameters. FR-018.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct FixedWingConfig {
    /// Mass (kg).
    pub mass_kg: f32,
    /// Maximum engine thrust (N).
    pub max_thrust_n: f32,
    /// Aerodynamic lift coefficient (dimensionless).
    pub lift_coefficient: f32,
    /// Aerodynamic drag coefficient (dimensionless).
    pub drag_coefficient: f32,
    /// Wing reference area (m²).
    pub wing_area_m2: f32,
    /// Speed below which lift drops to zero (m/s). FR-018.
    pub stall_speed_m_s: f32,
    /// Maximum pitch control torque (N·m).
    pub max_pitch_torque_n_m: f32,
    /// Maximum roll control torque (N·m).
    pub max_roll_torque_n_m: f32,
    /// Maximum yaw control torque (N·m).
    pub max_yaw_torque_n_m: f32,
    /// Velocity-delta threshold (m/s) above which `ImpactEvent` is emitted. FR-019.
    pub impact_threshold_m_s: f32,
}

/// Sum type discriminating all vehicle physics parameter sets.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum VehiclePhysicsConfig {
    /// Wheeled ground vehicle.
    GroundVehicle(GroundVehicleConfig),
    /// Floating watercraft.
    Watercraft(WatercraftConfig),
    /// Rotary-wing aircraft.
    Helicopter(HelicopterConfig),
    /// Fixed-wing aircraft.
    FixedWing(FixedWingConfig),
}

/// Top-level vehicle definition loaded from a per-vehicle RON file. FR-020.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct VehicleConfig {
    /// Display name (informational only).
    pub name: String,
    /// Physics variant and parameters.
    pub physics: VehiclePhysicsConfig,
}

// ── World spawns ─────────────────────────────────────────────────────────────

/// One vehicle placement entry. FR-026.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSpawnEntry {
    /// Filename relative to `assets/vehicles/`, e.g. `"car.ron"`.
    pub config: String,
    /// Global world position (f64 for floating-origin precision).
    pub position: (f64, f64, f64),
    /// Optional yaw rotation in degrees (ground vehicles / watercraft).
    pub yaw_deg: Option<f32>,
}

/// List of vehicle placements loaded from `assets/vehicles/world_spawns.ron`. FR-026.
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct WorldSpawnList {
    /// All vehicle spawn entries for this session.
    pub entries: Vec<VehicleSpawnEntry>,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locomotion_config_default_values() {
        let cfg = PlayerLocomotionConfig::default();
        assert_eq!(cfg.walk_speed_m_s, 5.0);
        assert_eq!(cfg.sprint_speed_m_s, 10.0);
        assert!(cfg.gravity_m_s2 < 0.0, "gravity must be negative");
        assert!(cfg.max_slope_angle_deg > 0.0);
        assert!(cfg.swim_speed_m_s > 0.0);
    }

    #[test]
    fn test_locomotion_config_round_trip() {
        let orig = PlayerLocomotionConfig::default();
        let serialised = ron::to_string(&orig).expect("serialise failed");
        let deser: PlayerLocomotionConfig = ron::from_str(&serialised).expect("deserialise failed");
        assert_eq!(deser.walk_speed_m_s, orig.walk_speed_m_s);
        assert_eq!(deser.swim_speed_m_s, orig.swim_speed_m_s);
    }

    #[test]
    fn test_world_spawn_list_round_trip() {
        let list = WorldSpawnList { entries: vec![] };
        let s = ron::to_string(&list).expect("serialise");
        let d: WorldSpawnList = ron::from_str(&s).expect("deserialise");
        assert_eq!(d.entries.len(), 0);
    }
}
