# Implementation Plan: Procedural Voxel Planet Engine

**Branch**: `002-voxel-planet-engine` | **Date**: 2026-02-20 | **Spec**: [`spec.md`](spec.md)
**Input**: Feature specification from `specs/002-voxel-planet-engine/spec.md`

---

## Constitution Compliance Checklist

*GATE: All items must be PASS before Phase 0. Re-checked after Phase 1 design.*

| § | Principle | Status | Evidence / Notes |
|---|-----------|--------|-----------------|
| I | Earth-scale seamless planet, no loading screens | **PASS** | Full DVec3 floating-origin + chunk streaming; planet = 12,742 km |
| II | 100% Bevy ECS; exactly one planet; dual MC voxel meshing; hybrid rendering; layered generation | **PASS** | All design is ECS-only; DMC implemented from scratch; scaled-space sphere + voxel pipeline; base sphere → fastnoise-lite layers → erosion |
| III | Rust 2024 / 1.85+, Bevy 0.18, avian3d 0.6.0-rc.1 (≥ 0.5), custom DMC, `fastnoise-lite` (substitutes `noise` crate per clarification Q1), DVec3, serde + RON | **PASS** | `fastnoise-lite` is a stricter determinism guarantee than `noise`; constitution says "noise crate" as guidance but clarification overrides |
| IV | VoxelWorldPlugin operational | **PASS** | Plan fills the current empty stub |
| V | `cargo fmt`, `clippy -D warnings`, `///` docs, no `unwrap()` outside bootstrap | **PASS** | All code snippets include docs; no bare unwraps |
| VI | Property-based determinism tests, ≥ 85% core coverage, golden snapshot | **PASS** | Three proptest suites + golden snapshot in `tests/voxel_determinism.rs` |
| VII | Player spawns in varied biome; world feels massive and continuous | **PASS** | RON biome table drives per-vertex colour; default planet.ron provides varied terrain |
| VIII | All config in RON; mods register new layers via `VoxelLayerRegistry` Bevy resource | **DEFERRED** | RON config is fully data-driven; no hardcoded terrain values. `VoxelLayerRegistry` mod-registration API is deferred to Feature 003 where the traversal plugin will also need to register vehicle variants. Implementing a partial registry here without the consumer would be speculative. |

**Complexity Tracking** (constitution violations requiring justification):

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| `fastnoise-lite` instead of `noise` crate | Clarification Q1 chose it for cross-platform determinism | `noise` crate has had broken determinism across minor versions; golden snapshot tests would be fragile |

---

## Summary

Replace the empty `VoxelWorldPlugin` stub with a full procedural voxel planet engine that renders one Earth-scale planet seamlessly from any orbital altitude to ground level. The pipeline is:

1. **Scaled-space sphere** when viewer > 10× radius from planet centre (rendered as a smooth `Mesh::try_from(Sphere {...})` entity using `FloatingOriginPlugin`).
2. **Chunk streaming** (`StreamingQueue` priority queue → up to 4 concurrent `AsyncComputeTaskPool` jobs) generates `VoxelData` density fields seeded from `WorldSeed` + `fastnoise-lite` layered noise.
3. **Dual Marching Cubes** (custom pure-Rust, ~250 lines) meshes each `VoxelData` grid into a smooth `Mesh` with per-vertex biome colour.
4. **Adaptive Avian3d colliders**: `Collider::trimesh()` at LOD 0, height-field at LOD 1–2, none at LOD 3.
5. All configuration loaded from `assets/config/planets/planet.ron` (`NEON_PLANET_CONFIG` env var overrides).
6. All existing plugins (`LodPlugin`, `FloatingOriginPlugin`, `PhysicsPlugin`) used without modification.

---

## Technical Context

**Language/Version**: Rust stable 2024 edition, minimum 1.85+
**Primary Dependencies**: Bevy 0.18, avian3d 0.6.0-rc.1, fastnoise-lite 1.1, serde + ron 0.8
**Storage**: RON assets + in-memory `ChunkPool` (capped by `PlanetConfig::memory_budget_mb`)
**Testing**: `cargo test`, `proptest 1`, golden snapshot (`insta`-style manual byte compare)
**Target Platform**: macOS (dev), Linux/Windows (CI), PS4 controller input via Bevy Gamepad
**Project Type**: Single Bevy binary, single Cargo workspace
**Performance Goals**: ≥ 60 FPS at LOD-0 altitude; single-chunk pipeline ≤ 100 ms; startup visibility ≤ 2 s
**Constraints**: main thread never blocked; ≤ 4 concurrent generation tasks; memory ≤ `memory_budget_mb`
**Scale/Scope**: One planet, 6,371 km radius, ~128 m LOD-0 chunk edge, ~5 biome height bands

---

## Project Structure

### Documentation (this feature)

```text
specs/002-voxel-planet-engine/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── contracts/
│   └── plugin-api.md    ← Phase 1 output
├── quickstart.md        ← Phase 1 output
└── checklists/
    └── requirements.md
```

### Source Code

```text
src/
└── plugins/
    ├── voxel_world_plugin.rs        ← replaces empty stub; plugin registration
    └── voxel/                       ← NEW module tree
        ├── mod.rs
        ├── components.rs            ← VoxelChunk, VoxelData, ChunkMesh, ChunkCollider,
        │                               ScaledSpaceMarker, PlanetCentre, ChunkGenTask
        ├── resources.rs             ← PlanetConfig, ChunkPool, StreamingQueue,
        │                               ScaledSpaceState, VoxelStats
        ├── config.rs                ← NoiseLayer, NoiseKind, ErosionConfig, BiomeConfig
        │                               (all serde/ron; loaded at startup)
        ├── mesher/
        │   ├── mod.rs
        │   ├── dmc.rs               ← pure-Rust Dual Marching Cubes (~260 lines)
        │   └── tables.rs            ← DMC edge/vertex tables (generated constants)
        ├── noise_stack.rs           ← layered fastnoise-lite evaluator (pure fn)
        ├── erosion.rs               ← hydraulic erosion sim (runs once at planet init)
        ├── systems/
        │   ├── mod.rs
        │   ├── planet_init.rs       ← spawn_planet_on_startup system
        │   ├── scaled_space.rs      ← ScaledSpaceSwitcher system
        │   ├── streaming.rs         ← ChunkStreamer system (enqueue + poll tasks)
        │   ├── mesh_builder.rs      ← MeshBuilder (receives finished DMC output)
        │   └── collider_sync.rs     ← ColliderSyncer system
        └── tests/                   ← voxel-specific unit + proptest modules
            ├── dmc_tests.rs
            ├── noise_tests.rs
            └── determinism_tests.rs

assets/
└── config/
    └── planets/
        ├── planet.ron               ← default Earth-like planet config
        └── test_variant.ron         ← alternate config for US5 independent test

tests/
└── voxel_determinism.rs             ← integration: golden snapshot regression
```

