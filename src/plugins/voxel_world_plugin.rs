//! Voxel World Engine plugin for Neon Expanse.
//!
//! Owns: `PlanetConfig` loading, scaled-space sphere, dual marching cubes
//! chunk streaming, Avian3d collision generation, and LOD integration.
//!
//! Populated in spec `002-voxel-planet-engine`.

use bevy::prelude::*;

use crate::plugins::voxel::{
    components::*,
    resources::*,
    systems::{
        collider_sync::collider_syncer,
        mesh_builder::{mesh_builder, setup_voxel_material},
        planet_init::{
            load_planet_config, reposition_origin_camera_to_orbit, spawn_planet_on_startup,
        },
        scaled_space::scaled_space_switcher,
        streaming::{chunk_streamer, chunk_task_poller},
    },
};

/// `Update` system: log `VoxelStats` diagnostics at `debug!` level.
///
/// Registered under `#[cfg(debug_assertions)]` to avoid overhead in release builds.
pub fn log_voxel_stats(stats: Res<VoxelStats>) {
    debug!(
        "VoxelStats — loaded: {} chunks, queued: {}, in-flight: {}, memory: {:.1} MB",
        stats.loaded_chunks, stats.queued_chunks, stats.in_flight_tasks, stats.memory_used_mb
    );
}

/// `Update` system: census every second — prints how many entities of each
/// interesting type exist so we can identify the entity-count leak.
///
/// Debug-assertions only (dev builds).
#[cfg(debug_assertions)]
pub fn entity_census(
    all_q: Query<Entity>,
    chunk_q: Query<Entity, With<VoxelChunk>>,
    task_q: Query<Entity, With<ChunkGenTask>>,
    mesh_q: Query<Entity, With<ChunkMesh>>,
    collider_q: Query<Entity, With<ChunkCollider>>,
    pool: Res<ChunkPool>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;

    let total = all_q.iter().count();
    let chunks = chunk_q.iter().count();
    let tasks = task_q.iter().count();
    let meshed = mesh_q.iter().count();
    let with_collider = collider_q.iter().count();
    let pool_loaded = pool.loaded.len();
    let bytes_mb = pool.bytes_used as f32 / (1024.0 * 1024.0);

    info!(
        "[CENSUS] total={total} | VoxelChunk={chunks} (pool.loaded={pool_loaded}) \
         | ChunkGenTask={tasks} | ChunkMesh={meshed} | ChunkCollider={with_collider} \
         | unknown_other={} | pool_bytes={bytes_mb:.1} MB",
        total.saturating_sub(chunks + tasks + meshed.max(chunks))
    );
}

/// Voxel World Engine plugin.
///
/// Registers all resources, startup systems, and update systems required for
/// the procedural voxel planet: config loading, scaled-space sphere, chunk
/// streaming, DMC meshing, and Avian3d collision generation.
pub struct VoxelWorldPlugin;

impl VoxelWorldPlugin {
    /// Returns a snapshot of current voxel world diagnostics.
    ///
    /// Intended for use by other plugins (e.g., TraversalPlugin) that need to
    /// throttle based on memory budget or in-flight task count.
    pub fn stats(world: &World) -> Option<VoxelStats> {
        world.get_resource::<VoxelStats>().copied()
    }
}

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        // ── Resources ─────────────────────────────────────────────────────
        app.init_resource::<ChunkPool>()
            .init_resource::<StreamingQueue>()
            .init_resource::<ScaledSpaceState>()
            .init_resource::<VoxelStats>();

        // ── Component reflection ──────────────────────────────────────────
        app.register_type::<VoxelChunk>()
            .register_type::<VoxelData>()
            .register_type::<ChunkMesh>()
            .register_type::<ChunkCollider>()
            .register_type::<ScaledSpaceMarker>()
            .register_type::<PlanetCentre>();

        // ── Startup systems ───────────────────────────────────────────────
        // setup_voxel_material must run before mesh_builder (inserts VoxelMaterial resource).
        app.add_systems(
            Startup,
            (
                setup_voxel_material,
                load_planet_config,
                spawn_planet_on_startup
                    .after(load_planet_config)
                    .after(setup_voxel_material),
            ),
        );

        // ── Orbital camera placement (Startup + 1) ───────────────────────
        // Runs in PostStartup so FloatingOriginPlugin::spawn_test_scene has
        // already created the FloatingOrigin camera entity.  Repositions the
        // single existing camera to 8,000 km altitude — outside the planet.
        // Registering here (not in FloatingOriginPlugin) keeps the camera
        // position concern with the planet engine, not the coordinate system.
        app.add_systems(PostStartup, reposition_origin_camera_to_orbit);

        // ── Update systems ────────────────────────────────────────────────
        // Ordering: scaled_space_switcher runs freely (read-only on sphere vis).
        //           chunk_streamer enqueues jobs.
        //           chunk_task_poller finalises completed tasks → inserts VoxelData.
        //           mesh_builder fires on Added<VoxelData> → inserts Mesh + ChunkMesh.
        //           collider_syncer fires on Added<ChunkMesh> → inserts Collider.
        app.add_systems(
            Update,
            (
                scaled_space_switcher,
                chunk_streamer,
                chunk_task_poller.after(chunk_streamer),
                mesh_builder.after(chunk_task_poller),
                collider_syncer.after(mesh_builder),
            ),
        );

        // ── Debug diagnostics (dev-only, T026 + T027) ────────────────────
        #[cfg(debug_assertions)]
        app.add_systems(Update, (log_voxel_stats, entity_census));

        // Note: FrameTimeDiagnosticsPlugin is already registered by CorePlugin.
    }
}
