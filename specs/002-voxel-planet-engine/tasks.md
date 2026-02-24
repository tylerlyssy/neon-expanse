# Tasks: Procedural Voxel Planet Engine

**Input**: Design documents from `specs/002-voxel-planet-engine/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/plugin-api.md ✓, quickstart.md ✓
**Branch**: `002-voxel-planet-engine`

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no unmet dependencies)
- **[Story]**: Which user story this task belongs to (US1–US6)
- All paths are relative to repository root

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Add the new dependency and establish the entire `src/plugins/voxel/` directory scaffold so that later phases can fill in file bodies without path conflicts.

- [X] T001 Add `fastnoise-lite = "1.1"` to `[dependencies]` in `Cargo.toml`
- [X] T002 [P] Create `assets/config/planets/planet.ron` with default Earth-like config (radius 6371 km, 3 noise layers FBM/Ridged/Billow, 2 erosion passes, 7-band biome table) per plan.md §2
- [X] T003 [P] Create empty module scaffold: `src/plugins/voxel/mod.rs`, `src/plugins/voxel/mesher/mod.rs`, `src/plugins/voxel/systems/mod.rs` with `//! TODO` stubs
- [X] T004 [P] Add `pub mod voxel;` to `src/plugins/mod.rs` so the new module compiles from root

**Checkpoint**: `cargo check` passes — new module tree is visible to the compiler.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core ECS types, config loader, and DMC lookup tables that every user story depends on. Nothing in Phase 3+ can compile without these.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T005 Implement `src/plugins/voxel/config.rs` — `NoiseKind (Fbm/Ridged/Billow)`, `NoiseLayer`, `ErosionConfig`, `BiomeConfig`, `PlanetConfig` with full `serde` + `ron` derives, `#[derive(Resource)]`, and a `Default` impl matching `planet.ron`
- [X] T006 [P] Implement `src/plugins/voxel/components.rs` — `VoxelChunk { chunk_coord: IVec3, lod: u8 }`, `VoxelData { densities: Vec<f32>, voxels_per_edge: u32 }`, `ChunkMesh { mesh_handle: Handle<Mesh>, vertex_count: u32 }`, `ChunkCollider` (marker), `ScaledSpaceMarker` (marker), `PlanetCentre` (marker), `ChunkGenTask(Task<ChunkGenOutput>)`, `ChunkGenOutput { coord: IVec3, lod: u8, voxel_data: VoxelData, mesh: Mesh, vertex_colours: Vec<[f32; 4]> }` — all with `#[derive(Component, Reflect)]` *(fields aligned with plan.md §3)*
- [X] T007 [P] Implement `src/plugins/voxel/resources.rs` — `ChunkPool { loaded: HashMap<IVec3, Entity>, bytes_used: usize }`, `StreamingQueue { pending: BinaryHeap<StreamJob>, in_flight: HashSet<IVec3>, task_count: u8 }`, `ScaledSpaceState { sphere_visible: bool }`, `VoxelStats { loaded_chunks: u32, queued_chunks: u32, in_flight_tasks: u8, memory_used_mb: f32 }` — all with `#[derive(Resource, Default)]`
- [X] T008 Implement `src/plugins/voxel/mesher/tables.rs` — DMC edge-to-vertex index tables and quad adjacency constants as `const` arrays
- [X] T009 Implement `src/plugins/voxel/noise_stack.rs` — `pub fn evaluate_density(pos: DVec3, seed: u64, config: &PlanetConfig) -> f32` using `fastnoise-lite`; pure function (no global state, no side-effects); base sphere SDF + additive noise layers; see plan.md §4
- [X] T010 Implement `src/plugins/voxel/mesher/dmc.rs` — custom pure-Rust Dual Marching Cubes `pub fn mesh_chunk(voxel_data: &VoxelData, chunk_origin_m: Vec3, cell_size_m: f32, biome: &BiomeConfig, planet_radius_m: f32) -> Mesh`; QEF vertex placement, per-vertex biome colour via `BiomeConfig`, watertight output; (depends on T042 — shared-border density contract must be stable before implementing DMC grid traversal); see plan.md §5
- [X] T011 Implement `src/plugins/voxel/erosion.rs` — `pub fn apply_erosion(grid: &mut Vec<f32>, config: &ErosionConfig, cols: u32, rows: u32)` hydraulic erosion; runs once at planet init; no async dependency
- [X] T042 Implement shared-border density sampling in `src/plugins/voxel/noise_stack.rs` — when generating the density grid for a chunk, extend the sample domain by 1 voxel beyond each face boundary so that adjacent chunks at the same LOD evaluate identical density values at shared edges; document the grid size contract as `(voxels_per_edge + 2)^3` (1 overlap per face); add unit test in `cfg(test)` asserting that two adjacent chunks produce equal density values at their shared face (FR-023; prerequisite for T010)
- [X] T012 Wire `src/plugins/voxel/mod.rs` — re-export all public items from `components`, `resources`, `config`, `noise_stack`, `erosion`, `mesher`, `systems` submodules