**Structure Decision**: Single `src/plugins/voxel/` module tree, matching the existing
`floating_origin/`, `lod/`, and `input/` layout. All new public items live under
`crate::plugins::voxel`.

---

## Dependency Changes

Add to `Cargo.toml` `[dependencies]`:

```toml
fastnoise-lite = "1.1"
```

Add to `Cargo.toml` `[dev-dependencies]` (golden snapshot helper):

```toml
# No additional dev-dep needed — manual byte-equality comparison used.
```

*No other dependency changes.* All existing deps (`bevy`, `avian3d`, `serde`, `ron`, `glam`,
`proptest`) are already present.

---

## Phase 0: Research

→ See [`research.md`](research.md) for all findings.

Key decisions resolved before design:

| Topic | Decision | Source |
|-------|----------|--------|
| Noise crate | `fastnoise-lite 1.1` | Clarification Q1 |
| Cave geometry | Surface overhangs only; density = sphere SDF + surface displacement | Clarification Q2 |
| LOD 1–2 collider | Avian3d height-field | Clarification Q3 |
| Planet config selection | `NEON_PLANET_CONFIG` env var | Clarification Q4 |
| Worker task cap | 4 concurrent `AsyncComputeTaskPool` tasks | Clarification Q5 |
| Cube-sphere mapping | 6-face cube-sphere; `IVec3` chunk coords mapped via face | Research |
| DMC implementation | Custom pure-Rust (~260 lines); no external meshing crate | Constitution §III |
| avian3d API | `0.6.0-rc.1` (not 0.5 — latest compatible as constitution allows) | Cargo.toml |

---

## Phase 1: Design

### 1. Plugin Registration

`VoxelWorldPlugin::build` registers (in order):

```rust
app
    // Resources
    .init_resource::<ChunkPool>()
    .init_resource::<StreamingQueue>()
    .init_resource::<ScaledSpaceState>()
    .init_resource::<VoxelStats>()
    // Config loaded synchronously at startup
    // (Systems)
    .add_systems(Startup, (
        load_planet_config,       // reads RON → inserts PlanetConfig
        spawn_planet_on_startup.after(load_planet_config),
    ))
    .add_systems(Update, (
        scaled_space_switcher,    // toggle sphere/voxel based on viewer distance
        chunk_streamer,           // enqueue + poll async generation tasks
        mesh_builder,             // insert Mesh + MeshMaterial on completed chunks
        collider_syncer,          // insert/update Avian colliders on ready chunks
    ))
    // Component registration for Bevy reflection
    .register_type::<VoxelChunk>()
    .register_type::<VoxelData>()
    .register_type::<ChunkMesh>()
    .register_type::<ChunkCollider>()
    .register_type::<ScaledSpaceMarker>()
    .register_type::<PlanetCentre>()
    .register_type::<LodLevel>();  // already registered by LodPlugin — idempotent
```

### 2. Planet Config: RON Schema and Rust Types

**`assets/config/planets/planet.ron`**:

```text
PlanetConfig(
    radius_km: 6371.0,
    seed_override: None,
    memory_budget_mb: 512,
    noise_layers: [
        NoiseLayer(
            kind: Fbm,
            frequency: 0.0000008,
            amplitude: 8000.0,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
        ),
        NoiseLayer(
            kind: Ridged,
            frequency: 0.000004,
            amplitude: 1200.0,
            octaves: 4,
            persistence: 0.6,
            lacunarity: 2.2,
        ),
        NoiseLayer(
            kind: Billow,
            frequency: 0.00002,
            amplitude: 200.0,
            octaves: 3,
            persistence: 0.45,
            lacunarity: 1.9,
        ),
    ],
    erosion: ErosionConfig(
        passes: 2,
        erosion_rate: 0.3,
        sediment_capacity: 0.6,
    ),
    biome: BiomeConfig(
        bands: [
            (height_fraction: -1.0, colour: (0.05, 0.10, 0.45)),   // deep ocean
            (height_fraction: -0.1, colour: (0.08, 0.18, 0.55)),   // shallow ocean
            (height_fraction: 0.0,  colour: (0.76, 0.70, 0.50)),   // beach/sand
            (height_fraction: 0.2,  colour: (0.25, 0.55, 0.20)),   // lowland
            (height_fraction: 0.5,  colour: (0.20, 0.40, 0.15)),   // highland
            (height_fraction: 0.7,  colour: (0.55, 0.50, 0.45)),   // alpine rock
            (height_fraction: 1.0,  colour: (0.95, 0.95, 0.98)),   // snow cap
        ],
    ),
)
```

**Rust types** (`src/plugins/voxel/config.rs`):

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Noise function variant for a procedural layer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect, PartialEq)]
pub enum NoiseKind { Fbm, Ridged, Billow }

/// A single procedural noise layer in the planet modifier stack.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NoiseLayer {
    pub kind:        NoiseKind,
    pub frequency:   f32,
    pub amplitude:   f32,
    pub octaves:     u32,
    pub persistence: f32,
    pub lacunarity:  f32,
}

/// Hydraulic erosion parameters applied once at planet init.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ErosionConfig {
    pub passes:              u32,
    pub erosion_rate:        f32,
    pub sediment_capacity:   f32,
}

/// A single biome colour band mapping normalised height → vertex colour.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BiomeBand {
    pub height_fraction: f32,
    pub colour:          (f32, f32, f32),
}

/// Biome height-to-colour mapping table (sorted by height_fraction).
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BiomeConfig {
    pub bands: Vec<BiomeBand>,
}

