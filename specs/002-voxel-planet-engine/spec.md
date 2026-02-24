# Feature Specification: Procedural Voxel Planet Engine

**Feature Branch**: `002-voxel-planet-engine`
**Created**: 2026-02-20
**Status**: Draft
**Input**: User description: "Create the full Procedural Voxel Planet Engine so the
game has one seamless Earth-scale planet exactly like Light No Fire — dual marching
cubes voxel meshing — integrates with FloatingOrigin (DVec3), LodPlugin, Physics
(Avian), Input — 100% data-driven via RON — Hybrid system: scaled-space >10×
radius, full voxel mesh below — Layered procedural generation: base sphere →
multiple noise layers → erosion simulation."

---

## Clarifications

### Session 2026-02-20

- Q: Which Rust noise crate should the density generator use? → A: `fastnoise-lite`
- Q: What cave geometry is in scope for 002? → A: Surface overhangs and arches only; no traversable hollow cave interiors.
- Q: Which collider type for LOD 1–2 chunks? → A: Height-field colliders.
- Q: What mechanism selects a non-default planet config at runtime? → A: `NEON_PLANET_CONFIG` environment variable.
- Q: What is the maximum number of concurrent chunk generation tasks? → A: 4 (fixed cap).

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Planet Visible From Orbit (Priority: P1)

As a player launching the game for the first time, I can see a recognisable
Earth-scale planet rendered in space from any orbital altitude, so the world
immediately communicates scale and invites exploration.

**Why this priority**: Without a visible planet the game has nothing. This is the
absolute minimum viable deliverable for 002 and unlocks all subsequent stories.

**Independent Test**: `cargo run` places the camera at 80,000 km altitude. A sphere
of roughly 12,742 km diameter appears against a black background with subtle
procedural surface colour variation driven by `WorldSeed`. No voxel meshing or
physics is required for this story to pass.

**Acceptance Scenarios**:

1. **Given** the app starts with default config, **When** the main scene initialises, **Then** a scaled-space sphere representing the planet is visible within 2 seconds and fills the expected screen area for the starting orbital altitude.
2. **Given** the camera is placed at any altitude above 10× planet radius (~63,710 km), **When** the frame renders, **Then** the planet renders as a smooth procedural sphere (LOD 3) with no visual seams.
3. **Given** a custom `planet.ron` file is present, **When** the planet config is loaded, **Then** the sphere reflects the custom radius and surface colour scheme defined in that file.

---

### User Story 2 — Seamless Descent to Voxel Terrain (Priority: P1)

As a player piloting a craft, I can fly continuously from orbit down to ground
level and experience an uninterrupted transition from the scaled-space sphere to
full-detail voxel terrain with overhangs, caves, and curved-horizon geometry, with
no seams, no pop-in, and no noticeable stall at any altitude band.

**Why this priority**: This "wow moment" is the defining feature of the 002 sprint.
It demonstrates the FloatingOrigin + LOD + voxel-meshing pipeline end-to-end and
is the primary quality gate for the sprint.

**Independent Test**: A scripted camera descends at constant speed from 100,000 km
to 10 m altitude over the planet surface. Each LOD boundary crossing is recorded.
The test passes if no single frame exceeds 33 ms (30 FPS floor), no geometry seam
appears at any LOD boundary, and voxel overhangs and surface curvature are visible
at altitudes below 1 km.

**Acceptance Scenarios**:

1. **Given** the camera is at 100,000 km altitude, **When** it descends continuously, **Then** LOD transitions occur at ~63,710 km, ~100 km, ~10 km, and ~1 km without visible geometry pop-in.
2. **Given** the camera crosses the full-voxel threshold (~1 km), **When** the frame renders, **Then** terrain features surface overhangs and arches consistent with a sphere of 6,371 km radius (no hollow cave interiors required).
3. **Given** the camera descends rapidly (1 km/s), **When** chunks have not yet fully generated, **Then** a lower-LOD placeholder mesh fills the gap; the viewer never sees empty sky where terrain should exist.

---

### User Story 3 — Physics-Solid Terrain Surface (Priority: P2)

As a player standing or driving on the planet surface, I am prevented from passing
through the terrain geometry by solid collision volumes so that movement, physics
objects, and future vehicles interact naturally with the landscape.

**Why this priority**: Essential before any traversal or gameplay mechanics in 003+,
but does not block the visual "wow moment" of Story 2.

**Independent Test**: A physics-enabled test entity (unit cube with mass) is spawned
10 m above the planet surface inside a loaded chunk. The test passes if the entity
rests stably on the surface within 3 seconds without clipping or tunnelling.