**Checkpoint**: `cargo check` passes with no errors in the voxel module. All types are addressable from root.

---

## Phase 3: User Story 1 — Planet Visible From Orbit (Priority: P1) 🎯 MVP

**Goal**: Spawn a scaled-space procedural sphere at planet centre that is visible within 2 seconds of startup from the default 80,000 km debug camera altitude. No voxel meshing required.

**Independent Test**: `cargo run` — a sphere roughly 12,742 km diameter appears against a black background within 2 s. Sphere surface colour varies with `WorldSeed`. `cargo check` and `cargo test` remain green.

- [X] T013 [US1] Implement `src/plugins/voxel/systems/planet_init.rs` — `pub fn load_planet_config(mut commands: Commands)` reads `NEON_PLANET_CONFIG` env var (or default `assets/config/planets/planet.ron`), deserialises to `PlanetConfig`, inserts as resource; falls back to `PlanetConfig::default()` on error with `warn!`; and `pub fn spawn_planet_on_startup(…)` spawns UV sphere entity (`Mesh::try_from(Sphere{…})`) with `ScaledSpaceMarker`, `GlobalPosition(DVec3::ZERO)`, and `PlanetCentre` marker
- [X] T014 [US1] Add `#[cfg(debug_assertions)] pub fn spawn_debug_camera(…)` in `src/plugins/voxel/systems/planet_init.rs` — places a free camera at `GlobalPosition(DVec3::new(0.0, 0.0, 80_000_000.0))` (80,000 km altitude); guarded by `debug_assertions` only; (depends on T013 — same file, must follow T013)
- [X] T015 [US1] Implement `src/plugins/voxel/systems/scaled_space.rs` — `pub fn scaled_space_switcher(…)` reads viewer `GlobalPosition`, planet radius from `PlanetConfig`, toggles `Visibility` on `ScaledSpaceMarker` entity when distance crosses 10× radius; smooth 1-frame hysteresis to prevent flicker
- [X] T016 [US1] Implement `src/plugins/voxel/systems/mod.rs` — `pub mod planet_init; pub mod scaled_space; pub mod streaming; pub mod mesh_builder; pub mod collider_sync;` re-exports
- [X] T017 [US1] Replace `src/plugins/voxel_world_plugin.rs` stub with full `VoxelWorldPlugin::build` registering: all four resources (`init_resource`), `Startup` systems `(load_planet_config, spawn_planet_on_startup.after(load_planet_config))`, `#[cfg(debug_assertions)]` `spawn_debug_camera`, `Update` systems stub (scaled_space_switcher only for this story), and `register_type` calls for all components; see plan.md §1

**Checkpoint**: `cargo run` shows the planet sphere from orbit within 2 s. `cargo test` stays green. All 8 existing Feature 001 tests pass.

---

## Phase 4: User Story 2 — Seamless Descent to Voxel Terrain (Priority: P1)

**Goal**: Flying from 100,000 km to 10 m altitude triggers chunk streaming with smooth LOD transitions. No geometry seams. Placeholder LOD fills gaps during fast descent.