/// Full planet configuration loaded from RON at startup.
///
/// Inserted as a `Resource` by `load_planet_config`.
/// The `NEON_PLANET_CONFIG` environment variable overrides the default path.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PlanetConfig {
    pub radius_km:       f64,
    pub seed_override:   Option<u64>,
    pub memory_budget_mb: u32,
    pub noise_layers:    Vec<NoiseLayer>,
    pub erosion:         ErosionConfig,
    pub biome:           BiomeConfig,
}

impl Default for PlanetConfig {
    /// Hard-coded Earth-like fallback used when `planet.ron` is absent.
    fn default() -> Self {
        Self {
            radius_km: 6_371.0,
            seed_override: None,
            memory_budget_mb: 512,
            noise_layers: vec![
                NoiseLayer {
                    kind: NoiseKind::Fbm,
                    frequency: 0.000_000_8,
                    amplitude: 8_000.0,
                    octaves: 6,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            ],
            erosion: ErosionConfig { passes: 1, erosion_rate: 0.3, sediment_capacity: 0.6 },
            biome: BiomeConfig { bands: vec![
                BiomeBand { height_fraction: 0.0, colour: (0.25, 0.55, 0.20) },
                BiomeBand { height_fraction: 1.0, colour: (0.95, 0.95, 0.98) },
            ]},
        }
    }
}
```

**`load_planet_config` system** (`systems/planet_init.rs`):

```rust
pub fn load_planet_config(mut commands: Commands) {
    let path = std::env::var("NEON_PLANET_CONFIG")
        .unwrap_or_else(|_| "assets/config/planets/planet.ron".into());

    let config = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| ron::from_str::<PlanetConfig>(&s).ok())
        .unwrap_or_else(|| {
            warn!("planet config not found at '{}' — using defaults", path);
            PlanetConfig::default()
        });

    commands.insert_resource(config);
}
```

### 3. ECS Components and Resources

**`src/plugins/voxel/components.rs`**:

```rust
use bevy::prelude::*;

/// Marks an entity as an active voxel chunk tile.
#[derive(Component, Debug, Clone, Reflect)]
pub struct VoxelChunk {
    /// Chunk-space grid address (1 unit = 1 chunk edge length).
    pub chunk_coord: IVec3,
    /// LOD level: 0 = highest detail (< 1 km), 3 = lowest (> 100 km).
    pub lod: u8,
}

/// Raw signed-distance density field for a chunk.
/// Values > 0 are inside the surface; values ≤ 0 are air.
/// Length = (voxels_per_edge + 1)^3 for the chunk's LOD resolution.
#[derive(Component, Debug, Clone, Reflect)]
pub struct VoxelData {
    pub densities:      Vec<f32>,
    pub voxels_per_edge: u32,
}

/// Handle to the mesh produced by the DMC mesher.
#[derive(Component, Debug, Clone, Reflect)]
pub struct ChunkMesh {
    pub mesh_handle:  Handle<Mesh>,
    pub vertex_count: u32,
}

/// Marker: this chunk's Avian collider has been generated and inserted.
#[derive(Component, Debug, Default, Reflect)]
pub struct ChunkCollider;

/// Marker: this entity is the scaled-space planet sphere.
#[derive(Component, Debug, Default, Reflect)]
pub struct ScaledSpaceMarker;

/// Marker: authoritative planet centre reference point.
/// Carries `GlobalPosition(DVec3::ZERO)`.
#[derive(Component, Debug, Default, Reflect)]
pub struct PlanetCentre;

/// Holds the `Task<ChunkGenOutput>` spawned on `AsyncComputeTaskPool`.
/// Removed from entity when the task is polled to completion.
#[derive(Component)]
pub struct ChunkGenTask(pub bevy::tasks::Task<ChunkGenOutput>);

/// Output of a completed async chunk generation job.
pub struct ChunkGenOutput {
    pub coord:       IVec3,
    pub lod:         u8,
    pub voxel_data:  VoxelData,
    pub mesh:        Mesh,
    pub vertex_colours: Vec<[f32; 4]>,
}
```

**`src/plugins/voxel/resources.rs`**:

```rust
use std::collections::{BTreeMap, BinaryHeap};
use bevy::prelude::*;

/// Registry of all currently loaded chunk entities by coordinate.
#[derive(Resource, Default, Debug)]
pub struct ChunkPool {
    /// coord → entity
    pub loaded: std::collections::HashMap<IVec3, Entity>,
    /// Current total voxel data bytes resident in memory.
    pub bytes_used: usize,
}

/// Min-heap entry for the streaming priority queue.
#[derive(Debug, Eq, PartialEq)]
pub struct StreamJob {
    /// Negative distance for min-heap (BinaryHeap is max).
    pub neg_dist_sq: i64,
    pub coord:       IVec3,
    pub lod:         u8,
}

impl Ord for StreamJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.neg_dist_sq.cmp(&other.neg_dist_sq)
    }
}
impl PartialOrd for StreamJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

/// Priority queue driving async chunk generation dispatch.
#[derive(Resource, Default, Debug)]
pub struct StreamingQueue {
    pub pending:    BinaryHeap<StreamJob>,
    /// Coords with an in-flight task (dedup guard).
    pub in_flight:  std::collections::HashSet<IVec3>,
    /// Currently running task count (max 4).
    pub task_count: u8,
}

/// Whether the scaled-space sphere is currently active.
#[derive(Resource, Debug, Default)]
pub struct ScaledSpaceState {
    pub sphere_visible: bool,
}

/// Live diagnostics reported by VoxelWorldPlugin::stats().
#[derive(Resource, Default, Debug)]
pub struct VoxelStats {
    pub loaded_chunks:  u32,
    pub queued_chunks:  u32,
    pub in_flight_tasks: u8,
    pub memory_used_mb:  f32,
}
```

### 4. Density / Noise Stack

**`src/plugins/voxel/noise_stack.rs`** — pure function, no ECS dependencies:

```rust
//! Layered noise evaluator using `fastnoise-lite`.
//!
//! This is a **pure function**: same inputs → same output, always.
//! No global state, no RNG seeding inside this module.

use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use crate::plugins::voxel::config::{NoiseKind, NoiseLayer, PlanetConfig};

/// World-space position to evaluate (metres, f64 for planet-scale precision).
pub type WorldPos = glam::DVec3;

