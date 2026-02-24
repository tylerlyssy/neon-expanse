//! ColliderSyncer: generates Avian3d colliders on newly meshed chunks.
//!
//! Runs in `Update` after `mesh_builder` (via system ordering in
//! `VoxelWorldPlugin::build`). Reacts to `Added<ChunkMesh>` so each
//! chunk is processed exactly once.
//!
//! # Collider strategy by LOD
//! - **LOD 0**: exact `Collider::trimesh_from_mesh` — full precision for player/vehicle physics.
//! - **LOD 1–2**: `Collider::heightfield` — sufficient for surface-only terrain interaction.
//! - **LOD 3**: `ChunkCollider` marker only — no physics shape (scaled-space; no interaction).
//!
//! The `RigidBody::Static` component is inserted alongside the collider so the
//! physics engine treats the terrain as immovable.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::plugins::voxel::{
    components::{ChunkCollider, ChunkMesh, VoxelChunk, VoxelData},
    systems::streaming::chunk_size_for_lod,
};

/// Query param alias for chunk collider syncing.
type ChunkColliderQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static VoxelChunk,
        &'static ChunkMesh,
        &'static VoxelData,
    ),
    (Added<ChunkMesh>, Without<ChunkCollider>),
>;

/// `Update` system: attach Avian3d colliders to newly meshed chunk entities.
///
/// # Query filter
/// `Added<ChunkMesh>` fires the single frame when `mesh_builder` inserts the mesh.
/// `Without<ChunkCollider>` provides idempotency safety against duplicate inserts.
pub fn collider_syncer(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    chunk_q: ChunkColliderQuery,
) {
    for (entity, chunk, chunk_mesh, voxel_data) in &chunk_q {
        let Some(mesh) = meshes.get(&chunk_mesh.mesh_handle) else {
            // Mesh not yet uploaded — will be caught next frame by Without<ChunkCollider>.
            continue;
        };

        match chunk.lod {
            0 => {
                // ── Full trimesh — exact terrain collision ────────────────────
                if let Some(collider) = Collider::trimesh_from_mesh(mesh) {
                    commands
                        .entity(entity)
                        .insert(collider)
                        .insert(RigidBody::Static)
                        .insert(ChunkCollider);
                } else {
                    // Empty mesh (no surface at this chunk); still mark so we
                    // don't retry every frame.
                    commands.entity(entity).insert(ChunkCollider);
                    debug!(
                        "collider_syncer: LOD 0 chunk {:?} has no collidable geometry",
                        chunk.chunk_coord
                    );
                }
            }
            1 | 2 => {
                // ── Heightfield — surface-only terrain interaction ─────────────
                let n = voxel_data.voxels_per_edge as usize;
                let stride = n + 2; // border-padded grid row width

                // Build 2D heights grid: rows = z, cols = x.
                let heights_2d: Vec<Vec<f32>> = (0..n)
                    .map(|z| {
                        (0..n)
                            .map(|x| {
                                (0..n)
                                    .rev()
                                    .map(|y| {
                                        let xi = x + 1;
                                        let yi = y + 1;
                                        let zi = z + 1;
                                        let idx = xi + yi * stride + zi * stride * stride;
                                        voxel_data.densities.get(idx).copied().unwrap_or(0.0)
                                    })
                                    .find(|&d| d > 0.0)
                                    .unwrap_or(0.0)
                            })
                            .collect()
                    })
                    .collect();

                let chunk_size = chunk_size_for_lod(chunk.lod) as f32;
                let scale = Vec3::new(chunk_size, chunk_size, chunk_size);

                let collider = Collider::heightfield(heights_2d, scale);
                commands
                    .entity(entity)
                    .insert(collider)
                    .insert(RigidBody::Static)
                    .insert(ChunkCollider);
            }
            _ => {
                // LOD ≥ 3: no physics shape needed — scaled-space only.
                commands.entity(entity).insert(ChunkCollider);
            }
        }
    }
}
