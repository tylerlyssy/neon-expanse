//! Player and vehicle spawn systems.

use avian3d::prelude::*;
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::plugins::floating_origin::components::{FloatingOrigin, GlobalPosition};
use crate::plugins::traversal::components::{
    EnterExitProximity, OnFootInputState, PlayerLocomotionState, PlayerTag, VehicleInputState,
    VehicleTag,
};
use crate::plugins::traversal::config::{
    PlayerLocomotionConfig, VehicleConfig, VehiclePhysicsConfig, WorldSpawnList,
};
use crate::plugins::traversal::resources::VehicleRegistry;
use crate::plugins::traversal::systems::camera_follow::TraversalCamera;

// ── T012: startup_load_configs ────────────────────────────────────────────────

/// Startup system: loads all traversal config files from `assets/`.
///
/// Reads:
/// - `assets/player/locomotion.ron` → inserts [`PlayerLocomotionConfig`]
/// - `assets/vehicles/*.ron`        → builds [`VehicleRegistry`]
/// - `assets/vehicles/world_spawns.ron` → inserts [`WorldSpawnList`]
///
/// Uses `warn!` + default fallback on any missing or malformed file so the
/// session always continues. FR-021 / FR-022 / FR-026.
pub fn startup_load_configs(mut commands: Commands) {
    // ── PlayerLocomotionConfig ───────────────────────────────────────────────
    let locomotion_cfg = load_ron_or_default::<PlayerLocomotionConfig>(
        "assets/player/locomotion.ron",
        "PlayerLocomotionConfig",
    );
    commands.insert_resource(locomotion_cfg);

    // ── VehicleRegistry ──────────────────────────────────────────────────────
    let mut registry = VehicleRegistry::default();
    match std::fs::read_dir("assets/vehicles") {
        Err(e) => {
            warn!("Could not read assets/vehicles/: {e} — no vehicles loaded");
        }
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_owned();
                if file_name == "world_spawns.ron" || !file_name.ends_with(".ron") {
                    continue;
                }
                let path_str = path.to_string_lossy().to_string();
                match std::fs::read_to_string(&path_str) {
                    Err(e) => {
                        warn!("Could not read {path_str}: {e} — skipping vehicle");
                    }
                    Ok(contents) => match ron::from_str::<VehicleConfig>(&contents) {
                        Err(e) => {
                            warn!("Could not parse {path_str}: {e} — skipping vehicle");
                        }
                        Ok(cfg) => {
                            info!("Loaded vehicle config: {} ({file_name})", cfg.name);
                            registry.0.push(cfg);
                        }
                    },
                }
            }
        }
    }
    commands.insert_resource(registry);

    // ── WorldSpawnList ───────────────────────────────────────────────────────
    let spawn_list =
        load_ron_or_default::<WorldSpawnList>("assets/vehicles/world_spawns.ron", "WorldSpawnList");
    commands.insert_resource(spawn_list);
}

/// Deserialises a RON resource file, falling back to `Default` with a `warn!`.
fn load_ron_or_default<T>(path: &str, type_name: &str) -> T
where
    T: Default + serde::de::DeserializeOwned,
{
    match std::fs::read_to_string(path) {
        Err(_) => {
            warn!("{type_name}: '{path}' not found — using defaults");
            T::default()
        }
        Ok(contents) => match ron::from_str::<T>(&contents) {
            Err(e) => {
                warn!("{type_name}: failed to parse '{path}': {e} — using defaults");
                T::default()
            }
            Ok(val) => {
                info!("{type_name} loaded from '{path}'");
                val
            }
        },
    }
}

// ── T016: spawn_player ────────────────────────────────────────────────────────

