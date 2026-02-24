//! Dual Marching Cubes lookup tables.
//!
//! Contains the constant edge-to-corner and adjacency data used by `dmc.rs`
//! to resolve active cells and emit quads from dual vertices.

// ── Corner offsets ────────────────────────────────────────────────────────────

/// The 8 unit-cube corner offsets, indexed 0–7.
///
/// Corner layout (right-hand, Y-up):
/// ```text
///   3───2       7───6
///   │   │  +Z   │   │
///   0───1       4───5
///    X →
/// ```
/// Bottom face: 0–3 (Z=0), top face: 4–7 (Z=1).
pub const CORNER_OFFSETS: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0], // 0
    [1.0, 0.0, 0.0], // 1
    [1.0, 1.0, 0.0], // 2
    [0.0, 1.0, 0.0], // 3
    [0.0, 0.0, 1.0], // 4
    [1.0, 0.0, 1.0], // 5
    [1.0, 1.0, 1.0], // 6
    [0.0, 1.0, 1.0], // 7
];

// ── Edge table ────────────────────────────────────────────────────────────────

/// The 12 edges of the unit cube as pairs of corner indices.
///
/// Edges are ordered: 4 bottom, 4 top, 4 vertical pillars.
pub const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0), // bottom face edges
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4), // top face edges
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7), // vertical edges
];

// ── Quad adjacency for dual vertices ─────────────────────────────────────────

/// For each of the 3 axis-aligned edge directions, the 4 cell offsets that
/// share that edge and therefore receive a quad connecting their dual vertices.
///
/// Each entry is `[axis, (dj0,dk0), (dj1,dk1), (dj2,dk2), (dj3,dk3)]`
/// expressed as signed offsets in the two perpendicular axes.
///
/// - Axis 0 (X-edges): shared by cells at offsets in (Y, Z)
/// - Axis 1 (Y-edges): shared by cells at offsets in (X, Z)
/// - Axis 2 (Z-edges): shared by cells at offsets in (X, Y)
///
/// Winding: counter-clockwise when viewed from +axis direction.
#[rustfmt::skip]
pub const QUAD_CCELL_OFFSETS_X: [(i32, i32); 4] = [
    ( 0,  0),
    ( 0, -1),
    (-1, -1),
    (-1,  0),
];

/// Corner-cell offsets for a Y-axis quad face.
///
/// Winding: counter-clockwise when viewed from +Y direction.
#[rustfmt::skip]
pub const QUAD_CCELL_OFFSETS_Y: [(i32, i32); 4] = [
    ( 0,  0),
    (-1,  0),
    (-1, -1),
    ( 0, -1),
];

/// Corner-cell offsets for a Z-axis quad face.
///
/// Winding: counter-clockwise when viewed from +Z direction.
#[rustfmt::skip]
pub const QUAD_CCELL_OFFSETS_Z: [(i32, i32); 4] = [
    ( 0,  0),
    ( 0, -1),
    (-1, -1),
    (-1,  0),
];