**Acceptance Scenarios**:

1. **Given** a chunk at LOD 0 (< 1 km) is loaded, **When** physics simulation runs, **Then** the chunk has an active collision volume that prevents a physics body from passing through.
2. **Given** the camera moves from one chunk to an adjacent one, **When** the new chunk's collision volume activates, **Then** there is no gap in collision coverage between the two chunks.
3. **Given** the camera is above LOD 2 altitude (> 10 km), **When** chunks at that LOD are loaded, **Then** collision volumes use a simplified representation sufficient to prevent obvious clipping.

---

### User Story 4 — Sustained 60+ FPS Performance (Priority: P2)

As a player flying at low altitude over varied terrain, the game maintains 60 frames
per second or above throughout, so exploration feels fluid regardless of the
complexity of the landscape being traversed.

**Why this priority**: Frame-rate loss is a hard blocker for any gameplay. Must be
validated before the feature merges to main.

**Independent Test**: A bench scene flies the camera at 500 m/s at 200 m altitude
for 60 seconds, crossing at least 20 chunk boundaries. `FrameTimeDiagnosticsPlugin`
output must report ≥ 60 FPS average and ≤ 33 ms worst-case frame time. Test runs
on the CI reference machine (Apple M-series or equivalent mid-range desktop GPU).

**Acceptance Scenarios**:

1. **Given** the camera is at 200 m altitude moving at walking speed, **When** 10 LOD-0 chunks are simultaneously loaded, **Then** frame time stays below 16.7 ms.
2. **Given** the camera makes a sudden 90-degree turn at 500 m/s, **When** new chunks stream in, **Then** the frame-rate dip lasts no longer than 500 ms before returning to 60 FPS.
3. **Given** 64 chunks are loaded across all LOD levels simultaneously, **When** the scene is rendered, **Then** CPU and GPU memory usage does not exceed expected budgets documented in `PlanetConfig`.

---

### User Story 5 — Data-Driven Planet Variants (Priority: P3)

As a content creator or modder, I can define an entirely new planet variant by
authoring a single RON configuration file under `assets/config/planets/`, restart
the engine pointing to that file, and see a structurally different planet with no
code changes required.

**Why this priority**: Constitution §VII requires 100% data-driven moddability.
This is a quality-of-life and future-proofing requirement; the base game needs only
one planet config to ship.

**Independent Test**: `assets/config/planets/test_variant.ron` is created with a
different radius, noise profile, and biome palette. `NEON_PLANET_CONFIG=assets/config/planets/test_variant.ron cargo run` loads that file. The result visually differs from the default
planet in radius and surface colouring.

**Acceptance Scenarios**:

1. **Given** a valid `planet.ron` file exists, **When** the engine starts, **Then** `PlanetConfig` is populated entirely from that file with no hard-coded overrides.
2. **Given** a `planet.ron` file is missing or malformed, **When** the engine starts, **Then** a descriptive error is logged and the engine falls back to a default Earth-like config rather than panicking.
3. **Given** two different RON files with different seeds and noise layers, **When** each is loaded in separate runs, **Then** the resulting planets are visually and structurally distinct.

---

### User Story 6 — Deterministic Terrain Reproducibility (Priority: P3)

As a developer or QA engineer, running the engine twice with the same `WorldSeed`
produces byte-identical voxel density fields and mesh vertex positions, so terrain
bugs are reproducible and regression-tested in CI.

**Why this priority**: Determinism is required by Constitution §VI and is the
foundation for reliable test automation in all future features.

**Independent Test**: `cargo test` includes a regression test that generates voxel
density for a fixed set of chunk coordinates using `WORLD_SEED`, serialises the
density arrays, and compares against a stored golden snapshot. The test fails if any
value differs.

**Acceptance Scenarios**:

1. **Given** `WorldSeed(NEON2026)` and a fixed chunk coordinate, **When** the density field is generated twice, **Then** both runs produce identical `Vec<f32>` output, verified by a `cargo test` assertion.
2. **Given** `PROPTEST_CASES=1000` fuzz runs with random chunk coordinates, **When** each coordinate is generated twice with the same seed, **Then** all pairs are identical.
3. **Given** a committed golden snapshot file, **When** `cargo test` runs in CI, **Then** the snapshot test passes without any manual update step.

---

### Edge Cases

- What happens when the viewer is at or near the planet centre (r ≈ 0)?
  → Density evaluation must not divide by zero; chunk generation clips to a minimum radius and logs a warning.
- What happens when chunk generation falls behind streaming demand (rapid descent)?
  → A lower-LOD placeholder mesh is used until higher-LOD data is ready; the viewer never sees empty geometry.
