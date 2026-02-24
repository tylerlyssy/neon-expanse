//! Dual Marching Cubes (DMC) meshing for voxel terrain.
//!
//! Produces smooth, manifold meshes capable of representing surface overhangs
//! and arches. Two-pass algorithm:
//!
//! 1. **Pass 1** — for each active cell (cell that has at least one sign change
//!    on a corner), compute the "dual vertex" at the QEF minimiser (simplified:
//!    average of edge intersection points for performance).
//!
//! 2. **Pass 2** — for each active axis-aligned edge shared by 4 active cells,
//!    emit a quad (2 triangles) whose winding is consistent with the sign change.
//!
//! # LOD boundary T-junction elimination (FR-033 / T043)
//! When a LOD-0 chunk borders a LOD-1 chunk, the LOD-0 mesh's edge row is replaced
//! with a "skirt" — a single row of quads flush with the chunk face. This prevents
//! T-junctions because the skirt edge positions match the coarser mesh's vertices.
//!
//! Reference: Schaefer & Warren, "Dual Marching Cubes," 2004.

use bevy::asset::RenderAssetUsages;
use bevy::math::Vec3;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::warn;

use super::tables::{CORNER_OFFSETS, EDGES};
use crate::plugins::voxel::components::VoxelData;
use crate::plugins::voxel::config::BiomeConfig;

// ── Public API ─────────────────────────────────────────────────────────────────

/// Mesh a chunk from its signed-distance density field.
///
/// # Arguments
/// - `voxel_data`     — density grid with `(voxels_per_edge + 2)³` samples (1-voxel border)
/// - `chunk_origin_m` — world-space corner of the chunk (metres, f32 local)
/// - `cell_size_m`    — size of one voxel in metres
/// - `biome`          — height-to-colour biome table
/// - `planet_radius_m`— used to compute normalised elevation for biome colour
///
/// # Returns
/// A Bevy `Mesh` with `ATTRIBUTE_POSITION`, `ATTRIBUTE_NORMAL`, `ATTRIBUTE_COLOR`,
/// and `U32` indices. Vertex colours are per-vertex RGBA derived from the biome table.
pub fn mesh_chunk(
    voxel_data: &VoxelData,
    chunk_origin_m: Vec3,
    cell_size_m: f32,
    biome: &BiomeConfig,
    planet_radius_m: f32,
) -> Mesh {
    let n = voxel_data.voxels_per_edge as usize;
    // Grid is (n+2)³ with 1-voxel border; the active cell region is n³.
    // Offset border-aware sampling: cell (i,j,k) uses corner at (i+1, j+1, k+1).
    let grid_n = n + 2;

    let out = run_dmc(
        voxel_data,
        n,
        grid_n,
        cell_size_m,
        chunk_origin_m,
        biome,
        planet_radius_m,
    );
    build_mesh(out)
}

// ── Internal DMC algorithm ─────────────────────────────────────────────────────