**Independent Test**: Manual descent from orbit to ground — LOD transitions occur at ~100 km / ~10 km / ~1 km. No single-frame blank geometry. Overhangs visible below 1 km.

- [X] T018 [US2] Implement `src/plugins/voxel/systems/streaming.rs` — `pub fn chunk_streamer(…)` queries viewer `GlobalPosition`, computes nearby chunk coordinates via cube-sphere face mapping, enqueues `StreamJob`s into `StreamingQueue` (priority = distance), dispatches up to 4 concurrent `AsyncComputeTaskPool::get().spawn(…)` tasks calling `evaluate_density` + `apply_erosion` only → produces `ChunkGenOutput { coord, voxel_data, vertex_colours }` (**does NOT call `mesh_chunk`** — mesh generation is owned by T019/mesh_builder); respects `memory_budget_mb` eviction (furthest chunk first); and `pub fn chunk_task_poller(…)` polls `ChunkGenTask` components, receives completed `ChunkGenOutput`, strips old placeholder entity if present, asserts `ChunkPool` entry updated before despawn, inserts **`VoxelData` component only** on chunk entity (mesh creation triggered via `Added<VoxelData>`), updates `ChunkPool` and `VoxelStats`; see plan.md §6
- [X] T019 [US2] Implement `src/plugins/voxel/systems/mesh_builder.rs` — `pub fn mesh_builder(…)` handles `Added<VoxelData>` trigger (sole owner of mesh creation), calls `mesh_chunk(voxel_data, chunk_origin_m, cell_size_m, biome, planet_radius_m)`, inserts `Mesh` asset and `ChunkMesh` component; assigns `MaterialMeshBundle` with vertex-colour material (no UV textures); manages LOD placeholder swap without single-frame blank
- [X] T043 [US2] Implement LOD-boundary T-junction elimination — at seam edges where a LOD-0 chunk borders a LOD-1+ chunk, inject stitching skirt geometry (4-vertex quads along the lower-detail edge) in `src/plugins/voxel/mesher/dmc.rs`; add unit test asserting no open boundary edges exist at a synthetic LOD-transition zone (FR-033; prevents T-junction cracks at LOD boundaries visible during descent)
- [X] T020 [US2] Register `chunk_streamer`, `chunk_task_poller`, `mesh_builder` in `VoxelWorldPlugin::build` `Update` schedule — add after `scaled_space_switcher`; ensure ordering: `chunk_streamer` before `chunk_task_poller` before `mesh_builder`
- [X] T021 [P] [US2] Create `assets/config/planets/planet.ron` biome bands validated to cover all 7 height fractions (already created in T002 — verify completeness and add any missing LOD-0 resolution field if plan.md §2 requires it)

**Checkpoint**: `cargo run` — descending from orbit to ground shows progressive LOD mesh appearing, no blank frames, overhangs visible on surface. `cargo test` green.

---

## Phase 5: User Story 3 — Physics-Solid Terrain Surface (Priority: P2)

**Goal**: LOD-0 chunks get Avian3d trimesh colliders; LOD 1–2 get height-field colliders; LOD 3 marker-only. A physics body rests on surface without clipping.

**Independent Test**: Spawn a unit cube with mass 10 m above a loaded LOD-0 surface — it rests stably within 3 s, does not clip through. `cargo test` green.

- [X] T022 [US3] Implement `src/plugins/voxel/systems/collider_sync.rs` — `pub fn collider_syncer(…)` queries `Added<ChunkMesh>`, reads `LodLevel`: LOD 0 → `Collider::trimesh_from_mesh(&mesh)`, LOD 1–2 → `Collider::heightfield(heights, rows, cols, scale)`, LOD 3 → `ChunkCollider` marker only (no physics shape); all collider work spawned on `AsyncComputeTaskPool`, inserted after mesh is fully finalised to avoid race; see plan.md §7
- [X] T023 [US3] Register `collider_syncer` in `VoxelWorldPlugin::build` `Update` schedule — add after `mesh_builder` to guarantee mesh exists before collider is generated
- [X] T024 [P] [US3] Add integration smoke test to `tests/voxel_determinism.rs` — spawn a `RigidBody` 10 m above a generated LOD-0 chunk, step physics 200 frames at 64 Hz, assert body Y position is within 0.1 m of surface and vertical velocity < 0.01 m/s (stable rest)