/// Startup system: spawns the player entity.
///
/// Reads the spawn coordinate from `PlanetConfig.spawn_position` when that
/// field is available (deferred to a future spec). Falls back to
/// `DVec3::new(0.0, 2.0, 0.0)` with a `warn!`. FR-001.
pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<PlayerLocomotionConfig>,
    planet_cfg: Option<Res<crate::plugins::voxel::config::PlanetConfig>>,
) {
    // Spawn the player just above the planet surface at the north pole — the
    // same hemisphere `reposition_origin_camera_to_orbit` looks down from.
    // Use the loaded PlanetConfig radius if available; fall back to the default
    // Earth-scale 6,371 km so the player is never spawned inside the sphere.
    let radius_m = planet_cfg
        .as_ref()
        .map(|c| c.radius_km * 1_000.0)
        .unwrap_or(6_371_000.0);
    // 50 m clearance above the mathematically-smooth sphere surface.
    let spawn_pos = DVec3::new(0.0, radius_m + 50.0, 0.0);
    info!("spawn_player: surface radius = {radius_m:.0} m → spawning at {spawn_pos}");

    let half_height = (config.capsule_height_m * 0.5).max(config.capsule_radius_m);

    commands.spawn((
        (
            Name::new("Player"),
            PlayerTag,
            PlayerLocomotionState::default(),
            OnFootInputState::default(),
            GlobalPosition(spawn_pos),
            RigidBody::Kinematic,
            Collider::capsule(config.capsule_radius_m, half_height),
        ),
        (
            LockedAxes::ROTATION_LOCKED,
            GravityScale(0.0), // manual gravity applied in locomotion_system
            LinearVelocity::default(),
            Visibility::default(),
            Transform::default(),
            GlobalTransform::default(),
            SceneRoot(asset_server.load("models/player.glb#Scene0")),
        ),
    ));

    // Spawn the traversal camera just behind/above the player's spawn point.
    // `activate_traversal_camera` (PostStartup) will transfer `FloatingOrigin`
    // from the orbital debug camera to this entity.
    // Offset: +1.5 m above, +3 m behind (along +Z in world space; camera_follow
    // applies this offset every frame anyway, but we set a sensible initial pos).
    let cam_pos = spawn_pos + DVec3::new(0.0, 3.5, 3.0);
    commands.spawn((
        Name::new("TraversalCamera"),
        TraversalCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4, // 45° vertical FOV
            near: 0.1,                        // 10 cm near clip
            far: 10_000.0,                    // 10 km far clip — sees terrain around player
            ..default()
        }),
        GlobalPosition(cam_pos),
        Transform::default(),
        GlobalTransform::default(),
    ));

    info!("Player spawned at {spawn_pos}");
}

// ── activate_traversal_camera ────────────────────────────────────────────────

/// `PostStartup` system: transfers [`FloatingOrigin`] from the orbital debug
/// camera to the [`TraversalCamera`] entity, then deactivates the orbital camera.
///
/// The floating-origin plugin requires **exactly one** `FloatingOrigin` entity for
/// `update_origin_frame`'s `query.single()` to succeed.  The orbital camera spawned
/// by `FloatingOriginPlugin` holds that marker; this system moves it so the
/// traversal camera drives the coordinate origin near the player.
#[allow(clippy::type_complexity)]
pub fn activate_traversal_camera(
    mut commands: Commands,
    orbital_q: Query<
        Entity,
        (
            With<Camera3d>,
            With<FloatingOrigin>,
            Without<TraversalCamera>,
        ),
    >,
    traversal_q: Query<Entity, With<TraversalCamera>>,
) {
    // ── Deactivate the orbital camera ────────────────────────────────────────
    for orbital in &orbital_q {
        commands
            .entity(orbital)
            .remove::<FloatingOrigin>()
            .insert(Camera {
                is_active: false,
                ..default()
            });
        info!("activate_traversal_camera: orbital camera deactivated");
    }

    // ── Promote the traversal camera to FloatingOrigin ───────────────────────
    for trav in &traversal_q {
        commands.entity(trav).insert(FloatingOrigin);
        info!("activate_traversal_camera: TraversalCamera is now FloatingOrigin");
    }
}

// ── T024: startup_spawn_vehicles ─────────────────────────────────────────────