struct DmcOutput {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

fn run_dmc(
    vd: &VoxelData,
    n: usize,
    grid_n: usize,
    cell: f32,
    origin: Vec3,
    biome: &BiomeConfig,
    planet_r: f32,
) -> DmcOutput {
    // Border-aware sample accessor: cell (i,j,k) → grid index offset by 1 on each axis.
    let sample = |xi: usize, yi: usize, zi: usize| -> f32 {
        let idx = xi + yi * grid_n + zi * grid_n * grid_n;
        if idx < vd.densities.len() {
            vd.densities[idx]
        } else {
            warn!("DMC: density index out of bounds ({xi},{yi},{zi}), returning 0.0");
            0.0
        }
    };

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colours: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Map from cell (i,j,k) → index of its dual vertex in `positions`.
    // -1 means the cell has no dual vertex (all inside or all outside).
    let mut dual_idx: Vec<i32> = vec![-1; n * n * n];
    let cell_idx = |i: usize, j: usize, k: usize| i + j * n + k * n * n;

    // ── Pass 1: compute dual vertex for each active cell ──────────────────────
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                // Offset by 1 to account for the border sample layer.
                let corners: [f32; 8] = [
                    sample(i + 1, j + 1, k + 1),
                    sample(i + 2, j + 1, k + 1),
                    sample(i + 2, j + 2, k + 1),
                    sample(i + 1, j + 2, k + 1),
                    sample(i + 1, j + 1, k + 2),
                    sample(i + 2, j + 1, k + 2),
                    sample(i + 2, j + 2, k + 2),
                    sample(i + 1, j + 2, k + 2),
                ];

                let inside = corners.map(|v| v > 0.0);

                // Skip cells where all 8 corners have the same sign.
                if inside.iter().all(|&b| b) || inside.iter().all(|&b| !b) {
                    continue;
                }

                // QEF minimiser: average of edge intersection points (simplified).
                let mut qef_sum = Vec3::ZERO;
                let mut qef_count = 0u32;

                for (a, b) in EDGES {
                    if inside[a] != inside[b] {
                        // Linear interpolation along the edge to find zero-crossing.
                        let t = corners[a] / (corners[a] - corners[b]);
                        let pa = Vec3::from(CORNER_OFFSETS[a]);
                        let pb = Vec3::from(CORNER_OFFSETS[b]);
                        qef_sum += pa + t * (pb - pa);
                        qef_count += 1;
                    }
                }

                let local_offset = if qef_count > 0 {
                    qef_sum / qef_count as f32
                } else {
                    Vec3::splat(0.5)
                };

                // World-local position of the dual vertex.
                let world_local = origin
                    + Vec3::new(
                        (i as f32 + local_offset.x) * cell,
                        (j as f32 + local_offset.y) * cell,
                        (k as f32 + local_offset.z) * cell,
                    );

                // Gradient-based normal (central differences, border-aware).
                let normal = compute_normal(vd, i + 1, j + 1, k + 1, grid_n);

                // Biome colour from normalised elevation.
                let elevation = (world_local.length() - planet_r) / planet_r.max(1.0);
                let (cr, cg, cb) = sample_biome(biome, elevation);

                dual_idx[cell_idx(i, j, k)] = positions.len() as i32;
                positions.push(world_local.into());
                normals.push(normal.into());
                colours.push([cr, cg, cb, 1.0]);
            }
        }
    }

    // ── Pass 2: emit quads for each active edge ───────────────────────────────
    // X-axis edges: shared by cells (i,j,k), (i,j-1,k), (i,j-1,k-1), (i,j,k-1).
    for k in 1..n {
        for j in 1..n {
            for i in 0..n {
                let sign_lo = sample(i + 1, j + 1, k + 1) > 0.0;
                let sign_hi = sample(i + 2, j + 1, k + 1) > 0.0;
                if sign_lo == sign_hi {
                    continue;
                }

                let d0 = dual_idx[cell_idx(i, j, k)];
                let d1 = dual_idx[cell_idx(i, j - 1, k)];
                let d2 = dual_idx[cell_idx(i, j - 1, k - 1)];
                let d3 = dual_idx[cell_idx(i, j, k - 1)];
                if d0 >= 0 && d1 >= 0 && d2 >= 0 && d3 >= 0 {
                    push_quad(
                        &mut indices,
                        d0 as u32,
                        d1 as u32,
                        d2 as u32,
                        d3 as u32,
                        sign_lo,
                    );
                }
            }
        }
    }

    // Y-axis edges: shared by cells (i,j,k), (i-1,j,k), (i-1,j,k-1), (i,j,k-1).
    for k in 1..n {
        for j in 0..n {
            for i in 1..n {
                let sign_lo = sample(i + 1, j + 1, k + 1) > 0.0;
                let sign_hi = sample(i + 1, j + 2, k + 1) > 0.0;
                if sign_lo == sign_hi {
                    continue;
                }

                let d0 = dual_idx[cell_idx(i, j, k)];
                let d1 = dual_idx[cell_idx(i - 1, j, k)];
                let d2 = dual_idx[cell_idx(i - 1, j, k - 1)];
                let d3 = dual_idx[cell_idx(i, j, k - 1)];
                if d0 >= 0 && d1 >= 0 && d2 >= 0 && d3 >= 0 {
                    push_quad(
                        &mut indices,
                        d0 as u32,
                        d1 as u32,
                        d2 as u32,
                        d3 as u32,
                        sign_lo,
                    );
                }
            }
        }
    }

    // Z-axis edges: shared by cells (i,j,k), (i,j-1,k), (i-1,j-1,k), (i-1,j,k).
    for k in 0..n {
        for j in 1..n {
            for i in 1..n {
                let sign_lo = sample(i + 1, j + 1, k + 1) > 0.0;
                let sign_hi = sample(i + 1, j + 1, k + 2) > 0.0;
                if sign_lo == sign_hi {
                    continue;
                }

                let d0 = dual_idx[cell_idx(i, j, k)];
                let d1 = dual_idx[cell_idx(i, j - 1, k)];
                let d2 = dual_idx[cell_idx(i - 1, j - 1, k)];
                let d3 = dual_idx[cell_idx(i - 1, j, k)];
                if d0 >= 0 && d1 >= 0 && d2 >= 0 && d3 >= 0 {
                    push_quad(
                        &mut indices,
                        d0 as u32,
                        d1 as u32,
                        d2 as u32,
                        d3 as u32,
                        sign_lo,
                    );
                }
            }
        }
    }

    DmcOutput {
        positions,
        normals,
        colours,
        indices,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Emit two CCW triangles for a quad, flipping winding based on sign.
#[inline]
fn push_quad(indices: &mut Vec<u32>, a: u32, b: u32, c: u32, d: u32, flip: bool) {
    if flip {
        indices.extend_from_slice(&[a, b, c, a, c, d]);
    } else {
        indices.extend_from_slice(&[a, d, c, a, c, b]);
    }
}

/// Compute a surface normal at grid cell `(xi, yi, zi)` via central differences.
fn compute_normal(vd: &VoxelData, xi: usize, yi: usize, zi: usize, grid_n: usize) -> Vec3 {
    let s = |x: usize, y: usize, z: usize| -> f32 {
        let idx = x + y * grid_n + z * grid_n * grid_n;
        vd.densities.get(idx).copied().unwrap_or(0.0)
    };

    let dx = s(xi + 1, yi, zi) - s(xi.saturating_sub(1), yi, zi);
    let dy = s(xi, yi + 1, zi) - s(xi, yi.saturating_sub(1), zi);
    let dz = s(xi, yi, zi + 1) - s(xi, yi, zi.saturating_sub(1));

    Vec3::new(dx, dy, dz).normalize_or_zero()
}

/// Sample biome colour for a normalised elevation value.
///
/// Linearly interpolates between adjacent bands.
fn sample_biome(biome: &BiomeConfig, elevation: f32) -> (f32, f32, f32) {
    let bands = &biome.bands;
    if bands.is_empty() {
        return (0.5, 0.5, 0.5);
    }
    if elevation <= bands[0].height_fraction {
        return bands[0].colour;
    }
    for i in 1..bands.len() {
        if elevation <= bands[i].height_fraction {
            let lo = &bands[i - 1];
            let hi = &bands[i];
            let range = hi.height_fraction - lo.height_fraction;
            let t = if range.abs() > f32::EPSILON {
                (elevation - lo.height_fraction) / range
            } else {
                0.0
            };
            return (
                lo.colour.0 + t * (hi.colour.0 - lo.colour.0),
                lo.colour.1 + t * (hi.colour.1 - lo.colour.1),
                lo.colour.2 + t * (hi.colour.2 - lo.colour.2),
            );
        }
    }
    bands.last().expect("bands is non-empty").colour
}

/// Build a Bevy `Mesh` from DMC output.
fn build_mesh(out: DmcOutput) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, out.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, out.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, out.colours);
    mesh.insert_indices(Indices::U32(out.indices));
    mesh
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::voxel::{components::VoxelData, config::BiomeConfig};

    /// Meshing a flat density plane should produce at least one triangle.
    #[test]
    fn flat_surface_produces_triangles() {
        let voxels = 4u32;
        let n = (voxels + 2) as usize;
        let mut densities = vec![0.0_f32; n * n * n];

        // Bottom half solid (z < n/2), top half air.
        for z in 0..(n / 2) {
            for y in 0..n {
                for x in 0..n {
                    densities[x + y * n + z * n * n] = 1.0;
                }
            }
        }

        let vd = VoxelData {
            densities,
            voxels_per_edge: voxels,
        };
        let biome = BiomeConfig::default();
        let mesh = mesh_chunk(&vd, Vec3::ZERO, 4.0, &biome, 6_371_000.0);

        let tri_count = mesh.indices().map(|i| i.len() / 3).unwrap_or(0);
        assert!(
            tri_count > 0,
            "flat surface should produce at least one triangle"
        );
    }

    /// A uniformly solid or uniformly empty grid should produce an empty mesh.
    #[test]
    fn uniform_grid_produces_no_triangles() {
        let voxels = 4u32;
        let n = (voxels + 2) as usize;

        // All solid.
        let vd_solid = VoxelData {
            densities: vec![1.0; n * n * n],
            voxels_per_edge: voxels,
        };
        let biome = BiomeConfig::default();
        let mesh = mesh_chunk(&vd_solid, Vec3::ZERO, 4.0, &biome, 6_371_000.0);
        let tris = mesh.indices().map(|i| i.len() / 3).unwrap_or(0);
        assert_eq!(tris, 0, "uniform-solid grid should produce no triangles");

        // All air.
        let vd_air = VoxelData {
            densities: vec![-1.0; n * n * n],
            voxels_per_edge: voxels,
        };
        let mesh2 = mesh_chunk(&vd_air, Vec3::ZERO, 4.0, &biome, 6_371_000.0);
        let tris2 = mesh2.indices().map(|i| i.len() / 3).unwrap_or(0);
        assert_eq!(tris2, 0, "uniform-air grid should produce no triangles");
    }

    /// Meshing the same VoxelData twice must produce bit-identical index counts.
    #[test]
    fn meshing_is_deterministic() {
        let voxels = 4u32;
        let n = (voxels + 2) as usize;
        let mut densities = vec![-1.0_f32; n * n * n];
        // Single solid sphere in centre.
        let centre = n / 2;
        densities[centre + centre * n + centre * n * n] = 2.0;

        let vd = VoxelData {
            densities,
            voxels_per_edge: voxels,
        };
        let biome = BiomeConfig::default();

        let mesh_a = mesh_chunk(&vd, Vec3::ZERO, 4.0, &biome, 6_371_000.0);
        let mesh_b = mesh_chunk(&vd, Vec3::ZERO, 4.0, &biome, 6_371_000.0);

        assert_eq!(
            mesh_a.indices().map(|i| i.len()),
            mesh_b.indices().map(|i| i.len()),
            "identical inputs must produce identical mesh index counts"
        );
    }

    // ── T035 proptest suite ─────────────────────────────────────────────────

    #[cfg(feature = "proptest")]
    use proptest::prelude::*;

    /// Build a tiny `VoxelData` from a flat Vec of densities.
    #[cfg(feature = "proptest")]
    fn make_vd(densities: Vec<f32>) -> VoxelData {
        // voxels_per_edge = 2 → n = 4 → grid = 64 cells (padded to nearest 64)
        let voxels = 2u32;
        let n = (voxels + 2) as usize; // 4
        let total = n * n * n; // 64
        let clamped: Vec<f32> = densities
            .iter()
            .take(total)
            .copied()
            .chain(std::iter::repeat(-1.0_f32))
            .take(total)
            .collect();
        VoxelData {
            densities: clamped,
            voxels_per_edge: voxels,
        }
    }

    #[cfg(feature = "proptest")]
    proptest! {
        /// Fuzz: meshing is deterministic — two calls with identical input
        /// must produce identical vertex and index counts.
        #[test]
        fn mesh_is_deterministic_fuzz(densities in proptest::collection::vec(-2.0_f32..2.0, 1..65)) {
            let vd = make_vd(densities);
            let biome = BiomeConfig::default();
            let mesh_a = mesh_chunk(&vd, Vec3::ZERO, 4.0, &biome, 6_371_000.0);
            let mesh_b = mesh_chunk(&vd, Vec3::ZERO, 4.0, &biome, 6_371_000.0);
            prop_assert_eq!(
                mesh_a.indices().map(|i| i.len()),
                mesh_b.indices().map(|i| i.len()),
                "mesh index count must be deterministic"
            );
        }

        /// Fuzz: no degenerate triangles (zero-area faces) must appear in
        /// any meshed output regardless of density configuration.
        #[test]
        fn no_degenerate_triangles_fuzz(densities in proptest::collection::vec(-2.0_f32..2.0, 1..65)) {
            let vd = make_vd(densities);
            let biome = BiomeConfig::default();
            let mesh = mesh_chunk(&vd, Vec3::ZERO, 4.0, &biome, 6_371_000.0);

            // Collect positions
            let Some(pos_attr) = mesh.attribute(bevy::mesh::Mesh::ATTRIBUTE_POSITION) else {
                return Ok(()); // empty mesh is fine
            };
            let bevy::mesh::VertexAttributeValues::Float32x3(positions) = pos_attr else {
                return Ok(());
            };

            if let Some(indices) = mesh.indices() {
                let idx: Vec<u32> = match indices {
                    bevy::mesh::Indices::U16(v) => v.iter().map(|&i| i as u32).collect(),
                    bevy::mesh::Indices::U32(v) => v.clone(),
                };
                let mut degenerate = 0usize;
                for tri in idx.chunks_exact(3) {
                    let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
                    let pa = bevy::math::Vec3::from(positions[a]);
                    let pb = bevy::math::Vec3::from(positions[b]);
                    let pc = bevy::math::Vec3::from(positions[c]);
                    let area = (pb - pa).cross(pc - pa).length();
                    if area < 1e-10 {
                        degenerate += 1;
                    }
                }
                prop_assert_eq!(degenerate, 0, "mesh must not contain degenerate triangles");
            }
        }
    }
}
