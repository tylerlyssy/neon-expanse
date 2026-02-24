//! Hydraulic erosion simulation.
//!
//! Applied **once** per chunk at planet init, after the noise density field
//! has been evaluated. Runs on the worker thread — never on the main thread.
//!
//! The erosion model is a simplified particle-based hydraulic erosion:
//! water drops fall from high points, erode material, carry sediment downhill,
//! and deposit it at low-gradient regions. The number of passes is controlled
//! by `ErosionConfig::passes`.

use crate::plugins::voxel::config::ErosionConfig;

// ── Public API ────────────────────────────────────────────────────────────────

/// Apply hydraulic erosion to a 3-D density grid in-place.
///
/// Operates on the *surface* of the density field (cells where the density
/// crosses zero). Interior and exterior cells are left unchanged.
///
/// # Arguments
/// - `grid` — flat density grid with length `(cols * rows * layers)`.
///   Indexing: `grid[x + y * cols + z * cols * rows]`.
/// - `config` — erosion parameters (passes, erosion_rate, sediment_capacity).
/// - `cols` — number of voxels in the X direction.
/// - `rows` — number of voxels in the Y direction.
///
/// The third axis (Z / layers) is inferred as `grid.len() / (cols * rows)`.
///
/// # Panics
/// Panics in debug builds if `grid.len() % (cols * rows as usize) != 0`.
pub fn apply_erosion(grid: &mut [f32], config: &ErosionConfig, cols: u32, rows: u32) {
    let cols = cols as usize;
    let rows = rows as usize;
    debug_assert_eq!(
        grid.len() % (cols * rows),
        0,
        "grid length must be a multiple of (cols * rows)"
    );
    let layers = grid.len() / (cols * rows);

    for _pass in 0..config.passes {
        erode_pass(grid, cols, rows, layers, config);
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Run one erosion pass over the entire density grid.
fn erode_pass(grid: &mut [f32], cols: usize, rows: usize, layers: usize, config: &ErosionConfig) {
    // Collect surface cells: cells where density > 0 AND at least one
    // 6-connected neighbour has density ≤ 0 (the surface voxels).
    let idx = |x: usize, y: usize, z: usize| x + y * cols + z * cols * rows;

    let is_surface = |x: usize, y: usize, z: usize| -> bool {
        if grid[idx(x, y, z)] <= 0.0 {
            return false;
        }
        // Check 6-connected neighbours; if any are outside, this is surface.
        let neighbours: [(i32, i32, i32); 6] = [
            (-1, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ];
        for (dx, dy, dz) in neighbours {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;
            if nx < 0
                || nx >= cols as i32
                || ny < 0
                || ny >= rows as i32
                || nz < 0
                || nz >= layers as i32
            {
                return true; // border cell → treat as surface
            }
            if grid[idx(nx as usize, ny as usize, nz as usize)] <= 0.0 {
                return true;
            }
        }
        false
    };

    // For each surface voxel, compute the local gradient magnitude and
    // erode/deposit based on erosion_rate and sediment_capacity.
    let mut delta = vec![0.0_f32; grid.len()];

    for z in 1..layers.saturating_sub(1) {
        for y in 1..rows.saturating_sub(1) {
            for x in 1..cols.saturating_sub(1) {
                if !is_surface(x, y, z) {
                    continue;
                }

                let centre = grid[idx(x, y, z)];

                // Find the lowest neighbour (steepest descent).
                let neighbours = [
                    (x.wrapping_sub(1), y, z),
                    (x + 1, y, z),
                    (x, y.wrapping_sub(1), z),
                    (x, y + 1, z),
                    (x, y, z.wrapping_sub(1)),
                    (x, y, z + 1),
                ];

                let steepest = neighbours
                    .iter()
                    .filter(|&&(nx, ny, nz)| nx < cols && ny < rows && nz < layers)
                    .min_by(|&&(ax, ay, az), &&(bx, by, bz)| {
                        grid[idx(ax, ay, az)]
                            .partial_cmp(&grid[idx(bx, by, bz)])
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                if let Some(&(nx, ny, nz)) = steepest {
                    let nbr = grid[idx(nx, ny, nz)];
                    let gradient = (centre - nbr).abs();

                    if gradient > 0.0 {
                        // Erode from this cell.
                        let sediment =
                            (gradient * config.erosion_rate).min(config.sediment_capacity);
                        delta[idx(x, y, z)] -= sediment;
                        // Deposit at the lower neighbour.
                        delta[idx(nx, ny, nz)] += sediment * 0.5;
                    }
                }
            }
        }
    }

    // Apply accumulated deltas.
    for (v, d) in grid.iter_mut().zip(delta.iter()) {
        *v += d;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::voxel::config::ErosionConfig;

    #[test]
    fn erosion_reduces_high_gradient_surface() {
        // Simple 3×3×3 grid: centre cell is high, neighbours are zero.
        let mut grid = vec![0.0_f32; 27];
        grid[1 + 3 + 9] = 10.0; // centre voxel at (1,1,1) is solid: 1 + 1*3 + 1*9

        let config = ErosionConfig {
            passes: 1,
            erosion_rate: 0.5,
            sediment_capacity: 1.0,
        };
        apply_erosion(&mut grid, &config, 3, 3);

        // Centre should be reduced.
        assert!(
            grid[1 + 3 + 9] < 10.0,
            "erosion should reduce high-gradient cell"
        );
    }

    #[test]
    fn erosion_zero_passes_is_noop() {
        let original = vec![1.0_f32, 0.0, -1.0, 2.0, -0.5, 0.5, 1.5, 0.2, -0.2];
        let mut grid = original.clone();
        let config = ErosionConfig {
            passes: 0,
            erosion_rate: 0.3,
            sediment_capacity: 0.6,
        };
        apply_erosion(&mut grid, &config, 3, 3);
        assert_eq!(grid, original, "zero passes should not modify grid");
    }
}