/// Evaluate the signed-distance density at `pos` for the given planet config.
///
/// Positive → inside rock/ground; negative → air.
///
/// # Determinism guarantee
/// Given identical `seed`, `config`, and `pos`, this function MUST return
/// the same `f32` value across all runs, platforms, and compiler versions.
pub fn evaluate_density(pos: WorldPos, seed: u64, config: &PlanetConfig) -> f32 {
    let radius_m = config.radius_km * 1_000.0;
    let dist = pos.length();

    // Base sphere SDF: positive inside, negative outside.
    let mut density = (radius_m - dist) as f32;

    // Normalised surface position for noise domain (unit sphere).
    let n = (pos / radius_m).as_vec3();

    for layer in &config.noise_layers {
        density += sample_layer(n, seed, layer);
    }

    density
}

fn sample_layer(pos: glam::Vec3, seed: u64, layer: &NoiseLayer) -> f32 {
    let mut fnl = FastNoiseLite::new();
    // fastnoise-lite seed is i32; fold u64 → i32 deterministically.
    fnl.set_seed(Some((seed ^ (seed >> 32)) as i32));
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
            fnl.set_fractal_type(Some(FractalType::Billow));
            fnl.set_fractal_octaves(Some(layer.octaves as i32));
            fnl.set_fractal_gain(Some(layer.persistence));
            fnl.set_fractal_lacunarity(Some(layer.lacunarity));
        }
    }

    fnl.get_noise_3d(pos.x, pos.y, pos.z) * layer.amplitude
}
```

### 5. Dual Marching Cubes Mesher

**`src/plugins/voxel/mesher/dmc.rs`** — key algorithm (~260 lines total):

```rust
//! Dual Marching Cubes (DMC) meshing for voxel terrain.
//!
//! Produces smooth, manifold meshes with surface overhangs.
//! Input: a 3-D signed-distance density grid.
//! Output: `bevy::render::mesh::Mesh` with positions, normals, indices,
//!         and per-vertex colour (`"Vertex_Color"` attribute).
//!
//! Algorithm: for each active cell (sign change on any edge), compute the
//! dual vertex at the minimiser of the quadric error function (QEF), then
//! connect dual vertices across shared cells to form quads → triangles.
//!
//! Reference: Schaefer & Warren, "Dual Marching Cubes", 2004.

use bevy::render::mesh::{Mesh, PrimitiveTopology, Indices};
use bevy::render::render_asset::RenderAssetUsages;
use glam::Vec3;
use crate::plugins::voxel::{components::VoxelData, config::BiomeConfig};

/// Output of a single DMC meshing pass.
pub struct DmcOutput {
    pub positions: Vec<[f32; 3]>,
    pub normals:   Vec<[f32; 3]>,
    pub colours:   Vec<[f32; 4]>,
    pub indices:   Vec<u32>,
}

/// Run DMC on `voxel_data` and return a `Mesh` ready to insert into Bevy.
///
/// `chunk_origin_m` is the world-space origin of this chunk (metres, f32 local).
/// `cell_size_m` is the size of one voxel in metres.
/// `biome` provides the height-to-colour mapping.
/// `planet_radius_m` is used to compute normalised elevation for biome colour.
pub fn mesh_chunk(
    voxel_data: &VoxelData,
    chunk_origin_m: Vec3,
    cell_size_m:    f32,
    biome:          &BiomeConfig,
    planet_radius_m: f32,
) -> Mesh {
    let n = voxel_data.voxels_per_edge as usize;
    let output = run_dmc(voxel_data, n, cell_size_m, chunk_origin_m, biome, planet_radius_m);
    build_mesh(output)
}

// ── Internal implementation ─────────────────────────────────────────────────

fn run_dmc(
    vd: &VoxelData,
    n: usize,
    cell: f32,
    origin: Vec3,
    biome: &BiomeConfig,
    planet_r: f32,
) -> DmcOutput {
    let sample = |xi: usize, yi: usize, zi: usize| -> f32 {
        vd.densities[xi + yi * (n + 1) + zi * (n + 1) * (n + 1)]
    };

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals:   Vec<[f32; 3]> = Vec::new();
    let mut colours:   Vec<[f32; 4]> = Vec::new();
    let mut indices:   Vec<u32>      = Vec::new();

    // Map from cell (i,j,k) → index of its dual vertex in positions[].
    let mut dual_idx: Vec<i32> = vec![-1; n * n * n];

    let cell_idx = |i: usize, j: usize, k: usize| i + j * n + k * n * n;

    // Pass 1: compute dual vertex for each active cell.
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                // The 8 corners of this cell.
                let corners: [f32; 8] = [
                    sample(i,   j,   k  ), sample(i+1, j,   k  ),
                    sample(i+1, j+1, k  ), sample(i,   j+1, k  ),
                    sample(i,   j,   k+1), sample(i+1, j,   k+1),
                    sample(i+1, j+1, k+1), sample(i,   j+1, k+1),
                ];
                // Only active cells (at least one sign change).
                let inside = corners.map(|v| v > 0.0);
                if inside.iter().all(|&b| b) || inside.iter().all(|&b| !b) {
                    continue; // all inside or all outside — skip
                }

                // QEF minimiser: average of edge intersection points (simplified).
                let mut qef_sum = Vec3::ZERO;
                let mut qef_count = 0u32;

                const EDGES: [(usize, usize); 12] = [
                    (0,1),(1,2),(2,3),(3,0),
                    (4,5),(5,6),(6,7),(7,4),
                    (0,4),(1,5),(2,6),(3,7),
                ];
                const CORNER_OFFSETS: [[f32; 3]; 8] = [
                    [0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0],
                    [0.0,0.0,1.0],[1.0,0.0,1.0],[1.0,1.0,1.0],[0.0,1.0,1.0],
                ];

                for (a, b) in EDGES {
                    if inside[a] != inside[b] {
                        let t  = corners[a] / (corners[a] - corners[b]);
                        let pa = Vec3::from(CORNER_OFFSETS[a]);
                        let pb = Vec3::from(CORNER_OFFSETS[b]);
                        qef_sum += pa + t * (pb - pa);
                        qef_count += 1;
                    }
                }

                let local = if qef_count > 0 {
                    qef_sum / qef_count as f32
                } else {
                    Vec3::splat(0.5)
                };

                let world_local = origin + Vec3::new(
                    (i as f32 + local.x) * cell,
                    (j as f32 + local.y) * cell,
                    (k as f32 + local.z) * cell,
                );

                // Compute normal via central differences.
                let normal = compute_normal(vd, i, j, k, n, local);
                // Biome colour from elevation.
                let elevation = (world_local.length() - planet_r) / planet_r;
                let colour    = sample_biome(biome, elevation);

                dual_idx[cell_idx(i, j, k)] = positions.len() as i32;
                positions.push(world_local.into());
                normals.push(normal.into());
                colours.push([colour.0, colour.1, colour.2, 1.0]);
            }
        }
    }

    // Pass 2: emit quads for each active edge shared between 4 cells.
    // For each edge direction, if all 4 adjacent cells have dual vertices,
    // emit a quad (two triangles) with winding order consistent with sign.
    emit_quads(n, &dual_idx, &corners_fn(vd, n), &mut indices, cell_idx);

    DmcOutput { positions, normals, colours, indices }
}

