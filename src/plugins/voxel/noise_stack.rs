//! Layered noise density evaluator using `fastnoise-lite`.
//!
//! This module is a **pure function library**: same inputs always produce the
//! same outputs. No global state, no RNG re-seeding inside evaluations.
//!
//! # Determinism guarantee
//! Given identical `seed`, `config`, and `pos`, `evaluate_density` MUST return
//! the same `f32` across all runs, platforms, and compiler versions.
//! This is enforced by proptest (see `tests` module) and the golden snapshot in
//! `tests/voxel_determinism.rs`.

use bevy::math::{DVec3, Vec3};
use bevy::prelude::warn_once;
use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};

use crate::plugins::voxel::config::{NoiseKind, PlanetConfig};

// ── Public API ────────────────────────────────────────────────────────────────

/// Evaluate the signed-distance density at a world-space position.
///
/// Returns a value where:
/// - `> 0` → inside rock / terrain (solid)
/// - `≤ 0` → air
///
/// # Arguments
/// - `pos`    — world-space position in metres (f64 for planet-scale precision)
/// - `seed`   — `WorldSeed` value (usually `0x4E45_4F4E_3230_3236`)
/// - `config` — planet configuration resource
///
/// # Determinism
/// This function is a pure mathematical mapping. It must not read the clock,
/// thread IDs, or any mutable global state.
#[inline]
pub fn evaluate_density(pos: DVec3, seed: u64, config: &PlanetConfig) -> f32 {
    let radius_m = config.radius_km * 1_000.0;

    // Guard against zero-length position to prevent divide-by-zero (edge case
    // when viewer is at exact planet centre).
    let dist = pos.length();
    if dist < 1.0 {
        warn_once!("evaluate_density called near planet centre (dist = {dist:.3} m); clamping.");
        return radius_m as f32; // treat as deep solid
    }

    // Base sphere SDF: positive inside (rock), negative outside (air).
    let mut density = (radius_m - dist) as f32;

    // Unit-sphere surface normal used as noise domain position.
    // Using the normalised world position keeps noise scale consistent at all
    // locations on the sphere, independent of planet radius.
    let n = (pos / radius_m).as_vec3();

    // Additive noise layers.
    for layer in &config.noise_layers {
        density += sample_layer(n, seed, layer);
    }

    density
}

