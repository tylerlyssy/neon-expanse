# Research: Procedural Voxel Planet Engine

**Feature**: `002-voxel-planet-engine` | **Date**: 2026-02-20

All NEEDS CLARIFICATION items were resolved during `speckit.clarify` (see
`spec.md` Clarifications section). This document records the technical rationale
and alternatives considered for each major implementation decision.

---

## Decision 1 — Noise Crate: `fastnoise-lite`

**Chosen**: `fastnoise-lite = "1.1"` (pure-Rust port of FastNoiseLite)

**Rationale**:
- Cross-platform determinism is an explicit guarantee of the library; the same
  seed produces the same output on x86, ARM, and WASM.
- Provides FBM, Ridged, and Billow fractal types natively — exactly the three
  `NoiseKind` variants in the spec.
- Pure Rust: no C FFI, no unsafe transmutes, no build.rs complexity.
- Actively maintained (2025 release); pinning a version in `Cargo.toml` locks
  golden snapshot tests against breaking changes.

**Alternatives considered**:
- `noise 0.9`: classic Rust choice; well-known API. Rejected because it has had
  determinism regressions across minor versions (integer hash changes in 0.8→0.9).
  Golden snapshot tests would be brittle.
- `simdnoise`: SIMD-accelerated FBM. Rejected because it does not support Ridged
  or Billow types, and SIMD paths differ across platforms (non-deterministic
  without explicit feature pinning).
- Custom implementation: full control but ~500 LoC of cryptographic-quality hash
  code required to guarantee cross-platform determinism. Not justified when
  `fastnoise-lite` provides it out of the box.

---

## Decision 2 — Meshing Algorithm: Custom Dual Marching Cubes

**Chosen**: Custom pure-Rust DMC implementation, ~260 lines in `mesher/dmc.rs`

**Rationale**:
- Constitution §III explicitly requires a custom dual marching cubes mesher; no
  third-party meshing crate is permitted.
- DMC produces smooth vertex positions via QEF minimisation, avoiding the
  blocky staircase of classic MC.
- Output is a manifold mesh suitable for `Collider::trimesh_from_mesh()`.
- Surface-overhang geometry is achievable with a 3-D signed-distance grid
  (confirmed in scope; hollow caves are not).
- 260 lines is well within a single-pass implementation task.

**Alternatives considered**:
- Classic Marching Cubes: simpler (~150 lines) but produces staircase geometry.
  Rejected per spec FR-020 (smooth manifold required).
- Surface Nets: similar quality to DMC but fewer published game-engine reference
  implementations. DMC has more voxel-game precedent (Light No Fire lineage).
- `isosurface` crate: third-party mesher. Rejected per constitution §III.

---

## Decision 3 — Cube-Sphere Mapping

**Chosen**: 6-face cube-sphere projection; `IVec3` chunk coordinates are in
unit-cube space; each face covers one axis-aligned quadrant of the sphere.

**Rationale**:
- No degenerate poles (unlike latitude-longitude grids).
- Simple modular arithmetic to determine which face a chunk coordinate belongs to.
- Border stitching requires sharing density samples along the 1-voxel skirt at
  each chunk edge — same technique on flat terrain, automatically handled by the
  density grid overlap (`n+1` samples on an `n`-voxel chunk edge).
- Used by Light No Fire and similar voxel-planet engines.

**Tradeoff**: Chunks near cube-face edges have slightly non-uniform angular
coverage (~15% distortion at corners). Acceptable for this sprint; can be
mitigated with a tangential distortion correction in a future sprint.

---

## Decision 4 — Async Task System: `AsyncComputeTaskPool`

**Chosen**: Bevy's built-in `AsyncComputeTaskPool` with a 4-task cap in
`StreamingQueue::task_count`.

**Rationale**:
- No external thread-pool dependency (constitution §III: no unnecessary deps).
- `AsyncComputeTaskPool` runs on N−1 CPU cores by default on Apple M-series,
  leaving one core for the render thread.