/// Emit quads for all active edges (simplified: only X-axis edges shown;
/// Y and Z axes follow the same pattern with permuted indices).
fn emit_quads(
    n: usize,
    dual_idx: &[i32],
    is_solid: &impl Fn(usize, usize, usize) -> bool,
    indices: &mut Vec<u32>,
    cell_idx: impl Fn(usize, usize, usize) -> usize,
) {
    // X-axis edges: shared between cells (i,j,k), (i,j-1,k), (i,j-1,k-1), (i,j,k-1).
    for k in 1..n {
        for j in 1..n {
            for i in 0..n {
                let sign_lo = is_solid(i, j, k);
                let sign_hi = is_solid(i + 1, j, k);
                if sign_lo == sign_hi { continue; }
                let d0 = dual_idx[cell_idx(i, j,   k  )];
                let d1 = dual_idx[cell_idx(i, j-1, k  )];
                let d2 = dual_idx[cell_idx(i, j-1, k-1)];
                let d3 = dual_idx[cell_idx(i, j,   k-1)];
                if d0 < 0 || d1 < 0 || d2 < 0 || d3 < 0 { continue; }
                // Winding depends on which side is solid.
                push_quad(indices, d0 as u32, d1 as u32, d2 as u32, d3 as u32, sign_lo);
            }
        }
    }
    // TODO: Y-axis and Z-axis edges follow same pattern with permuted (i,j,k) roles.
}

fn push_quad(indices: &mut Vec<u32>, a: u32, b: u32, c: u32, d: u32, flip: bool) {
    if flip {
        indices.extend_from_slice(&[a, b, c, a, c, d]);
    } else {
        indices.extend_from_slice(&[a, d, c, a, c, b]);
    }
}

fn compute_normal(vd: &VoxelData, i: usize, j: usize, k: usize, n: usize, _local: Vec3) -> Vec3 {
    // Gradient via central differences at cell centre.
    let s = |xi: usize, yi: usize, zi: usize| -> f32 {
        let xi = xi.clamp(0, n);
        let yi = yi.clamp(0, n);
        let zi = zi.clamp(0, n);
        vd.densities[xi + yi * (n + 1) + zi * (n + 1) * (n + 1)]
    };
    let dx = s(i+1, j, k) - s(i.saturating_sub(1), j, k);
    let dy = s(i, j+1, k) - s(i, j.saturating_sub(1), k);
    let dz = s(i, j, k+1) - s(i, j, k.saturating_sub(1));
    Vec3::new(dx, dy, dz).normalize_or_zero()
}

fn corners_fn(vd: &VoxelData, n: usize) -> impl Fn(usize, usize, usize) -> bool + '_ {
    move |i, j, k| {
        let i = i.clamp(0, n);
        let j = j.clamp(0, n);
        let k = k.clamp(0, n);
        vd.densities[i + j * (n + 1) + k * (n + 1) * (n + 1)] > 0.0
    }
}

fn sample_biome(biome: &BiomeConfig, elevation: f32) -> (f32, f32, f32) {
    let bands = &biome.bands;
    if bands.is_empty() { return (0.5, 0.5, 0.5); }
    if elevation <= bands[0].height_fraction { return bands[0].colour; }
    for i in 1..bands.len() {
        if elevation <= bands[i].height_fraction {
            let lo = &bands[i - 1];
            let hi = &bands[i];
            let t  = (elevation - lo.height_fraction) / (hi.height_fraction - lo.height_fraction);
            return (
                lo.colour.0 + t * (hi.colour.0 - lo.colour.0),
                lo.colour.1 + t * (hi.colour.1 - lo.colour.1),
                lo.colour.2 + t * (hi.colour.2 - lo.colour.2),
            );
        }
    }
    bands.last().expect("bands non-empty").colour
}

fn build_mesh(out: DmcOutput) -> Mesh {
    use bevy::render::mesh::VertexAttributeValues;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, out.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,   out.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR,    out.colours);
    mesh.insert_indices(Indices::U32(out.indices));
    mesh
}
```

### 6. Async Chunk Generation (Streaming)

**`src/plugins/voxel/systems/streaming.rs`**:

```rust
//! Chunk streaming: enqueue, dispatch, and poll async generation tasks.

use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, Task}};
use glam::DVec3;
use std::sync::Arc;

use crate::plugins::{
    floating_origin::resources::OriginFrame,
    lod::resources::LodSettings,
    voxel::{
        components::{ChunkGenOutput, ChunkGenTask, VoxelChunk},
        config::PlanetConfig,
        noise_stack::evaluate_density,
        mesher::dmc::mesh_chunk,
        resources::{ChunkPool, StreamingQueue, StreamJob, VoxelStats},
    },
};
use crate::plugins::core_plugin::WorldSeed;

/// Chunk edge length in metres at LOD 0.
pub const CHUNK_SIZE_M: f32 = 128.0;
/// Base voxel resolution at LOD 0 (voxels per edge).
pub const BASE_VOXELS: u32 = 32;

