//! ECS components for the voxel planet engine.
//!
//! All components derive `Reflect` for Bevy's scene/inspector tooling.
//! Marker components carry no data; their presence on an entity signals
//! a specific state in the chunk lifecycle.

use bevy::{prelude::*, tasks::Task};

// ── Chunk identity ────────────────────────────────────────────────────────────

/// Marks an entity as an active voxel chunk tile.
///
/// The `chunk_coord` is an `IVec3` in *chunk-space* (1 unit = 1 chunk edge length
/// at the given LOD). Use `streaming::chunk_centre_world` to convert to metres.
#[derive(Component, Debug, Clone, Reflect)]
pub struct VoxelChunk {
    /// Chunk-space grid address.
    pub chunk_coord: IVec3,
    /// LOD level: 0 = highest detail (< 1 km), 3 = lowest (> 100 km).
    pub lod: u8,
}

// ── Density data ──────────────────────────────────────────────────────────────

/// Raw signed-distance density field for a chunk.
///
/// Values `> 0` are inside rock/ground; values `≤ 0` are air.
///
/// # Grid layout
/// The grid is (voxels_per_edge + 2)³ to include a 1-voxel border overlap at
/// each face, guaranteeing identical density values at shared chunk edges (FR-023).
/// Indexing: `densities[x + y*(n+2) + z*(n+2)*(n+2)]` where `n = voxels_per_edge`.
#[derive(Component, Debug, Clone, Reflect)]
pub struct VoxelData {
    /// Flattened 3-D density grid. Length = (voxels_per_edge + 2)³.
    pub densities: Vec<f32>,
    /// Voxel resolution per chunk edge (excludes the 1-voxel border).
    pub voxels_per_edge: u32,
}

// ── Mesh handle ───────────────────────────────────────────────────────────────

/// Handle to the mesh produced by the DMC mesher for this chunk.
#[derive(Component, Debug, Clone, Reflect)]
pub struct ChunkMesh {
    /// Asset handle to the Bevy `Mesh`.
    pub mesh_handle: Handle<Mesh>,
    /// Number of vertices in the mesh (for memory budget tracking).
    pub vertex_count: u32,
}

// ── Physics marker ────────────────────────────────────────────────────────────

/// Marker: this chunk's Avian3d collider has been generated and attached.
///
/// Queried with `Without<ChunkCollider>` to find chunks still needing colliders.
#[derive(Component, Debug, Default, Reflect)]
pub struct ChunkCollider;

// ── Rendering markers ─────────────────────────────────────────────────────────

/// Marker: this entity is the scaled-space planet sphere.
///
/// Toggled by `scaled_space_switcher` based on viewer distance.
#[derive(Component, Debug, Default, Reflect)]
pub struct ScaledSpaceMarker;

/// Marker: authoritative planet centre reference point at `GlobalPosition(DVec3::ZERO)`.
///
/// All chunk distance queries use this entity as the reference origin.
#[derive(Component, Debug, Default, Reflect)]
pub struct PlanetCentre;

// ── Async task wrapper ────────────────────────────────────────────────────────

/// Wraps the `Task<ChunkGenOutput>` spawned on `AsyncComputeTaskPool`.
///
/// Attached to a temporary entity; removed (entity despawned) when the task
/// resolves and the chunk entity is finalised.
#[derive(Component)]
pub struct ChunkGenTask(pub Task<ChunkGenOutput>);

// ── Async output ──────────────────────────────────────────────────────────────

/// Output produced by a completed async chunk generation job.
///
/// Moved from the worker thread back to the main thread by `chunk_task_poller`.
pub struct ChunkGenOutput {
    /// Chunk-space coordinate that was generated.
    pub coord: IVec3,
    /// LOD level of the generated chunk.
    pub lod: u8,
    /// Raw density field (includes 1-voxel border overlap per face).
    pub voxel_data: VoxelData,
    /// Ready-to-insert Bevy `Mesh` produced by the DMC mesher.
    pub mesh: Mesh,
    /// Per-vertex RGBA colours (matches `mesh` vertex count).
    pub vertex_colours: Vec<[f32; 4]>,
    /// Vertex count for `ChunkMesh::vertex_count`.
    pub positions_count: u32,
}