/// Post-startup system: spawns all vehicles listed in [`WorldSpawnList`].
///
/// Iterates entries, matches each to a [`VehicleConfig`] in [`VehicleRegistry`],
/// and spawns the appropriate bundle. FR-026.
pub fn startup_spawn_vehicles(
    mut commands: Commands,
    registry: Res<VehicleRegistry>,
    spawn_list: Res<WorldSpawnList>,
) {
    for entry in &spawn_list.entries {
        let Some(cfg) = registry.0.iter().find(|c| {
            // Match by the last path component of the config filename.
            let cfg_name = entry.config.trim_start_matches("vehicles/");
            c.name.to_lowercase().replace(' ', "_") == cfg_name.trim_end_matches(".ron")
                || entry.config.contains(&c.name)
        }) else {
            warn!(
                "startup_spawn_vehicles: no config found for '{}' — skipping",
                entry.config
            );
            continue;
        };

        let pos = DVec3::new(entry.position.0, entry.position.1, entry.position.2);
        let yaw = entry.yaw_deg.unwrap_or(0.0_f32).to_radians();
        let rotation = Quat::from_rotation_y(yaw);

        spawn_vehicle(&mut commands, cfg, pos, rotation);
    }
}

/// Spawns a single vehicle entity from a [`VehicleConfig`] at the given position.
fn spawn_vehicle(commands: &mut Commands, cfg: &VehicleConfig, pos: DVec3, rotation: Quat) {
    let name = cfg.name.clone();
    match &cfg.physics {
        VehiclePhysicsConfig::GroundVehicle(gv) => {
            // Split into nested tuple to avoid the 12-element Bundle limit.
            commands.spawn((
                (
                    Name::new(format!("Vehicle:{name}")),
                    VehicleTag::GroundVehicle,
                    gv.clone(),
                    VehicleInputState::default(),
                    EnterExitProximity::default(),
                    GlobalPosition(pos),
                    RigidBody::Dynamic,
                ),
                (
                    Collider::cuboid(1.0, 0.5, 2.0),
                    Mass(gv.mass_kg),
                    LinearVelocity::default(),
                    AngularVelocity::default(),
                    Visibility::default(),
                    Transform::from_translation(Vec3::ZERO).with_rotation(rotation),
                    GlobalTransform::default(),
                ),
            ));
        }
        VehiclePhysicsConfig::Watercraft(wc) => {
            commands.spawn((
                (
                    Name::new(format!("Vehicle:{name}")),
                    VehicleTag::Watercraft,
                    wc.clone(),
                    VehicleInputState::default(),
                    EnterExitProximity::default(),
                    GlobalPosition(pos),
                    RigidBody::Dynamic,
                ),
                (
                    Collider::cuboid(1.5, 0.6, 3.0),
                    Mass(wc.mass_kg),
                    LinearVelocity::default(),
                    AngularVelocity::default(),
                    Visibility::default(),
                    Transform::from_translation(Vec3::ZERO).with_rotation(rotation),
                    GlobalTransform::default(),
                ),
            ));
        }
        VehiclePhysicsConfig::Helicopter(hc) => {
            commands.spawn((
                (
                    Name::new(format!("Vehicle:{name}")),
                    VehicleTag::Helicopter,
                    hc.clone(),
                    VehicleInputState::default(),
                    EnterExitProximity::default(),
                    GlobalPosition(pos),
                    RigidBody::Dynamic,
                ),
                (
                    Collider::cuboid(1.5, 0.8, 2.5),
                    Mass(hc.mass_kg),
                    LinearVelocity::default(),
                    AngularVelocity::default(),
                    GravityScale(1.0),
                    Visibility::default(),
                    Transform::from_translation(Vec3::ZERO).with_rotation(rotation),
                    GlobalTransform::default(),
                ),
            ));
        }
        VehiclePhysicsConfig::FixedWing(fw) => {
            commands.spawn((
                (
                    Name::new(format!("Vehicle:{name}")),
                    VehicleTag::FixedWing,
                    fw.clone(),
                    VehicleInputState::default(),
                    EnterExitProximity::default(),
                    GlobalPosition(pos),
                    RigidBody::Dynamic,
                ),
                (
                    Collider::cuboid(6.0, 0.5, 4.0),
                    Mass(fw.mass_kg),
                    LinearVelocity::default(),
                    AngularVelocity::default(),
                    GravityScale(1.0),
                    Visibility::default(),
                    Transform::from_translation(Vec3::ZERO).with_rotation(rotation),
                    GlobalTransform::default(),
                ),
            ));
        }
    }
    info!("Spawned vehicle '{name}' at {pos}");
}