**Checkpoint**: Physics body rests on surface without clipping. `cargo test` green including smoke test.

---

## Phase 6: User Story 4 — Sustained 60+ FPS Performance (Priority: P2)

**Goal**: Frame time stays ≤ 16.7 ms at 200 m altitude with 10+ chunks loading. Worst-case spike ≤ 33 ms. Memory within `memory_budget_mb`.

**Independent Test**: `FrameTimeDiagnosticsPlugin` enabled — 60 s fly-through bench at 500 m/s, 200 m altitude, 20+ chunk crossings. Avg ≥ 60 FPS, worst-case ≤ 33 ms.

- [X] T025 [US4] Audit `chunk_streamer` task dispatch — verify `StreamingQueue::task_count` hard cap at 4 is enforced on every frame; add `debug_assert!(queue.task_count <= 4)` guard; verify `memory_budget_mb` eviction path is exercised when pool is full
- [X] T026 [P] [US4] Add `VoxelWorldPlugin::stats()` diagnostic — implement `pub fn stats(stats: Res<VoxelStats>)` system and expose via `VoxelWorldPlugin::stats()` public fn returning `VoxelStats` snapshot; register as `Update` system; log at `debug!` level each frame
- [X] T027 [P] [US4] Add `FrameTimeDiagnosticsPlugin` to `VoxelWorldPlugin::build` under `#[cfg(debug_assertions)]` — enables FPS output without affecting release build
- [X] T028 [US4] Profile and bound single-chunk pipeline — add a `std::time::Instant` wall-clock measurement inside the async task closure for `evaluate_density + apply_erosion + mesh_chunk`; log `warn!` if elapsed > 100 ms; do NOT block main thread (measurement only)

**Checkpoint**: Manual fly-through bench shows ≥ 60 FPS average on reference machine. `cargo test` green.

---

## Phase 7: User Story 5 — Data-Driven Planet Variants (Priority: P3)

**Goal**: A new RON file at `assets/config/planets/test_variant.ron` loaded via `NEON_PLANET_CONFIG` produces a visually distinct planet with no source changes, no recompile.

**Independent Test**: `NEON_PLANET_CONFIG=assets/config/planets/test_variant.ron cargo run` — planet radius and surface colour differ from default `planet.ron` run.

- [X] T029 [US5] Create `assets/config/planets/test_variant.ron` — different `radius_km` (e.g. 3389 km, Mars-scale), different noise frequencies and amplitudes, 5-band desert biome palette; structurally distinct from `planet.ron`
- [X] T030 [P] [US5] Harden `load_planet_config` in `src/plugins/voxel/systems/planet_init.rs` — verify unknown RON fields are ignored (`#[serde(deny_unknown_fields)]` removed or `default` applied), log `warn!` with config file path on any deserialise error, confirm `Default::default()` fallback path is exercised (add `debug_assert` in dev builds)
- [X] T031 [P] [US5] Add unit test in `src/plugins/voxel/tests/` (inline cfg-test module in `config.rs`) — test `PlanetConfig::deserialize` roundtrip for both `planet.ron` and `test_variant.ron` contents; test that missing file falls back to `Default` without panicking; test that `NEON_PLANET_CONFIG` env var path is read correctly

**Checkpoint**: Both RON variants loadable and visually distinct. `cargo test` green including config roundtrip tests.

---

## Phase 8: User Story 6 — Deterministic Terrain Reproducibility (Priority: P3)

**Goal**: Two runs with `WorldSeed(NEON2026)` produce byte-identical density arrays. Golden snapshot in CI. Proptest 1000-case fuzz passes.

**Independent Test**: `cargo test` — determinism regression test and proptest suite both pass. No snapshot update step required.