/// Generate the full density grid for a chunk, including a 1-voxel shared border.
///
/// # Shared-border contract (FR-023 / T042)
/// Adjacent chunks at the same LOD produce **identical** density values at their
/// shared face because the sample positions are computed from absolute world
/// coordinates — not relative to the chunk origin. The grid is `(n+2)³` where
/// `n = voxels_per_edge`, giving a 1-voxel overlap on every face that the DMC
/// mesher uses to eliminate cracks between chunk boundaries.
///
/// # Arguments
/// - `chunk_origin` — world-space position of the chunk corner at (0,0,0) local space (metres)
/// - `cell_size_m`  — metres per voxel edge
/// - `voxels`       — voxels per edge (excluding border), e.g. 32 at LOD 0
/// - `seed`         — `WorldSeed` value
/// - `config`       — planet config reference
///
/// # Returns
/// A flat `Vec<f32>` of length `(voxels+2)³` with indexing
/// `[x + y*(voxels+2) + z*(voxels+2)²]`, starting 1 voxel before `chunk_origin`.
pub fn generate_chunk_densities(
    chunk_origin: DVec3,
    cell_size_m: f32,
    voxels: u32,
    seed: u64,
    config: &PlanetConfig,
) -> Vec<f32> {
    let n = (voxels + 2) as usize; // includes 1-voxel border on each face
    let mut densities = Vec::with_capacity(n * n * n);

    // Begin 1 voxel before chunk_origin to include the shared border.
    let start = chunk_origin - DVec3::splat(cell_size_m as f64);

    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let pos = start
                    + DVec3::new(
                        x as f64 * cell_size_m as f64,
                        y as f64 * cell_size_m as f64,
                        z as f64 * cell_size_m as f64,
                    );
                densities.push(evaluate_density(pos, seed, config));
            }
        }
    }

    densities
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Sample a single noise layer at the given unit-sphere position.
fn sample_layer(pos: Vec3, seed: u64, layer: &crate::plugins::voxel::config::NoiseLayer) -> f32 {
    let mut fnl = FastNoiseLite::new();

    // Fold u64 seed into i32 deterministically. XOR upper/lower 32 bits.
    fnl.set_seed(Some(((seed ^ (seed >> 32)) & 0xFFFF_FFFF) as i32));
    fnl.set_frequency(Some(layer.frequency));

    match layer.kind {
        NoiseKind::Fbm => {
            fnl.set_noise_type(Some(NoiseType::Perlin));
            fnl.set_fractal_type(Some(FractalType::FBm));
            fnl.set_fractal_octaves(Some(layer.octaves as i32));
            fnl.set_fractal_gain(Some(layer.persistence));
            fnl.set_fractal_lacunarity(Some(layer.lacunarity));
        }
        NoiseKind::Ridged => {
            fnl.set_noise_type(Some(NoiseType::Perlin));
            fnl.set_fractal_type(Some(FractalType::Ridged));
            fnl.set_fractal_octaves(Some(layer.octaves as i32));
            fnl.set_fractal_gain(Some(layer.persistence));
            fnl.set_fractal_lacunarity(Some(layer.lacunarity));
        }
        NoiseKind::Billow => {
            fnl.set_noise_type(Some(NoiseType::Perlin));
            // fastnoise-lite does not have a Billow fractal type; PingPong
            // produces a similar rounded-hills appearance.
            fnl.set_fractal_type(Some(FractalType::PingPong));
            fnl.set_fractal_octaves(Some(layer.octaves as i32));
            fnl.set_fractal_gain(Some(layer.persistence));
            fnl.set_fractal_lacunarity(Some(layer.lacunarity));
        }
    }

    fnl.get_noise_3d(pos.x, pos.y, pos.z) * layer.amplitude
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::voxel::config::PlanetConfig;

    /// Density evaluated twice at the same position with the same seed must
    /// return bit-identical results (FR-034 determinism requirement).
    #[test]
    fn evaluate_density_is_deterministic() {
        let config = PlanetConfig::default();
        let seed = 0x4E45_4F4E_3230_3236_u64;
        let pos = DVec3::new(1_234_567.0, -2_345_678.0, 3_456_789.0);

        let a = evaluate_density(pos, seed, &config);
        let b = evaluate_density(pos, seed, &config);
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "density must be bit-identical on repeat call"
        );
    }

    /// Adjacent chunks must produce equal density at their shared face (FR-023).
    ///
    /// Chunk A's last sample in X and chunk B's first sample in X (offset by
    /// exactly one chunk width) must evaluate the same world-space position,
    /// therefore produce identical density.
    #[test]
    fn adjacent_chunks_share_border_density() {
        let config = PlanetConfig::default();
        let seed = 0x4E45_4F4E_3230_3236_u64;
        let voxels = 8u32; // small grid for the test
        let cell_size = 4.0_f32; // 4 m per voxel
        let chunk_size = voxels as f64 * cell_size as f64;

        // Chunk A at origin.
        let origin_a = DVec3::new(6_370_000.0, 0.0, 0.0);
        let densities_a = generate_chunk_densities(origin_a, cell_size, voxels, seed, &config);

        // Chunk B immediately to the +X side of chunk A.
        let origin_b = origin_a + DVec3::new(chunk_size, 0.0, 0.0);
        let densities_b = generate_chunk_densities(origin_b, cell_size, voxels, seed, &config);

        let n = (voxels + 2) as usize;
        // The shared face between A and B is at world position `origin_a + chunk_size`.
        //
        // In grid A: start_a = origin_a - cell  =>  x = (chunk_size + cell) / cell = voxels + 1 = n-1
        // In grid B: start_b = origin_b - cell  =>  x = (0 + cell) / cell = 1
        //
        // Therefore A[x=n-1, y, z] == B[x=1, y, z] for all y, z.
        for z in 0..n {
            for y in 0..n {
                let idx_a = (n - 1) + y * n + z * n * n;
                let idx_b = 1 + y * n + z * n * n;
                let va = densities_a[idx_a];
                let vb = densities_b[idx_b];
                assert_eq!(
                    va.to_bits(),
                    vb.to_bits(),
                    "shared border mismatch at y={y} z={z}: a={va} b={vb}"
                );
            }
        }
    }

    #[cfg(feature = "proptest")]
    use proptest::prelude::*;

    #[cfg(feature = "proptest")]
    proptest! {
        /// Fuzz: density must be deterministic for arbitrary world positions.
        #[test]
        fn density_deterministic_fuzz(
            x in -8_000_000.0_f64..8_000_000.0,
            y in -8_000_000.0_f64..8_000_000.0,
            z in -8_000_000.0_f64..8_000_000.0,
        ) {
            let config = PlanetConfig::default();
            let seed = 0x4E45_4F4E_3230_3236_u64;
            let pos = DVec3::new(x, y, z);
            let a = evaluate_density(pos, seed, &config);
            let b = evaluate_density(pos, seed, &config);
            prop_assert_eq!(a.to_bits(), b.to_bits());
        }
    }
}