- What happens at the poles and the anti-meridian where chunk grids meet?
  → Chunk coordinates use cube-sphere mapping with no degenerate poles; seam stitching tests cover all 6 cube faces.
- What happens when `planet.ron` is absent or contains unknown fields?
  → Engine logs a `WARN`, falls back to default Earth-like config, and continues; it does not panic.
- What happens when two `PlanetConfig` resources are registered simultaneously?
  → The system is defined to support exactly one active planet; a second insert replaces and logs a `WARN`.
- What happens when a chunk is requested while a prior generation job for the same coord is in flight?
  → The duplicate request is deduplicated via `StreamingQueue`; no duplicate mesh or collider is created.
- What happens when the camera teleports instantly from orbit to ground level?
  → The streaming system prioritises chunks nearest the new position; placeholder geometry fills in until real chunks load (≤ 2 s SLA).

---

## Requirements *(mandatory)*

### Functional Requirements

#### Planet Configuration

- **FR-001**: The system MUST load planet parameters from the RON asset file at `assets/config/planets/planet.ron` by default. If the `NEON_PLANET_CONFIG` environment variable is set to a valid file path at startup, that file is used instead. The loaded parameters are stored in a `PlanetConfig` resource.
- **FR-002**: `PlanetConfig` MUST include: planet radius (km), world seed override (optional, defaults to `WorldSeed`), an ordered list of `NoiseLayer` descriptors, erosion pass count, and a biome height-colour table.
- **FR-003**: Each `NoiseLayer` MUST specify: noise type (FBM, Ridged, Billow), frequency, amplitude, octaves, persistence, and lacunarity — all serialisable as RON fields.
- **FR-004**: The system MUST fall back to hard-coded Earth-like defaults when `planet.ron` is absent or fails to deserialise, logging a human-readable warning without panicking.
- **FR-005**: `PlanetConfig` MUST be a Bevy `Resource` accessible to all VoxelWorldPlugin systems via normal ECS queries.
- **FR-006**: Adding a new planet variant MUST require only a new RON file; no Rust source changes, no recompile.
- **FR-007**: The planet radius in `PlanetConfig` MUST drive all derivative constants (scaled-space switch distance, LOD thresholds multiplier, chunk size in km) so no magic numbers appear outside that resource.
- **FR-008**: `PlanetConfig` MUST expose a `memory_budget_mb: u32` field that caps the total voxel data held in `ChunkPool` at runtime.

#### Scaled-Space Rendering

- **FR-009**: The system MUST render the planet as a smooth procedural sphere when the viewer's `GlobalPosition` is at a distance greater than 10× the planet radius from the planet centre.
- **FR-010**: The scaled-space sphere MUST derive its surface colour from the biome height-colour table in `PlanetConfig`, giving a visually consistent appearance from orbit.
- **FR-011**: The scaled-space sphere MUST be a single `Mesh` entity registered in the ECS with a `GlobalPosition` at the planet centre; it MUST use `FloatingOriginPlugin` for positional accuracy.
- **FR-012**: The system MUST transition between the scaled-space sphere and voxel meshing as the viewer crosses the 10× radius threshold with no multi-frame pop-in. For this sprint, a 1-frame `Visibility` hysteresis toggle is the accepted implementation; continuous alpha-blend fading between representations is deferred to a future sprint.
- **FR-013**: The scaled-space sphere MUST be visible at all altitudes above the transition threshold, including from the maximum expected game camera distance (~500,000 km).

#### Chunk Streaming

- **FR-014**: The planet surface MUST be subdivided into a grid of `VoxelChunk` entities; chunk coordinate is an `IVec3` in chunk-space (not world-space).
- **FR-015**: The system MUST asynchronously generate, load, and unload chunks based on viewer `GlobalPosition`, keeping chunks within the streaming radius resident and evicting farther chunks when the memory budget is exceeded.
- **FR-016**: Chunk generation MUST be driven by a `StreamingQueue` resource that prioritises chunks by their distance to the viewer, nearest first.
- **FR-017**: The system MUST never block the main game thread during chunk mesh or density generation; all heavy computation MUST run on a worker thread via Bevy's async task system. No more than 4 chunk generation tasks may be in-flight simultaneously; additional requests remain queued in `StreamingQueue` until a slot is free.
- **FR-018**: When a higher-LOD chunk completes generation, the lower-LOD placeholder covering the same region MUST be replaced without a visible single-frame gap.
- **FR-019**: The system MUST respect `PlanetConfig::memory_budget_mb`; when the budget is reached, the furthest loaded chunks MUST be unloaded before new ones are generated.