- [X] T032 [US6] Implement integration test in `tests/voxel_determinism.rs` — generate density for 100 fixed `IVec3` chunk coords using `WorldSeed(0x4E45_4F4E_3230_3236)` and default `PlanetConfig`; serialise to `Vec<u8>` (little-endian f32); compare against committed golden snapshot bytes loaded from `tests/snapshots/voxel_density_golden.bin`; test fails if any byte differs; generate snapshot with `GENERATE_GOLDEN=1 cargo test` env guard
- [X] T033 [P] [US6] Create `tests/snapshots/voxel_density_golden.bin` — run the snapshot generator (`GENERATE_GOLDEN=1 cargo test voxel_determinism`) once on the reference machine and commit the output binary file
- [X] T034 [P] [US6] Add proptest suite in `src/plugins/voxel/noise_stack.rs` (cfg-test module) — `proptest! { fn density_is_deterministic(coord in any::<(i32, i32, i32)>()) { let a = evaluate_density(…); let b = evaluate_density(…); assert_eq!(a, b); } }` with `PROPTEST_CASES=1000`
- [X] T035 [P] [US6] Add proptest suite in `src/plugins/voxel/mesher/` (cfg-test module in `dmc.rs`) — generate random `VoxelData`, mesh twice with identical config, assert `vertex_count` and vertex position bytes are equal; fuzz for watertight property (no degenerate triangles)

**Checkpoint**: `PROPTEST_CASES=1000 cargo test` green. Golden snapshot committed. Determinism proven across 1000 fuzz cases.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates required before merge to main.

- [X] T036 [P] Run `cargo fmt --all` and fix all formatting violations across new files in `src/plugins/voxel/`
- [X] T037 [P] Run `cargo clippy --all-targets -- -D warnings` and fix all lint warnings in new code
- [X] T038 Add `///` doc comments to all public items in `src/plugins/voxel/` — `PlanetConfig`, `VoxelChunk`, `ChunkPool`, `StreamingQueue`, `evaluate_density`, `mesh_chunk`, `apply_erosion`, `VoxelWorldPlugin`; no bare `unwrap()` outside bootstrap
- [X] T039 Verify all 8 existing Feature 001 tests still pass — run `cargo test` and confirm no regressions (`SC-010`)
- [X] T040 [P] Final `cargo check` + `cargo test` full suite — must be green with zero errors and zero warnings; confirm `SC-011` (fmt + clippy) and `SC-012` (single-chunk ≤ 100 ms diagnostic logged)
- [X] T041 [P] Run quickstart.md merge checklist — follow steps in `specs/002-voxel-planet-engine/quickstart.md` §7 and confirm all items checked
- [X] T044 [P] Implement SC-002 automated visual seam verification — create `tests/lod_seam_visual.rs` using Bevy's `ScreenshotManager` to capture one frame at each LOD transition altitude during a scripted descent (100,000 km → ground); compare saved PNGs against committed golden frames in `tests/snapshots/lod_seam_*.png` (any pixel diff > 1% of frame area fails); or, if Bevy 0.18 `ScreenshotManager` API proves insufficient, produce a documented manual inspection checklist in `specs/002-voxel-planet-engine/quickstart.md` §5 with annotated expected screenshots committed alongside (SC-002)

**Checkpoint**: All success criteria SC-001 through SC-012 verified. Ready to merge `002-voxel-planet-engine` → main.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on T001 (fastnoise-lite in Cargo.toml) and T003–T004 (module scaffold)
- **Phase 3 (US1)**: Depends on Phase 2 complete (config, components, resources all needed)
- **Phase 4 (US2)**: Depends on Phase 3 complete (spawned planet entity + streaming hooks need scaled-space already running)
- **Phase 5 (US3)**: Depends on Phase 4 complete (colliders need mesh to exist)
- **Phase 6 (US4)**: Depends on Phase 5 complete (perf audit requires full pipeline)
- **Phase 7 (US5)**: Depends on Phase 2 complete only — config loading is standalone; can be worked in parallel with US4
- **Phase 8 (US6)**: Depends on Phase 2 complete only — `evaluate_density` is a pure function; can be worked in parallel with US4/US5
- **Phase 9 (Polish)**: Depends on all desired stories complete

### User Story Dependencies

