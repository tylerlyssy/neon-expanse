//! Planet initialisation systems.
//!
//! Runs in `Startup` to load configuration, spawn the scaled-space sphere,
//! and (in debug builds) spawn a free camera at orbital altitude for testing.

use bevy::{math::DVec3, prelude::*};

use crate::plugins::{
    core_plugin::WorldSeed,
    floating_origin::components::{FloatingOrigin, GlobalPosition},
    voxel::{
        components::{PlanetCentre, ScaledSpaceMarker},
        config::PlanetConfig,
    },
};

// ── Config loading ────────────────────────────────────────────────────────────

/// Startup system: load `PlanetConfig` from RON and insert as a resource.
///
/// Path resolution order:
/// 1. `NEON_PLANET_CONFIG` environment variable (if set to a valid path).
/// 2. `assets/config/planets/planet.ron` (default).
/// 3. `PlanetConfig::default()` (hard-coded Earth-like fallback — never panics).
///
/// A `warn!` is emitted for any read or deserialise error; the engine continues.
pub fn load_planet_config(mut commands: Commands) {
    let default_path = "assets/config/planets/planet.ron".to_string();
    let path = std::env::var("NEON_PLANET_CONFIG").unwrap_or(default_path);

    let config = std::fs::read_to_string(&path)
        .map_err(|e| {
            warn!(
                "PlanetConfig: could not read '{}': {e} — using defaults",
                path
            );
        })
        .ok()
        .and_then(|src| {
            ron::from_str::<PlanetConfig>(&src)
                .map_err(|e| {
                    warn!(
                        "PlanetConfig: failed to deserialise '{}': {e} — using defaults",
                        path
                    );
                })
                .ok()
        })
        .unwrap_or_default();

    info!(
        "PlanetConfig loaded: radius={:.1} km, {} noise layers, {} biome bands",
        config.radius_km,
        config.noise_layers.len(),
        config.biome.bands.len(),
    );

    commands.insert_resource(config);
}

// ── Planet sphere spawn ───────────────────────────────────────────────────────

/// Startup system: spawn the scaled-space sphere and the planet centre marker.
///
/// Runs **after** `load_planet_config` (enforced by `.after()` in plugin registration).
///
/// Spawns:
/// - A `PlanetCentre` marker entity at `GlobalPosition(DVec3::ZERO)`.
/// - A UV sphere mesh entity with `ScaledSpaceMarker` at the same origin.
pub fn spawn_planet_on_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planet_config: Res<PlanetConfig>,
    world_seed: Res<WorldSeed>,
) {
    let radius_m = (planet_config.radius_km * 1_000.0) as f32;

    // ── Planet centre reference ───────────────────────────────────────────
    commands.spawn((
        Name::new("PlanetCentre"),
        PlanetCentre,
        GlobalPosition(DVec3::ZERO),
        Transform::default(),
        Visibility::Hidden,
    ));

    // ── Scaled-space sphere ───────────────────────────────────────────────
    // UV sphere with 64 longitudinal and 32 latitudinal segments for a smooth
    // appearance from any orbital altitude up to ~500,000 km.
    // Base colour is an approximate biome average; full-detail biome texturing
    // is driven by the vertex shader in future sprints.
    let sphere_mesh = meshes.add(Sphere { radius: radius_m }.mesh().uv(64, 32));

    // Derive a representative surface colour from the first mid-elevation biome band.
    let surface_colour = planet_config
        .biome
        .bands
        .iter()
        .find(|b| b.height_fraction >= 0.1 && b.height_fraction <= 0.5)
        .map(|b| Color::srgb(b.colour.0, b.colour.1, b.colour.2))
        .unwrap_or(Color::srgb(0.25, 0.55, 0.20));

    let mat = materials.add(StandardMaterial {
        base_color: surface_colour,
        perceptual_roughness: 0.9,
        metallic: 0.0,
        unlit: true, // visible from orbit without directional light
        ..default()
    });

    commands.spawn((
        Name::new("PlanetSphere"),
        ScaledSpaceMarker,
        Mesh3d(sphere_mesh),
        MeshMaterial3d(mat),
        GlobalPosition(DVec3::ZERO),
        Transform::default(),
        Visibility::Visible,
    ));

    info!(
        "Planet sphere spawned: radius={:.0} m, seed={:#018x}",
        radius_m, world_seed.0,
    );
}

// ── Orbital camera placement ──────────────────────────────────────────────────

/// `PostStartup` system: move the `FloatingOrigin` camera to orbital altitude.
///
/// Runs unconditionally (debug **and** release) after all `Startup` systems
/// have finished — guaranteeing that `FloatingOriginPlugin::spawn_test_scene`
/// has already spawned the camera entity.
///
/// The camera is repositioned to 8,000 km along the +Y axis (above the north
/// pole) so it sees the planet from outside its radius (~6,371 km).  Only the
/// [`GlobalPosition`] is written; `FloatingOriginPlugin` syncs `Transform` on
/// the next `PostUpdate`.
///
/// # Why not `spawn_debug_camera`?
/// Spawning a *second* entity with [`FloatingOrigin`] breaks
/// `update_origin_frame`'s `Query::single()` (two entities ⟹ silent
/// `QuerySingleError`), leaving `OriginFrame` stuck at `DVec3::ZERO` so
/// nothing renders.  Instead this system *mutates* the one camera that already
/// exists.
pub fn reposition_origin_camera_to_orbit(
    mut query: Query<(&mut GlobalPosition, &mut Transform, &mut Projection), With<FloatingOrigin>>,
) {
    /// 8,000 km — comfortably outside the default Earth-radius planet (6,371 km).
    const ORBITAL_ALTITUDE_M: f64 = 15_000_000.0;

    if let Ok((mut pos, mut transform, mut projection)) = query.single_mut() {
        // Place the GlobalPosition above the north pole.
        pos.0 = DVec3::new(0.0, ORBITAL_ALTITUDE_M, 0.0);

        // Point the camera toward the planet centre (DVec3::ZERO).
        // sync_transforms will zero out Transform.translation each PostUpdate
        // but preserves rotation, so setting it here is sufficient.
        // Camera is above +Y; planet centre is in the -Y direction; use +Z as up.
        *transform = Transform::IDENTITY.looking_at(Vec3::NEG_Y, Vec3::Z);

        // Extend the far clip plane so the planet sphere fits in the frustum.
        // Default Camera3d far = 1,000 m — the sphere is 8,000,000 m away.
        // Near = 1,000 m (1 km) avoids precision issues at orbital altitude.
        *projection = Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4, // 45° vertical FOV
            near: 1_000.0,                    // 1 km
            far: 20_000_000.0,                // 20,000 km — covers full sphere
            ..default()
        });

        info!(
            "Origin camera repositioned to {:.0} km orbital altitude, far={:.0} km",
            ORBITAL_ALTITUDE_M / 1_000.0,
            20_000.0_f32,
        );
    } else {
        warn!("reposition_origin_camera_to_orbit: no unique FloatingOrigin entity found");
    }
}