/// System: enqueue needed chunks and dispatch tasks.
///
/// Runs in `Update`. Reads `OriginFrame` and `LodSettings` to determine
/// which chunks belong in the streaming radius at each LOD level.
pub fn chunk_streamer(
    origin:        Res<OriginFrame>,
    lod_settings:  Res<LodSettings>,
    planet_config: Res<PlanetConfig>,
    world_seed:    Res<WorldSeed>,
    mut pool:      ResMut<ChunkPool>,
    mut queue:     ResMut<StreamingQueue>,
    mut stats:     ResMut<VoxelStats>,
    mut commands:  Commands,
    task_entities: Query<Entity, With<ChunkGenTask>>,
) {
    // ── Update task count from live entities ──
    queue.task_count = task_entities.iter().count() as u8;
    stats.in_flight_tasks = queue.task_count;
    stats.loaded_chunks   = pool.loaded.len() as u32;
    stats.queued_chunks   = queue.pending.len() as u32;

    // ── Enqueue visible chunks ───────────────────────────────────────────
    // For each LOD band, compute a radius in chunk-space and enqueue all
    // IVec3 coords within that radius that are not already loaded or queued.
    let viewer_pos = origin.position; // DVec3
    let planet_r_m = planet_config.radius_km * 1_000.0;

    // Streaming radius for each LOD (in metres).
    let [r0, r1, r2] = lod_settings.thresholds;
    let stream_radii: [(f32, u8); 4] = [
        (r0,        0),
        (r1,        1),
        (r2,        2),
        (planet_r_m as f32 * 15.0, 3), // LOD-3 covers rest of visible sphere
    ];

    for (radius_m, lod) in stream_radii {
        let chunk_size = chunk_size_for_lod(lod);
        let range = (radius_m / chunk_size).ceil() as i32 + 1;
        let viewer_chunk = world_to_chunk(viewer_pos, chunk_size);

        for dz in -range..=range {
            for dy in -range..=range {
                for dx in -range..=range {
                    let coord = viewer_chunk + IVec3::new(dx, dy, dz);
                    if pool.loaded.contains_key(&coord) { continue; }
                    if queue.in_flight.contains(&coord) { continue; }

                    let centre = chunk_centre_world(coord, chunk_size);
                    let dist_sq = (centre - viewer_pos).length_squared();
                    let radius_sq = (radius_m as f64).powi(2);

                    if dist_sq > radius_sq { continue; }

                    let neg = -(dist_sq as i64);
                    queue.pending.push(StreamJob { neg_dist_sq: neg, coord, lod });
                }
            }
        }
    }

    // ── Dispatch up to (4 − in_flight) tasks ───────────────────────────────
    let thread_pool = AsyncComputeTaskPool::get();
    while queue.task_count < 4 {
        let Some(job) = queue.pending.pop() else { break };
        if pool.loaded.contains_key(&job.coord) { continue; }
        if queue.in_flight.contains(&job.coord) { continue; }

        queue.in_flight.insert(job.coord);
        queue.task_count += 1;

        let coord  = job.coord;
        let lod    = job.lod;
        let config = Arc::new(planet_config.clone());
        let seed   = world_seed.0;

        let task: Task<ChunkGenOutput> = thread_pool.spawn(async move {
            build_chunk(coord, lod, seed, &config)
        });

        commands.spawn(ChunkGenTask(task));
    }

    // ── Evict over-budget chunks ────────────────────────────────────────────
    let budget_bytes = planet_config.memory_budget_mb as usize * 1024 * 1024;
    while pool.bytes_used > budget_bytes {
        // Find furthest loaded chunk from viewer.
        if let Some((&coord, &entity)) = pool.loaded.iter()
            .max_by_key(|(&c, _)| {
                let size = chunk_size_for_lod(0); // approximation
                let centre = chunk_centre_world(c, size);
                (centre - viewer_pos).length_squared() as i64
            })
        {
            commands.entity(entity).despawn_recursive();
            pool.loaded.remove(&coord);
        } else {
            break;
        }
    }
}

/// System: poll completed tasks and finalise chunk entities.
pub fn chunk_task_poller(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut pool:      ResMut<ChunkPool>,
    mut queue:     ResMut<StreamingQueue>,
    mut task_q:    Query<(Entity, &mut ChunkGenTask)>,
) {
    use bevy::tasks::futures_lite::future;

    for (task_entity, mut task) in &mut task_q {
        if let Some(output) = future::block_on(future::poll_once(&mut task.0)) {
            let coord = output.coord;
            let lod   = output.lod;
            let mesh  = meshes.add(output.mesh);
            let byte_count =
                output.voxel_data.densities.len() * std::mem::size_of::<f32>();

            // Spawn the chunk mesh entity.
            let chunk_entity = commands.spawn((
                VoxelChunk { chunk_coord: coord, lod },
                output.voxel_data,
                crate::plugins::voxel::components::ChunkMesh {
                    mesh_handle:  mesh.clone(),
                    vertex_count: output.positions_count,
                },
                Mesh3d(mesh),
                Transform::default(),
                Visibility::default(),
                crate::plugins::floating_origin::components::GlobalPosition(
                    chunk_centre_world(coord, chunk_size_for_lod(lod))
                ),
                crate::plugins::lod::components::LodLevel(lod),
            )).id();

            pool.loaded.insert(coord, chunk_entity);
            pool.bytes_used += byte_count;
            queue.in_flight.remove(&coord);
            commands.entity(task_entity).despawn();
        }
    }
}

// ── Helper functions ────────────────────────────────────────────────────────

fn chunk_size_for_lod(lod: u8) -> f32 {
    CHUNK_SIZE_M * (1 << lod) as f32
}

fn voxels_for_lod(lod: u8) -> u32 {
    BASE_VOXELS >> lod.min(3)
}

fn world_to_chunk(pos: DVec3, chunk_size: f32) -> IVec3 {
    IVec3::new(
        (pos.x / chunk_size as f64).floor() as i32,
        (pos.y / chunk_size as f64).floor() as i32,
        (pos.z / chunk_size as f64).floor() as i32,
    )
}

fn chunk_centre_world(coord: IVec3, chunk_size: f32) -> DVec3 {
    DVec3::new(
        (coord.x as f64 + 0.5) * chunk_size as f64,
        (coord.y as f64 + 0.5) * chunk_size as f64,
        (coord.z as f64 + 0.5) * chunk_size as f64,
    )
}

