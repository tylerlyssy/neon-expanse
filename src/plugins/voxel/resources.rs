//! ECS resources for the voxel planet engine.
//!
//! All resources derive `Default` so they can be registered with
//! `app.init_resource::<T>()` in `VoxelWorldPlugin::build`.

use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap, HashSet};

// ── Chunk pool ────────────────────────────────────────────────────────────────

/// Registry of all currently loaded chunk entities keyed by chunk-space coordinate.
///
/// Used for deduplication (avoid spawning the same chunk twice) and eviction
/// (find the furthest chunk to remove when `memory_budget_mb` is exceeded).
#[derive(Resource, Default, Debug)]
pub struct ChunkPool {
    /// Coord → entity map for fast lookup.
    pub loaded: HashMap<IVec3, Entity>,
    /// Per-coord density byte counts; entries mirror `loaded` 1-to-1.
    pub byte_counts: HashMap<IVec3, usize>,
    /// Running total of voxel density bytes held in memory. Updated by the
    /// task poller when chunks are added/removed.
    pub bytes_used: usize,
}

// ── Streaming queue ───────────────────────────────────────────────────────────

/// Priority-queue entry for a pending chunk generation job.
///
/// Stored in a `BinaryHeap` (max-heap). Distance is negated so that the
/// binary heap behaves as a min-heap, popping the *nearest* chunk first.
#[derive(Debug, Eq, PartialEq)]
pub struct StreamJob {
    /// Negative squared distance from viewer (min-heap trick on BinaryHeap).
    pub neg_dist_sq: i64,
    /// Chunk-space coordinate to generate.
    pub coord: IVec3,
    /// LOD level for the generated chunk.
    pub lod: u8,
}

impl Ord for StreamJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.neg_dist_sq.cmp(&other.neg_dist_sq)
    }
}

impl PartialOrd for StreamJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority queue driving async chunk generation dispatch.
///
/// `chunk_streamer` enqueues `StreamJob`s; entries are dispatched up to the
/// maximum of 4 concurrent tasks (`task_count < 4`).
#[derive(Resource, Default, Debug)]
pub struct StreamingQueue {
    /// Min-heap (nearest-first) of pending generation jobs.
    pub pending: BinaryHeap<StreamJob>,
    /// Coordinates with an in-flight task — prevents duplicate dispatch.
    pub in_flight: HashSet<IVec3>,
    /// Number of `AsyncComputeTaskPool` tasks currently running. Hard cap: 4.
    pub task_count: u8,
}

// ── Scaled-space state ────────────────────────────────────────────────────────

/// Whether the scaled-space planet sphere is currently displayed.
///
/// Updated by `scaled_space_switcher` each frame.
#[derive(Resource, Debug, Default)]
pub struct ScaledSpaceState {
    /// `true` when viewer distance > 10× planet radius (sphere mode active).
    pub sphere_visible: bool,
}

// ── Diagnostics ───────────────────────────────────────────────────────────────

// ── Shared material ───────────────────────────────────────────────────────────

/// Handle to the shared vertex-colour material used by all voxel chunk meshes.
///
/// Inserted during `Startup` by `setup_voxel_material`. All chunk entities share
/// this material — colour variation comes from per-vertex `ATTRIBUTE_COLOR`,
/// avoiding per-chunk material allocations.
///
/// Must be inserted before `mesh_builder` runs (guaranteed by system ordering).
#[derive(Resource, Debug, Clone)]
pub struct VoxelMaterial(pub Handle<StandardMaterial>);

// ── Diagnostics ───────────────────────────────────────────────────────────────

/// Live diagnostics reported by `VoxelWorldPlugin`.
///
/// Updated every frame by `chunk_streamer`. Exposed via `VoxelWorldPlugin::stats`
/// for tests and external plugins (e.g., TraversalPlugin budget throttling).
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct VoxelStats {
    /// Number of chunks currently resident in `ChunkPool`.
    pub loaded_chunks: u32,
    /// Number of pending entries in `StreamingQueue`.
    pub queued_chunks: u32,
    /// Number of in-flight `AsyncComputeTaskPool` tasks (max 4).
    pub in_flight_tasks: u8,
    /// Approximate voxel data memory used (megabytes).
    pub memory_used_mb: f32,
}
