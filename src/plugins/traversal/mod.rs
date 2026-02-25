//! Traversal plugin — on-foot locomotion, vehicles, enter/exit lifecycle.
//!
//! Owns: `PlayerLocomotionState`, `VehicleTag`, all vehicle physics systems,
//! and the enter/exit lifecycle. Populated in spec `003-basic-traversal`.

use avian3d::prelude::PhysicsSystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

pub mod components;
pub mod config;
pub mod events;
pub mod resources;
pub mod systems;

pub use components::*;
pub use config::*;
pub use events::*;
pub use resources::*;

use systems::{
    aircraft::{fixed_wing_system, helicopter_system},
    camera_follow::camera_follow_system,
    enter_exit::{
        apply_enter_vehicle, apply_exit_vehicle, enter_exit_vehicle, update_enter_exit_prompts,
    },
    ground_vehicle::ground_vehicle_system,
    impact::{
        cache_pre_impact_velocity, impact_detection_system, setup_vehicle_collision_tracking,
    },
    input::{poll_on_foot_input, poll_vehicle_input},
    locomotion::locomotion_system,
    lod::vehicle_lod_system,
    spawn::{
        activate_traversal_camera, spawn_player, startup_load_configs, startup_spawn_vehicles,
    },
    watercraft::watercraft_system,
};

/// Bevy plugin for all traversal gameplay: on-foot locomotion, vehicle
/// entry/exit, ground vehicles, watercraft, and aircraft.
pub struct TraversalPlugin;

impl Plugin for TraversalPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── Resources ────────────────────────────────────────────────────
            .init_resource::<CurrentVehicle>()
            .init_resource::<VehicleRegistry>()
            .insert_resource(WorldSpawnList::default())
            // ── Messages (buffered events) ────────────────────────────────────
            .add_message::<ImpactEvent>()
            .add_message::<EnterVehicleEvent>()
            .add_message::<ExitVehicleEvent>()
            // ── Reflect registration ──────────────────────────────────────────
            .register_type::<PlayerTag>()
            .register_type::<PlayerLocomotionState>()
            .register_type::<OnFootInputState>()
            .register_type::<VehicleInputState>()
            .register_type::<VehicleTag>()
            .register_type::<EnterExitProximity>()
            .register_type::<CurrentVehicle>()
            .register_type::<PlayerLocomotionConfig>()
            .register_type::<EnterExitHintMarker>()
            // ── Startup ───────────────────────────────────────────────────────
            // T012: load configs first, then spawn entities that depend on them.
            .add_systems(
                Startup,
                (
                    startup_load_configs,
                    spawn_player.after(startup_load_configs),
                ),
            )
            // T024: spawn vehicle entities after configs are loaded.
            .add_systems(
                PostStartup,
                (
                    startup_spawn_vehicles,
                    // Transfer FloatingOrigin from orbital camera to traversal camera.
                    // Must run after VoxelWorldPlugin::reposition_origin_camera_to_orbit.
                    activate_traversal_camera.after(startup_spawn_vehicles),
                ),
            )
            // ── Update (input polling + enter/exit + LOD) ─────────────────────
            .add_systems(
                Update,
                (
                    // T017: on-foot input snapshot.
                    poll_on_foot_input,
                    // T025: vehicle input snapshot.
                    poll_vehicle_input,
                    // T026: proximity refresh for enter/exit prompts.
                    update_enter_exit_prompts.after(poll_on_foot_input),
                    // T026: enter/exit request detection.
                    enter_exit_vehicle.after(update_enter_exit_prompts),
                    // T026: apply lifecycle changes.
                    apply_enter_vehicle.after(enter_exit_vehicle),
                    apply_exit_vehicle.after(enter_exit_vehicle),
                    // T040: vehicle LOD visibility.
                    vehicle_lod_system,
                    // T028: ensure new vehicles get impact-tracking components.
                    setup_vehicle_collision_tracking,
                ),
            )
            // ── FixedUpdate (physics, before solver) ──────────────────────────
            .add_systems(
                FixedUpdate,
                (
                    // T018: player locomotion before Avian solver.
                    locomotion_system.before(PhysicsSystems::StepSimulation),
                    // T027: ground vehicle suspension + drive.
                    ground_vehicle_system.before(PhysicsSystems::StepSimulation),
                    // T032: watercraft buoyancy + thrust.
                    watercraft_system.before(PhysicsSystems::StepSimulation),
                    // T037: helicopter hover + translation.
                    helicopter_system.before(PhysicsSystems::StepSimulation),
                    // T038: fixed-wing thrust + aerodynamics.
                    fixed_wing_system.before(PhysicsSystems::StepSimulation),
                    // T028 (pre-step): cache velocity before Avian modifies it.
                    cache_pre_impact_velocity.before(PhysicsSystems::StepSimulation),
                ),
            )
            // ── FixedUpdate (physics, after solver) ───────────────────────────
            .add_systems(
                FixedUpdate,
                // T028 (post-step): detect impacts from CollisionStart messages.
                impact_detection_system.after(PhysicsSystems::StepSimulation),
            )
            // ── PostUpdate (camera) ───────────────────────────────────────────
            // T019: camera must be placed before TransformPropagate reads GlobalPosition.
            .add_systems(
                PostUpdate,
                camera_follow_system.before(TransformSystems::Propagate),
            );
    }
}