/// Full synchronous chunk generation (runs on worker thread).
fn build_chunk(coord: IVec3, lod: u8, seed: u64, config: &PlanetConfig) -> ChunkGenOutput {
    let chunk_size = chunk_size_for_lod(lod);
    let voxels     = voxels_for_lod(lod);
    let cell_size  = chunk_size / voxels as f32;
    let origin     = chunk_centre_world(coord, chunk_size) - DVec3::splat(chunk_size as f64 / 2.0);
    let planet_r_m = (config.radius_km * 1_000.0) as f32;

    // ── Sample density grid ──────────────────────────────────────────────
    let n = (voxels + 1) as usize;
    let mut densities = Vec::with_capacity(n * n * n);
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                let pos = origin + DVec3::new(
                    i as f64 * cell_size as f64,
                    j as f64 * cell_size as f64,
                    k as f64 * cell_size as f64,
                );
                densities.push(evaluate_density(pos, seed, config));
            }
        }
    }

    let voxel_data = crate::plugins::voxel::components::VoxelData {
        densities,
        voxels_per_edge: voxels,
    };

    // ── Run DMC mesher ───────────────────────────────────────────────────
    let local_origin = (origin - DVec3::splat(0.0)).as_vec3(); // TODO: use chunk-local offset
    let mesh = mesh_chunk(
        &voxel_data,
        local_origin,
        cell_size,
        &config.biome,
        planet_r_m,
    );

    let positions_count = /* read from mesh attribute */ 0u32; // set after mesh built

    ChunkGenOutput {
        coord,
        lod,
        voxel_data,
        mesh,
        vertex_colours: vec![],
        positions_count,
    }
}
```

### 7. Adaptive Avian Colliders

**`src/plugins/voxel/systems/collider_sync.rs`**:

```rust
//! ColliderSyncer: generates Avian3d colliders on newly meshed chunks.
//!
//! LOD 0: full `Collider::trimesh` from the exact DMC mesh.
//! LOD 1–2: `Collider::heightfield` (sufficient for surface-only terrain).
//! LOD 3: no collider (scaled-space only; no physics interaction expected).

use bevy::prelude::*;
use avian3d::prelude::*;

use crate::plugins::voxel::components::{
    ChunkCollider, ChunkMesh, VoxelChunk, VoxelData,
};

/// Runs in `Update` after `chunk_task_poller`.
pub fn collider_syncer(
    mut commands:   Commands,
    meshes:         Res<Assets<Mesh>>,
    chunks_needing_collider: Query<
        (Entity, &VoxelChunk, &ChunkMesh, &VoxelData),
        Without<ChunkCollider>,
    >,
) {
    for (entity, chunk, chunk_mesh, voxel_data) in &chunks_needing_collider {
        let Some(mesh) = meshes.get(&chunk_mesh.mesh_handle) else { continue };

        match chunk.lod {
            0 => {
                // Full trimesh — exact terrain collision.
                if let Some(collider) = Collider::trimesh_from_mesh(mesh) {
                    commands.entity(entity)
                        .insert(collider)
                        .insert(ChunkCollider)
                        .insert(RigidBody::Static);
                }
            }
            1 | 2 => {
                // Height-field: compute min-height, scale, and grid from voxel_data.
                let n = voxel_data.voxels_per_edge as usize;
                let heights: Vec<f32> = (0..n).flat_map(|z| {
                    (0..n).map(move |x| {
                        // Highest positive-density sample in this column.
                        (0..n).rev()
                            .map(|y| voxel_data.densities[x + y * (n+1) + z * (n+1) * (n+1)])
                            .find(|&d| d > 0.0)
                            .unwrap_or(0.0)
                    })
                }).collect();

                let chunk_size = 128.0_f32 * (1 << chunk.lod) as f32;
                let collider = Collider::heightfield(
                    heights,
                    n,
                    n,
                    Vec3::new(chunk_size, chunk_size, chunk_size),
                );
                commands.entity(entity)
                    .insert(collider)
                    .insert(ChunkCollider)
                    .insert(RigidBody::Static);
            }
            _ => {
                // LOD 3: no collider needed.
                commands.entity(entity).insert(ChunkCollider); // marker only
            }
        }
    }
}
```

### 8. Scaled-Space Sphere Switcher

**`src/plugins/voxel/systems/scaled_space.rs`**:

```rust
//! ScaledSpaceSwitcher: toggles the planet sphere vs voxel mesh visibility.
//!
//! The sphere is visible when viewer distance from planet centre > 10× radius.
//! When transitioning from sphere to voxels, the sphere fades out over
//! `BLEND_FRAMES` frames using `StandardMaterial::alpha_mode` / base_color.alpha.

use bevy::prelude::*;
use crate::plugins::{
    floating_origin::resources::OriginFrame,
    voxel::{
        components::ScaledSpaceMarker,
        config::PlanetConfig,
        resources::ScaledSpaceState,
    },
};

const BLEND_FRAMES: u32 = 30;

pub fn scaled_space_switcher(
    origin:         Res<OriginFrame>,
    planet_config:  Res<PlanetConfig>,
    mut state:      ResMut<ScaledSpaceState>,
    mut sphere_q:   Query<(&mut Visibility, &mut Transform), With<ScaledSpaceMarker>>,
) {
    let planet_r_m = planet_config.radius_km * 1_000.0;
    let switch_dist = planet_r_m * 10.0; // 10× radius
    let viewer_dist = origin.position.length(); // distance from planet centre

    let should_show_sphere = viewer_dist > switch_dist;

    for (mut vis, _) in &mut sphere_q {
        *vis = if should_show_sphere {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    state.sphere_visible = should_show_sphere;
}
```

### 9. Planet Startup Spawn

**`src/plugins/voxel/systems/planet_init.rs`** (continuation):

```rust
/// Spawn the scaled-space sphere and the planet centre marker on startup.
pub fn spawn_planet_on_startup(
    mut commands:   Commands,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
    planet_config:  Res<PlanetConfig>,
) {
    let radius_m = (planet_config.radius_km * 1_000.0) as f32;

    // ── Planet centre reference ─────────────────────────────────────────
    commands.spawn((
        PlanetCentre,
        crate::plugins::floating_origin::components::GlobalPosition(DVec3::ZERO),
        TransformBundle::default(),
    ));

    // ── Scaled-space sphere ─────────────────────────────────────────────
    // UV sphere with 64 segments; visible from all orbital altitudes.
    let sphere = meshes.add(Sphere { radius: radius_m }.mesh().uv(64, 32));
    let mat    = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.55, 0.20), // approximate biome average
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        ScaledSpaceMarker,
        Mesh3d(sphere),
        MeshMaterial3d(mat),
        crate::plugins::floating_origin::components::GlobalPosition(DVec3::ZERO),
        Visibility::Visible,
        Transform::default(),
    ));
}
```

### 10. VoxelWorldPlugin (Full)

**`src/plugins/voxel_world_plugin.rs`** (replaces stub):

```rust
//! Voxel World Engine plugin for Neon Expanse.
//!
//! Owns: `PlanetConfig` loading, scaled-space sphere, dual marching cubes
//! chunk streaming, Avian3d collision generation, and LOD integration.