#### Dual Marching Cubes Mesher

- **FR-020**: The system MUST generate terrain meshes using Dual Marching Cubes (DMC) to produce smooth, manifold geometry capable of representing surface overhangs and arches. Traversable hollow cave interiors are out of scope for this feature; the density field represents surface displacement over a base sphere SDF only.
- **FR-021**: The density field for each chunk MUST be a 3-D scalar grid (`VoxelData`) evaluated by layering the noise functions defined in `PlanetConfig::noise_layers` over a base sphere SDF. Noise evaluation MUST use the `fastnoise-lite` crate; no other noise library may be used for density generation.
- **FR-022**: The mesher MUST produce watertight (no holes) meshes at all LOD levels so physics colliders can be generated without manual repair.
- **FR-023**: Adjacent chunks MUST share density samples at their borders to guarantee seamless meshing with no cracks between chunk boundaries at the same LOD level.
- **FR-024**: The mesh resolution (voxels per chunk edge) MUST decrease by a factor of 2 at each successive LOD level, halving vertex count per level.
- **FR-025**: The mesher MUST assign per-vertex colour from the biome height-colour table based on the vertex's elevation above the base sphere, requiring no texture UV coordinates.

#### Physics Collider Generation

- **FR-026**: Every loaded chunk at LOD 0 (< 1 km from viewer) MUST have an Avian3d trimesh collider generated from its final voxel mesh.
- **FR-027**: Chunks at LOD 1–2 (1 km–100 km) MUST use height-field colliders to prevent obvious clipping of fast-moving physics objects; convex decomposition is not required at these LOD levels.
- **FR-028**: Colliders MUST be attached as Avian3d `Collider` components on the chunk entity and MUST update whenever the chunk mesh is regenerated.
- **FR-029**: Collider generation MUST occur on a worker thread; the collider MUST be inserted into the ECS only after the associated mesh is fully finalised.

#### LOD Integration

- **FR-030**: The voxel planet system MUST reuse the existing `LodPlugin` and its `LodLevel` component without modifying them; chunk LOD is determined by the `LodLevel` assigned to each chunk entity.
- **FR-031**: The system MUST use `LodSettings::thresholds` (`[1_000.0, 10_000.0, 100_000.0]` metres) as the authoritative LOD boundaries for chunk selection, unless overridden by `PlanetConfig`.
- **FR-032**: When the viewer crosses a `LodSettings` threshold, the system MUST swap the affected chunk entities transactionally: the higher-detail chunk entity MUST be spawned before the lower-detail entity is despawned for that region.
- **FR-033**: LOD transitions at the same position on the surface MUST produce meshes that align at boundaries, eliminating T-junction cracks between adjacent LOD levels.

#### Determinism and Performance

- **FR-034**: Voxel density generation MUST be a pure function of `(chunk_coord: IVec3, PlanetConfig, WorldSeed)` with no dependence on wall-clock time, frame number, or thread ordering.
- **FR-035**: Density generation MUST be seeded from the `WorldSeed` resource provided by `CorePlugin`; `PlanetConfig::seed_override` overrides only when explicitly set.
- **FR-036**: The full pipeline (density evaluation + DMC meshing + collider bake) for a single LOD-0 chunk MUST complete within 100 ms on a single worker thread to support 60 FPS streaming at walking speed.
- **FR-037**: The system MUST expose a `VoxelWorldPlugin::stats()` diagnostic reporting: loaded chunk count, queued chunk count, in-flight task count (max 4), and memory used (MB).
- **FR-038**: All positions used by VoxelWorldPlugin systems MUST be stored as `GlobalPosition(DVec3)` components; no system within this plugin MUST read or write `Transform` directly for world-space logic.

### Key Entities

