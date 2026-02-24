//! MeshBuilder: converts `VoxelData` into a Bevy `Mesh` asset.
//!
//! Runs in `Update` after `chunk_task_poller`. Reacts to `Added<VoxelData>` —
//! this system is the **sole owner** of mesh creation; `chunk_streamer` never
//! calls the DMC mesher directly.
//!
//! # LOD placeholder swap
//! Inserts new mesh before despawning any lower-LOD placeholder, so there is
//! never a single-frame gap without visible geometry at a given coordinate.

use bevy::prelude::*;

use crate::plugins::voxel::{
    components::{ChunkMesh, VoxelChunk, VoxelData},
    config::PlanetConfig,
    mesher::dmc::mesh_chunk,
    resources::VoxelMaterial,
    systems::streaming::{chunk_size_for_lod, voxels_for_lod},
};

/// Query alias for chunk mesh building.
type MeshBuilderQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static VoxelChunk, &'static VoxelData),
    (Added<VoxelData>, Without<ChunkMesh>),
>;

// ── Setup ─────────────────────────────────────────────────────────────────────

/// `Startup` system: create and register the shared vertex-colour material.
///
/// Every chunk mesh uses this material; biome colours are encoded in
/// `Mesh::ATTRIBUTE_COLOR` per vertex so no additional uniforms are needed.
pub fn setup_voxel_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    commands.insert_resource(VoxelMaterial(mat));
    info!("VoxelMaterial registered (vertex-colour SharedMaterial)");
}

// ── mesh_builder ──────────────────────────────────────────────────────────────

/// `Update` system: build and attach a `Mesh` for every newly finalised chunk.
///
/// # Trigger
/// Queries entities with `Added<VoxelData>` that do not yet have `ChunkMesh`.
/// This fires once per chunk, the frame after `chunk_task_poller` inserts the data.
///
/// # System signature parameters
/// - `commands`       — for inserting components on the chunk entity
/// - `meshes`         — `Assets<Mesh>` to register the new mesh
/// - `mat`            — shared `VoxelMaterial` (vertex-colour `StandardMaterial`)
/// - `planet_config`  — biome bands + planet radius
/// - **chunked query** — `(Entity, &VoxelChunk, &VoxelData)` + filter `Added<VoxelData>` + `Without<ChunkMesh>`
pub fn mesh_builder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mat: Res<VoxelMaterial>,
    planet_config: Res<PlanetConfig>,
    chunk_q: MeshBuilderQuery,
) {
    for (entity, chunk, voxel_data) in &chunk_q {
        let lod = chunk.lod;
        let chunk_size = chunk_size_for_lod(lod);
        let voxels = voxels_for_lod(lod);
        let cell_size = (chunk_size / voxels as f64) as f32;

        // Mesh local origin is ZERO; the entity's `GlobalPosition` + Transform
        // place it correctly in world space via FloatingOriginPlugin.
        let chunk_origin_m = Vec3::ZERO;
        let planet_r_m = (planet_config.radius_km * 1_000.0) as f32;

        // ── Run DMC mesher ────────────────────────────────────────────────────
        let mesh = mesh_chunk(
            voxel_data,
            chunk_origin_m,
            cell_size,
            &planet_config.biome,
            planet_r_m,
        );

        let vertex_count = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .map(|a: &bevy::mesh::VertexAttributeValues| a.len() as u32)
            .unwrap_or(0);

        // ── Register mesh and attach components ───────────────────────────────
        let mesh_handle = meshes.add(mesh);

        commands.entity(entity).insert((
            ChunkMesh {
                mesh_handle: mesh_handle.clone(),
                vertex_count,
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat.0.clone()),
        ));

        debug!(
            "mesh_builder: chunk {:?} LOD {} → {} vertices",
            chunk.chunk_coord, lod, vertex_count,
        );
    }
}