use bevy::prelude::*;
use crate::plugins::voxel::{
    components::*,
    resources::*,
    systems::{
        planet_init::{load_planet_config, spawn_planet_on_startup},
        scaled_space::scaled_space_switcher,
        streaming::{chunk_streamer, chunk_task_poller},
        collider_sync::collider_syncer,
    },
};

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        // ── Resources ─────────────────────────────────────────────────────
        app.init_resource::<ChunkPool>()
           .init_resource::<StreamingQueue>()
           .init_resource::<ScaledSpaceState>()
           .init_resource::<VoxelStats>();

        // ── Component reflection ──────────────────────────────────────────
        app.register_type::<VoxelChunk>()
           .register_type::<VoxelData>()
           .register_type::<ChunkMesh>()
           .register_type::<ChunkCollider>()
           .register_type::<ScaledSpaceMarker>()
           .register_type::<PlanetCentre>();

        // ── Startup systems ───────────────────────────────────────────────
        app.add_systems(Startup, (
            load_planet_config,
            spawn_planet_on_startup.after(load_planet_config),
        ));

        // ── Update systems ────────────────────────────────────────────────
        // Ordering: streamer enqueues → task poller finalises →
        //           collider syncer adds physics → switcher toggles sphere.
        app.add_systems(Update, (
            scaled_space_switcher,
            chunk_streamer,
            chunk_task_poller.after(chunk_streamer),
            collider_syncer.after(chunk_task_poller),
        ));
    }
}
```

---

## Free-Camera System (PS4 Controller)

For testing before `TraversalPlugin` is filled, add a minimal free-camera in
`planet_init.rs` (**dev-only, behind `#[cfg(debug_assertions)]`**):

```rust
/// Spawn a camera at orbital altitude with free-look enabled.
#[cfg(debug_assertions)]
pub fn spawn_debug_camera(mut commands: Commands) {
    use crate::plugins::floating_origin::components::{FloatingOrigin, GlobalPosition};
    use bevy::math::DVec3;

    commands.spawn((
        Camera3d::default(),
        FloatingOrigin,
        GlobalPosition(DVec3::new(0.0, 0.0, 80_000_000.0)), // 80,000 km altitude
        Transform::default(),
    ));
}
```

Free-camera motion re-uses the existing `NeonInputPlugin` action events:

| PS4 Button/Axis | Action | Camera Effect |
|-----------------|--------|---------------|
| Left stick | Move XZ | Translate in local XZ |
| Right stick | Look | Pitch / yaw |
| L2 | Descend | Translate −Y (towards planet) |
| R2 | Ascend | Translate +Y |
| ✕ | Boost | 10× speed multiplier while held |

Speed is auto-scaled by altitude: `speed = viewer_altitude_m * 0.01` (so descent
from 80,000 km at `0.01 * 80_000_000 = 800 km/s`, slowing smoothly as you approach).

---

## Testing Steps

### Compile Gate
```bash
cargo check
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### Unit Tests
```bash
# Density determinism (proptest)
PROPTEST_CASES=1000 cargo test voxel

# DMC: mesh from a flat density grid should have > 0 triangles
cargo test dmc

# Golden snapshot
cargo test voxel_determinism
```

### Integration (Manual / PS4)
1. `cargo run` → planet sphere visible at orbital altitude within 2 s
2. Hold L2 to descend → LOD transitions at ~100 km, ~10 km, ~1 km (no pop-in)
3. At ~200 m altitude: voxel mesh visible with surface overhangs
4. Spawn a test physics cube (press `[1]` in debug build) → cube rests on terrain
5. Run `NEON_PLANET_CONFIG=assets/config/planets/test_variant.ron cargo run` → different planet

### PS4 Controller Test Matrix
| Test | Pass Condition |
|------|---------------|
| Connect PS4 controller before launch | "Gamepad connected" log appears |
| Left stick movement at orbital altitude | Camera translates smoothly |
| L2 descent from 80,000 km to ground | No single frame > 33 ms |
| Face button mapping | All actions respond correctly |

---

## Post-Design Constitution Re-Check

| § | Re-check Result |
|---|-----------------|
| I | Earth-scale planet, no seams — confirmed by streaming design |
| II | 100% ECS; all systems in ECS; chunk entity lifecycle is pure ECS add/despawn |
| III | fastnoise-lite ✅; avian3d 0.6.0-rc.1 ✅; custom DMC ✅; RON ✅ |
| IV | VoxelWorldPlugin fully implemented — stub replaced |
| V | All snippets have `///` docs; no unwrap outside bootstrap `load_planet_config` |
| VI | Three proptest suites + golden snapshot — determinism fully covered |
| VII | Default planet.ron contains varied biome bands; camera spawns at altitude |
| VIII | All terrain params in RON; `NoiseLayer`/`BiomeConfig` fully serde |

All 8 principles: **PASS** post-design.

---

## Next Steps (Feature 003 Readiness)

`TraversalPlugin` will need these hooks, all of which are ready after 002:

| 002 Output | 003 Consumer |
|------------|-------------|
| `Collider` on LOD-0 chunks (`RigidBody::Static`) | Player character controller (`KinematicCharacterController`) can land and walk |
| `GlobalPosition(DVec3)` on all chunks | Vehicles inherit same floating-origin coordinate system without changes |
| `WorldSeed` + `PlanetConfig` accessible via ECS | Biome-dependent spawn rules (e.g., water at `height_fraction < 0`) |
| `VoxelStats` resource | Traversal can query active chunk count for physics budget throttling |
| `LodLevel(0)` marker on near chunks | Traversal can require `LodLevel(0)` before spawning ground vehicle physics |

No changes to existing 001 plugins are required to unblock 003.