| Story | Depends On | Can Start After |
|-------|-----------|-----------------|
| US1 — Planet Visible From Orbit | Phase 2 | Foundational done |
| US2 — Seamless Descent | US1 (sphereentity must exist) | US1 checkpoint |
| US3 — Physics-Solid Terrain | US2 (mesh must exist) | US2 checkpoint |
| US4 — 60+ FPS Performance | US3 (full pipeline) | US3 checkpoint |
| US5 — Data-Driven Variants | Phase 2 only | Foundational done |
| US6 — Deterministic Reproducibility | Phase 2 only | Foundational done |

### Within Each User Story

- Config/types before systems that use them
- Systems before plugin registration that adds them
- Plugin registration before manual test/validation
- Mesh complete before collider generation

---

## Parallel Execution Examples

### Phase 2 Parallel Batch

```text
T005 config.rs     ← independent
T006 components.rs ← independent
T007 resources.rs  ← independent
T008 tables.rs     ← independent
```

*All four files have no intra-phase dependencies — implement simultaneously.*

### Phase 3 (US1) Parallel Batch

```text
T013 planet_init.rs systems ← core logic
T014 spawn_debug_camera     ← separate fn, same file, after T013
```

### Phase 4 (US2) Parallel Batch

```text
T018 streaming.rs  ← independent of mesh_builder
T019 mesh_builder  ← independent of streaming internals
```

*T020 (registration) must follow both.*

### Phase 7+8 (US5+US6) Can Run in Parallel With Phase 6

US5 and US6 only depend on Phase 2 types — they can be worked concurrently with the US4 performance pass if two developers are available.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete **Phase 1**: Setup — add dep, scaffold module tree
2. Complete **Phase 2**: Foundational — all types and pure logic
3. Complete **Phase 3**: US1 — visible planet from orbit
4. **STOP AND VALIDATE**: `cargo run` shows sphere, `cargo test` green
5. Demo / review before proceeding

### Incremental Delivery

| Milestone | Phases | What You Get |
|-----------|--------|-------------|
| **MVP** | 1 + 2 + 3 | Planet sphere visible from orbit |
| **Sprint 1** | + Phase 4 | Full descent to voxel terrain |
| **Sprint 2** | + Phase 5 | Physics-solid surface |
| **Sprint 3** | + Phase 6 | 60 FPS confirmed |
| **Complete** | + 7 + 8 + 9 | Moddable, deterministic, shippable |

### Single-Developer Sequence

```text
Phase 1 → Phase 2 → Phase 3 → [validate] → Phase 4 → Phase 5 →
Phase 6 → Phase 7 → Phase 8 → Phase 9 → merge
```

---

## Task Summary

| Phase | Tasks | Story | Parallelisable |
|-------|-------|-------|---------------|
| Phase 1: Setup | T001–T004 | — | T002, T003, T004 |
| Phase 2: Foundational | T005–T012, T042 | — | T006, T007, T008, T009, T011 |
| Phase 3: US1 | T013–T017 | US1 | *(T014 depends on T013 — not parallel)* |
| Phase 4: US2 | T018–T021, T043 | US2 | T021 |
| Phase 5: US3 | T022–T024 | US3 | T024 |
| Phase 6: US4 | T025–T028 | US4 | T026, T027 |
| Phase 7: US5 | T029–T031 | US5 | T030, T031 |
| Phase 8: US6 | T032–T035 | US6 | T033, T034, T035 |
| Phase 9: Polish | T036–T041, T044 | — | T036, T037, T040, T041, T044 |
| **Total** | **44 tasks** | | **22 parallelisable** |

---

## Format Validation

All 41 tasks follow the required checklist format:

- ✅ Every task starts with `- [ ]`
- ✅ Every task has a sequential ID (T001–T041)
- ✅ `[P]` present only on tasks with no unmet same-phase dependencies
- ✅ `[US1]–[US6]` labels present on all user-story phase tasks
- ✅ Setup and Foundational phase tasks have no story label
- ✅ Polish phase tasks have no story label
- ✅ Every task includes an exact file path or explicit scope