- **`VoxelChunk`**: Component marking an entity as a chunk tile. Carries `chunk_coord: IVec3` (chunk-space grid address) and `lod: u8` (0 = highest detail).
- **`VoxelData`**: Component holding the raw signed-distance density field as `densities: Vec<f32>` at the voxel grid resolution appropriate for the chunk's LOD level.
- **`ChunkMesh`**: Component holding the mesh handle produced by the DMC mesher for this chunk, along with `vertex_count: u32` for budget tracking.
- **`ChunkCollider`**: Component marking that an Avian3d `Collider` has been generated and attached to this chunk entity.
- **`PlanetConfig`**: Resource. Deserialised from RON. Fields: `radius_km: f64`, `seed_override: Option<u64>`, `noise_layers: Vec<NoiseLayer>`, `erosion_passes: u32`, `biome_config: BiomeConfig`, `memory_budget_mb: u32`. The maximum concurrent generation task count is fixed at 4 and not exposed as a config field.
- **`NoiseLayer`**: Plain data struct (serde). Fields: `kind: NoiseKind` (FBM | Ridged | Billow), `frequency: f32`, `amplitude: f32`, `octaves: u32`, `persistence: f32`, `lacunarity: f32`.
- **`ErosionConfig`**: Nested in `PlanetConfig`. Fields: `passes: u32`, `erosion_rate: f32`, `sediment_capacity: f32`. Controls hydraulic erosion simulation applied after noise.
- **`BiomeConfig`**: Nested in `PlanetConfig`. Ordered list of `(height_fraction: f32, colour: [f32; 3])` entries mapping normalised elevation to vertex colour.
- **`ChunkPool`**: Resource. Registry of all currently loaded chunk entities keyed by `IVec3` chunk coordinate. Used for deduplication and eviction.
- **`StreamingQueue`**: Resource. Priority queue (min-heap by distance) of pending `(IVec3, u8)` (coord, lod) jobs. Drives worker thread dispatch.
- **`ScaledSpaceMarker`**: Component on the scaled-space sphere entity, used to query and toggle it during LOD-transition blend.
- **`PlanetCentre`**: Component on an ECS entity at `GlobalPosition(DVec3::ZERO)` acting as the authoritative reference point for all chunk distance queries.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The planet sphere is visible and correctly sized within 2 seconds of application launch on the reference machine.
- **SC-002**: A continuous descent from 100,000 km to ground level produces zero visible geometry seams or pop-in events at any LOD boundary. Verification for this sprint is manual visual inspection along a scripted camera path; a task for automated screenshot comparison tooling is tracked as T044.
- **SC-003**: Frame time remains below 16.7 ms (≥ 60 FPS) throughout a 60-second automated fly-through at 500 m/s at 200 m altitude with 10+ chunks loading simultaneously.
- **SC-004**: Worst-case frame spike during a rapid descent from orbit to ground (1 km/s) never exceeds 33 ms (30 FPS floor) for more than 500 ms cumulatively.
- **SC-005**: A unit-mass physics body spawned 10 m above a loaded LOD-0 chunk surface rests stably on the surface within 3 seconds without clipping or tunnelling.
- **SC-006**: `cargo test` includes a determinism regression test that confirms identical voxel density arrays for a fixed set of 100 chunk coordinates across two sequential runs with `WORLD_SEED`.
- **SC-007**: `PROPTEST_CASES=1000 cargo test` passes with all density-determinism fuzz cases green.
- **SC-008**: A new planet variant is fully functional after authoring only a RON file with no Rust source edits and no recompile step.
- **SC-009**: Total GPU memory consumed by all loaded voxel meshes never exceeds `PlanetConfig::memory_budget_mb` during the 60-second flight benchmark.
- **SC-010**: All 8 existing tests from feature 001 continue to pass unmodified after 002 is merged (`cargo test`).
- **SC-011**: `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass with zero violations after 002 implementation.
- **SC-012**: LOD-0 chunk density + mesh + collider pipeline for a single chunk completes in under 100 ms on the reference machine, measured by the `VoxelWorldPlugin::stats()` diagnostic.

---

## Assumptions

- Planet radius is Earth-scale (6,371 km) unless overridden in `planet.ron`; performance targets are calibrated for this scale.
- The player/camera entity is the sole LOD reference point; multiplayer streaming from multiple origins is out of scope for this feature.
- Hydraulic erosion simulation runs at planet-init time, not per-frame; erosion parameters affect initial terrain shape only.
- Chunk edge length at LOD 0 is 128 m; this gives a 96-voxel grid at approximately 4 voxels per metre resolution. Exact grid size may be tuned in `PlanetConfig`.
- Bevy 0.18 `AsyncComputeTaskPool` is used for chunk generation; no external thread-pool library is introduced.
- Noise evaluation uses the `fastnoise-lite` crate (pure-Rust, cross-platform deterministic); no other noise crate is permitted for density generation. Golden snapshot tests are pinned to a specific `fastnoise-lite` version.
- Planet has a single continuous surface (no ring planet, no binary planet) for this sprint.

## Out of Scope

- Player locomotion and vehicle physics (Feature 003 — TraversalPlugin)
- Advanced biome texturing and material blending (future sprint)
- Vegetation, structures, or dynamic world modifications (future sprint)
- Multiplayer or server-side chunk streaming
- Atmospheric scattering or weather simulation
- More than one simultaneously active planet
- Traversable hollow cave networks and underground interiors (future sprint)