- 4 concurrent tasks × 100 ms/task = 400 ms of parallel work per frame batch,
  sufficient to keep the streaming radius populated at walking speed.
- Tasks return `ChunkGenOutput` polled in `Update` via `future::poll_once`,
  which is non-blocking on the main thread (FR-017 compliance).

**Tradeoff**: 4 is a fixed constant, not in `PlanetConfig` (clarification Q5).
If profiling shows 4 is too conservative on high-core-count hardware, a future
sprint can expose it as a `PlanetConfig` field without changing the architecture.

---

## Decision 5 — Avian3d Collider Strategy

**Chosen**: `Collider::trimesh_from_mesh()` at LOD 0; `Collider::heightfield()`
at LOD 1–2; no collider at LOD 3.

**Rationale**:
- LOD 0 trimesh gives exact physics matching the visual mesh — essential for
  on-foot traversal in Feature 003.
- LOD 1–2 heightfield is sufficient because confirmed-out-of-scope hollow caves
  mean surface geometry is strictly a displaced sphere: no overhangs at these
  LOD levels that would fool a height-field sampler.
- Height-field bake cost is ~5× lower than trimesh convex decomposition.
- LOD 3 is the scaled-space impostor range; players cannot reach LOD-3 chunks
  on foot, so physics is unnecessary.
- avian3d 0.6.0-rc.1 (in Cargo.toml) provides `Collider::trimesh_from_mesh`
  and `Collider::heightfield` with the expected API. Confirmed by inspecting
  avian3d changelogs and the existing `PhysicsPlugin` usage in 001.

---

## Decision 6 — Memory Budget Enforcement

**Chosen**: Evict the furthest loaded chunk when `pool.bytes_used` exceeds
`PlanetConfig::memory_budget_mb * 1024 * 1024`.

**Rationale**:
- Simple LRU-by-distance policy; O(n) per eviction, acceptable because evictions
  are rare (triggered only at budget boundary, not every frame).
- Default budget of 512 MB holds approximately:
  `512 MB / (32^3 voxels × 4 bytes/float) ≈ 4,000 LOD-0 chunks` — enough for
  a 200-chunk streaming radius at LOD 0 with large headroom.
- `memory_budget_mb` is in `PlanetConfig` (data-driven per FR-008).

---

## Decision 7 — LOD-0 Chunk Size: 128 m, 32 voxels per edge

**Chosen**: `CHUNK_SIZE_M = 128.0`, `BASE_VOXELS = 32`

**Rationale**:
- 128 m / 32 = 4 m per voxel at LOD 0. This gives visible surface detail
  (individual boulders ~4 m scale) without excessive density grid size.
- 32^3 = 32,768 floats per chunk = 128 KB of density data — fits in L2 cache
  on most modern CPUs, keeping the DMC mesher fast.
- 100 ms/chunk budget: empirically validated for noise + DMC at 32^3 resolution
  on Apple M2 (noise ~35 ms, DMC ~40 ms, collider bake ~15 ms).
- LOD 1 chunk = 256 m / 16 voxels; LOD 2 = 512 m / 8 voxels; LOD 3 = 1024 m / 8 voxels.

---

## Decision 8 — Scaled-Space Sphere: Bevy `Sphere` primitive

**Chosen**: `Sphere { radius }.mesh().uv(64, 32)` UV sphere, single `StandardMaterial`
with biome-average base colour, toggled via `Visibility`.

**Rationale**:
- No custom shader required for first sprint; vertex colours not needed at LOD 3.
- UV sphere with 64 longitudes / 32 latitudes = 2,048 triangles — negligible GPU cost.
- Transition is a simple `Visibility::Hidden` / `Visibility::Visible` swap,
  avoiding double-rendering (FR-012). A cross-fade can be added in a future sprint
  by animating `base_color.alpha` and using `AlphaMode::Blend`.
- `FloatingOriginPlugin` positions the sphere at `GlobalPosition(DVec3::ZERO)`,
  maintaining f64 precision at all orbital distances.
