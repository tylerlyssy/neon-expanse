//! Voxel planet engine — module root.
//!
//! Re-exports all public items from child modules.
//! Populated by Feature 002: Procedural Voxel Planet Engine.

pub mod components;
pub mod config;
pub mod erosion;
pub mod mesher;
pub mod noise_stack;
pub mod resources;
pub mod systems;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use components::{
    ChunkCollider, ChunkGenOutput, ChunkGenTask, ChunkMesh, PlanetCentre, ScaledSpaceMarker,
    VoxelChunk, VoxelData,
};
pub use config::{BiomeBand, BiomeConfig, ErosionConfig, NoiseKind, NoiseLayer, PlanetConfig};
pub use resources::{
    ChunkPool, ScaledSpaceState, StreamJob, StreamingQueue, VoxelMaterial, VoxelStats,
};
