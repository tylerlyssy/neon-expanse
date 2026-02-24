//! Chunk streaming: enqueue, dispatch, and poll async generation tasks.
//!
//! Two systems run in `Update` in order:
//! 1. `chunk_streamer`    — enqueues needed chunks, dispatches up to 4 tasks.
//! 2. `chunk_task_poller` — polls completed tasks, inserts `VoxelData` on entities.
//!
//! `mesh_builder` and `collider_syncer` react to `Added<VoxelData>` as downstream steps.

use bevy::tasks::futures_lite::future;
use bevy::{
    math::DVec3,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use crate::plugins::{
    core_plugin::WorldSeed,
    floating_origin::{components::GlobalPosition, resources::OriginFrame},
    lod::{components::LodLevel, resources::LodSettings},
    voxel::{
        components::{ChunkGenOutput, ChunkGenTask, VoxelChunk, VoxelData},
        config::PlanetConfig,
        erosion::apply_erosion,
        noise_stack::generate_chunk_densities,
        resources::{ChunkPool, StreamJob, StreamingQueue, VoxelStats},
    },
};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Chunk edge length in metres at LOD 0.
pub const CHUNK_SIZE_M: f64 = 128.0;

/// Base number of voxels per chunk edge at LOD 0.
pub const BASE_VOXELS: u32 = 32;

/// Maximum concurrent chunk generation tasks (FR-017 / Clarification Q5).
pub const MAX_TASKS: u8 = 4;

/// Hard cap on the chunk-coordinate search radius per LOD level.
///
/// Prevents the O(N³) enumeration loop from stalling the main thread when the
/// viewer is far from the planet surface.  At LOD-0 this caps coverage at
/// 10 × 128 m = 1,280 m; at LOD-3 at 4 × 1,024 m = 4,096 m — sufficient
/// for smooth terrain streaming while keeping the inner loop < 10 k iterations.
const MAX_RANGE_PER_LOD: [i32; 4] = [10, 8, 6, 4];

/// Altitude (metres above the planet surface) beyond which chunk streaming is
/// skipped entirely.  When the viewer is in orbit the scaled-space sphere
/// already renders the planet; generating surface chunks would stall the
/// AsyncComputeTaskPool with work that is never visible.
const STREAM_CUTOFF_ALTITUDE_M: f64 = 500_000.0; // 500 km

// ── chunk_streamer ────────────────────────────────────────────────────────────

/// `Update` system: enqueue needed chunks and dispatch async generation tasks.
///
/// For each LOD band, computes which chunk coordinates are within the streaming
/// radius and adds them to `StreamingQueue` if not already loaded/in-flight.
/// Dispatches up to `MAX_TASKS` tasks per frame; additional requests stay queued.
#[allow(clippy::too_many_arguments)]
pub fn chunk_streamer(
    origin: Res<OriginFrame>,
    lod_settings: Res<LodSettings>,
    planet_config: Res<PlanetConfig>,
    world_seed: Res<WorldSeed>,
    mut pool: ResMut<ChunkPool>,
    mut queue: ResMut<StreamingQueue>,
    mut stats: ResMut<VoxelStats>,
    task_q: Query<(), With<ChunkGenTask>>,
    mut commands: Commands,
) {
    // Sync task count from live entities (source of truth).
    queue.task_count = task_q.iter().count() as u8;
    debug_assert!(
        queue.task_count <= MAX_TASKS,
        "task_count ({}) exceeded MAX_TASKS ({MAX_TASKS})",
        queue.task_count
    );

    // Update diagnostics.
    stats.in_flight_tasks = queue.task_count;
    stats.loaded_chunks = pool.loaded.len() as u32;
    stats.queued_chunks = queue.pending.len() as u32;
    stats.memory_used_mb = pool.bytes_used as f32 / (1024.0 * 1024.0);

    let viewer = origin.position;
    let [r0, r1, r2] = lod_settings.thresholds;

    // ── Orbit guard ───────────────────────────────────────────────────────
    // Skip all chunk streaming when the viewer is high above the planet.
    // From orbit the scaled-space sphere already renders the planet; trying
    // to enumerate surface chunks at LOD-3 with a 63,710 km radius would
    // require a (2×62222+1)³ ≈ 10¹⁵ iteration loop on the main thread.
    let planet_radius_m = planet_config.radius_km as f64 * 1_000.0;
    let viewer_altitude_m = viewer.length() - planet_radius_m;
    if viewer_altitude_m > STREAM_CUTOFF_ALTITUDE_M {
        return;
    }

    // Streaming radii: use LOD thresholds as the streaming sphere for each LOD.
    // LOD-3 extends to 2× the LOD-2 threshold (~200 km) — enough for
    // the lowest-detail skirt around the view frustum without blowing up range.
    let stream_bands: [(f64, u8); 4] = [
        (r0 as f64, 0),
        (r1 as f64, 1),
        (r2 as f64, 2),
        (r2 as f64 * 2.0, 3),
    ];

    // ── Enqueue visible chunks ────────────────────────────────────────────
    for (radius_m, lod) in stream_bands {
        let chunk_size = chunk_size_for_lod(lod);
        // Cap range to MAX_RANGE_PER_LOD so the inner cube loop is always
        // bounded even if a large streaming radius is configured later.
        let range =
            ((radius_m / chunk_size).ceil() as i32 + 1).min(MAX_RANGE_PER_LOD[lod as usize]);
        let viewer_chunk = world_to_chunk(viewer, chunk_size);

        for dz in -range..=range {
            for dy in -range..=range {
                for dx in -range..=range {
                    let coord = viewer_chunk + IVec3::new(dx, dy, dz);

                    if pool.loaded.contains_key(&coord) {
                        continue;
                    }
                    if queue.in_flight.contains(&coord) {
                        continue;
                    }

                    let centre = chunk_centre_world(coord, chunk_size);
                    let dist_sq = (centre - viewer).length_squared();

                    if dist_sq > radius_m * radius_m {
                        continue;
                    }

                    queue.pending.push(StreamJob {
                        neg_dist_sq: -(dist_sq as i64),
                        coord,
                        lod,
                    });
                }
            }
        }
    }

    // ── Dispatch tasks ────────────────────────────────────────────────────
    let thread_pool = AsyncComputeTaskPool::get();

    while queue.task_count < MAX_TASKS {
        let Some(job) = queue.pending.pop() else {
            break;
        };

        // Double-check: may have been loaded since enqueue.
        if pool.loaded.contains_key(&job.coord) {
            continue;
        }
        if queue.in_flight.contains(&job.coord) {
            continue;
        }

        queue.in_flight.insert(job.coord);
        queue.task_count += 1;

        let coord = job.coord;
        let lod = job.lod;
        let config = planet_config.clone();
        let seed = world_seed.0;

        let task: Task<ChunkGenOutput> = thread_pool.spawn(async move {
            let start = std::time::Instant::now();
            let output = build_chunk(coord, lod, seed, &config);
            let elapsed = start.elapsed();
            if elapsed.as_millis() > 100 {
                warn!(
                    "Chunk generation exceeded 100 ms budget: {:?} ms for coord {:?} LOD {}",
                    elapsed.as_millis(),
                    coord,
                    lod,
                );
            }
            output
        });

        commands.spawn(ChunkGenTask(task));
    }

    // ── Evict over-budget chunks ──────────────────────────────────────────
    let budget_bytes = planet_config.memory_budget_mb as usize * 1024 * 1024;
    while pool.bytes_used > budget_bytes {
        // Evict the furthest loaded chunk from the viewer.
        let to_evict = pool
            .loaded
            .iter()
            .max_by_key(|(c, _)| {
                let cs = chunk_size_for_lod(0); // approximate
                (chunk_centre_world(**c, cs) - viewer).length_squared() as i64
            })
            .map(|(c, e)| (*c, *e));

        if let Some((coord, entity)) = to_evict {
            commands.entity(entity).despawn();
            pool.loaded.remove(&coord);
            // Note: bytes_used is decremented by chunk_task_poller when it
            // processes the entity removal. Here we do a best-effort decrement.
            pool.bytes_used = pool.bytes_used.saturating_sub(1024); // approximate
        } else {
            break;
        }
    }
}

// ── chunk_task_poller ─────────────────────────────────────────────────────────

/// `Update` system: poll completed async tasks and finalise chunk entities.
///
/// When a `ChunkGenTask` completes:
/// 1. Insert `VoxelData` onto a new chunk entity (triggers `mesh_builder` via `Added<VoxelData>`).
/// 2. Register the entity in `ChunkPool`.
/// 3. Despawn the temporary task entity.
///
/// # Ordering guarantee (FR-032)
/// `ChunkPool.loaded` is updated *before* any old placeholder is despawned,
/// ensuring no single-frame gap in coverage.
pub fn chunk_task_poller(
    mut commands: Commands,
    mut pool: ResMut<ChunkPool>,
    mut queue: ResMut<StreamingQueue>,
    mut task_q: Query<(Entity, &mut ChunkGenTask)>,
) {
    for (task_entity, mut task) in &mut task_q {
        let Some(output) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        let coord = output.coord;
        let lod = output.lod;
        let byte_count = output.voxel_data.densities.len() * std::mem::size_of::<f32>();

        // Spawn the chunk entity with VoxelData — mesh_builder reacts to `Added<VoxelData>`.
        let chunk_entity = commands
            .spawn((
                Name::new(format!(
                    "Chunk({},{},{})-LOD{}",
                    coord.x, coord.y, coord.z, lod
                )),
                VoxelChunk {
                    chunk_coord: coord,
                    lod,
                },
                output.voxel_data,
                GlobalPosition(chunk_centre_world(coord, chunk_size_for_lod(lod))),
                LodLevel(lod),
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        // Register in pool BEFORE despawning any placeholder (FR-032).
        let old_entity = pool.loaded.insert(coord, chunk_entity);
        pool.bytes_used += byte_count;
        queue.in_flight.remove(&coord);

        // Despawn old placeholder/lower-LOD chunk for this coord.
        if let Some(old) = old_entity {
            if old != chunk_entity {
                commands.entity(old).despawn();
            }
        }

        // Despawn the temporary task entity.
        commands.entity(task_entity).despawn();

        debug!(
            "Chunk({},{},{})-LOD{} finalised: {} density samples",
            coord.x,
            coord.y,
            coord.z,
            lod,
            byte_count / std::mem::size_of::<f32>(),
        );
    }
}

// ── Chunk generation (worker thread) ─────────────────────────────────────────

/// Density-only chunk generation pipeline. Runs on `AsyncComputeTaskPool`.
///
/// Pipeline: `generate_chunk_densities` → `apply_erosion` → return `ChunkGenOutput`.
/// Mesh creation is intentionally deferred to `mesh_builder` (main thread) via `Added<VoxelData>`.
fn build_chunk(coord: IVec3, lod: u8, seed: u64, config: &PlanetConfig) -> ChunkGenOutput {
    let chunk_size = chunk_size_for_lod(lod);
    let voxels = voxels_for_lod(lod);
    let cell_size = (chunk_size / voxels as f64) as f32;
    let origin = chunk_centre_world(coord, chunk_size) - DVec3::splat(chunk_size / 2.0);

    // ── Density evaluation ────────────────────────────────────────────────
    let mut densities = generate_chunk_densities(origin, cell_size, voxels, seed, config);

    // ── Hydraulic erosion ─────────────────────────────────────────────────
    let n = voxels + 2; // includes 1-voxel border
    apply_erosion(&mut densities, &config.erosion, n, n);

    let voxel_data = VoxelData {
        densities,
        voxels_per_edge: voxels,
    };

    ChunkGenOutput {
        coord,
        lod,
        voxel_data,
        // Mesh is left empty; mesh_builder owns mesh creation via Added<VoxelData>.
        mesh: Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        ),
        vertex_colours: vec![],
        positions_count: 0,
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Chunk edge length in metres for a given LOD level.
pub fn chunk_size_for_lod(lod: u8) -> f64 {
    CHUNK_SIZE_M * f64::from(1u32 << lod.min(15))
}

/// Voxel resolution per edge for a given LOD level.
pub fn voxels_for_lod(lod: u8) -> u32 {
    BASE_VOXELS >> lod.min(4)
}

/// Convert a world-space position to chunk-space coordinates.
pub fn world_to_chunk(pos: DVec3, chunk_size: f64) -> IVec3 {
    IVec3::new(
        (pos.x / chunk_size).floor() as i32,
        (pos.y / chunk_size).floor() as i32,
        (pos.z / chunk_size).floor() as i32,
    )
}

/// Compute the world-space centre of a chunk (metres).
pub fn chunk_centre_world(coord: IVec3, chunk_size: f64) -> DVec3 {
    DVec3::new(
        (coord.x as f64 + 0.5) * chunk_size,
        (coord.y as f64 + 0.5) * chunk_size,
        (coord.z as f64 + 0.5) * chunk_size,
    )
}
