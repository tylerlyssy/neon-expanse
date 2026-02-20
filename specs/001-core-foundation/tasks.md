# Tasks: Core Foundation

**Input**: Design documents from `specs/001-core-foundation/`
**Prerequisites**: [plan.md](plan.md) ✅ · [spec.md](spec.md) ✅ · [research.md](research.md) ✅ · [data-model.md](data-model.md) ✅ · [contracts/plugin-api.md](contracts/plugin-api.md) ✅

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no incomplete task dependencies)
- **[Story]**: Maps to user stories in spec.md (US1–US4)
- All file paths are relative to workspace root `/Users/tylerlyssy/neon-expanse/`

---

## Phase 1: Setup

**Purpose**: Cargo manifest, build configuration, and module scaffold — required before any source file compiles.

- [X] T001 Create `Cargo.toml` with all dependencies: `bevy = "0.18"`, `avian3d = "0.5"`, `serde = "1"`, `ron = "0.8"`, `glam = "0.29"`, dev `proptest = "1"`; edition 2024; rust-version 1.85; dev/release profiles; `tracy = ["bevy/trace_tracy"]` feature flag
- [X] T002 [P] Create `.cargo/config.toml` with `target-cpu = "native"` and fast-linker configuration for macOS
- [X] T003 [P] Create `src/plugins/mod.rs` declaring all eight plugin modules (`core_plugin`, `window_plugin`, `physics_plugin`, `voxel_world_plugin`, `traversal_plugin`, `floating_origin`, `input`, `lod`) with `pub mod` entries

**Checkpoint**: `cargo check` must succeed with zero errors before proceeding.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Components, resources, and plugin wrappers that have no cross-plugin dependencies and are consumed by every user story phase. Must be complete before Phase 3.

**⚠️ CRITICAL**: `LodLevel` (T007) and `LodSettings` (T008) are imported by `floating_origin/systems.rs` — they must exist before Phase 4 begins.

- [X] T004 Implement `PhysicsPlugin` struct that calls `app.add_plugins(PhysicsPlugins::default())` and logs `"Neon Expanse physics initialized (avian3d, 64 Hz)"` in `src/plugins/physics_plugin.rs`
- [X] T005 [P] Implement `VoxelWorldPlugin` as an empty `impl Plugin` stub with doc comment describing future ownership (chunk streaming, mesher, procedural gen) in `src/plugins/voxel_world_plugin.rs`
- [X] T006 [P] Implement `TraversalPlugin` as an empty `impl Plugin` stub with doc comment describing future ownership (locomotion, vehicles, aircraft) in `src/plugins/traversal_plugin.rs`
- [X] T007 [P] Implement `LodLevel(pub u8)` component (derive `Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect`) with level table doc comment in `src/plugins/lod/components.rs`
- [X] T008 [P] Implement `LodSettings { thresholds: [f32; 3] }` resource (derive `Resource, Debug, Clone, Reflect`) with `Default` impl `[1_000.0, 10_000.0, 100_000.0]` in `src/plugins/lod/resources.rs`

**Checkpoint**: `cargo check` passes; all five files compile cleanly.

---

## Phase 3: User Story 1 — Stable Launch Experience (Priority: P1) 🎯 MVP

**Goal**: A borderless fullscreen window titled "Neon Expanse v0.0.1-dev" opens, prints two expected console messages, and exits cleanly on window close. All 8 plugins are registered.

**Independent Test**: `cargo run` → borderless fullscreen window opens with correct title, console shows `"Neon Expanse core initialized"` and controller/keyboard status message within 1 second, process exits code 0 on window close with no panics.

- [X] T009 [US1] Implement `neon_window_plugins()` returning `DefaultPlugins` configured with `WindowMode::BorderlessFullscreen(MonitorSelection::Primary)`, `PresentMode::AutoNoVsync`, title `"Neon Expanse v0.0.1-dev"` in `src/plugins/window_plugin.rs`
- [X] T010 [US1] Implement `CorePlugin` inserting `WorldSeed(0x4E45_4F4E_3230_3236_u64)` resource and `Time::<Fixed>::from_hz(64.0)` resource; declare `pub const WORLD_SEED: u64 = 0x4E45_4F4E_3230_3236; // ASCII: "NEON2026"` in `src/plugins/core_plugin.rs`
- [X] T011 [US1] Implement `main()` in `src/main.rs` adding all 8 plugins in dependency order: `neon_window_plugins()`, `CorePlugin`, `NeonInputPlugin`, `PhysicsPlugin`, `FloatingOriginPlugin`, `LodPlugin`, `VoxelWorldPlugin`, `TraversalPlugin`; add all necessary `use` imports

**Checkpoint**: `cargo run` compiles and opens a window (even if black/empty). US1 acceptance scenarios 1, 2, and 5 are provably satisfied.

---

## Phase 4: User Story 2 — Planetary-Scale Origin Stability (Priority: P1)

**Goal**: A cyan test sphere appears on screen 100 m in front of the camera at ~6,000 km global distance; it never drifts, jitters, or produces NaN values; the floating-origin sync pipeline is provably correct across 300+ frames.

**Independent Test**: `cargo run` → cyan sphere visible on screen. Inspect console or add debug print to confirm test entity's `GlobalPosition` ≈ `6_000_000 m` and camera's `GlobalPosition` ≈ `6_000_100 m`. Observe 300 frames with no drift. Run `cargo test` → floating-origin property tests pass.

- [X] T012 [P] [US2] Implement `GlobalPosition(pub DVec3)`, `FloatingOrigin` ZST, and `TestOriginMarker` ZST components (derive `Component, Reflect`; add NaN/infinity invariant doc comments) in `src/plugins/floating_origin/components.rs`
- [X] T013 [P] [US2] Implement `OriginFrame { position: DVec3 }` resource (derive `Resource, Debug, Clone, Copy`) with `Default` impl (`DVec3::ZERO`) and invariant doc comment in `src/plugins/floating_origin/resources.rs`
- [X] T014 [P] [US2] Implement `spawn_test_scene` startup system in `src/plugins/floating_origin/systems.rs`: spawns `Camera3d` + `FloatingOrigin` at `DVec3(6_000_100, 0, 0)`, cyan sphere at `DVec3(6_000_000, 0, 0)` with `RigidBody::Static` + `LodLevel(3)` + `TestOriginMarker`, a point light, and inserts `AmbientLight`; runs in `Startup` schedule
- [X] T014b [P] [US2] Implement `update_origin_frame` system in `src/plugins/floating_origin/systems.rs`: reads the single `FloatingOrigin` entity's `GlobalPosition` and writes it into the `OriginFrame` resource; runs in `PreUpdate`; add `debug_assert!` guarding against NaN/infinity (depends on T012)
- [X] T014c [US2] Implement `sync_transforms` system in `src/plugins/floating_origin/systems.rs`: for every entity with `GlobalPosition`, sets `Transform.translation = (gpos.0 - origin_frame.position).as_vec3()`; add `debug_assert!` guarding NaN; runs in `PostUpdate` `.before(TransformSystem::TransformPropagate)` (depends on T012, T013)
- [X] T015 [US2] Implement `FloatingOriginPlugin` calling `init_resource::<OriginFrame>()`, registering component types, adding `Startup` + `PreUpdate` + `PostUpdate` systems with `.before(TransformSystem::TransformPropagate)` ordering in `src/plugins/floating_origin/plugin.rs`
- [X] T016 [US2] Create `src/plugins/floating_origin/mod.rs` with `pub mod` for `components`, `resources`, `systems`; `pub use plugin::FloatingOriginPlugin`
- [X] T017 [US2] Implement `update_lod_levels` system querying all `(&GlobalPosition, &mut LodLevel)` entities; compute `f32` distance from `OriginFrame`; assign level 0/1/2/3 per `LodSettings` thresholds in `src/plugins/lod/systems.rs`
- [X] T018 [US2] Implement `LodPlugin` calling `init_resource::<LodSettings>()`, registering `LodLevel`/`LodSettings` reflection types, adding `update_lod_levels` to `PostUpdate` in `src/plugins/lod/plugin.rs`
- [X] T019 [US2] Create `src/plugins/lod/mod.rs` with `pub mod` for `components`, `resources`, `systems`; `pub use plugin::LodPlugin`
- [X] T020 [P] [US2] Write three `proptest!` property tests in `tests/floating_origin_tests.rs`: `sync_offset_is_finite_at_planetary_scale` (offset `DVec3→Vec3` is always finite); `same_position_produces_zero_offset`; `camera_transform_is_always_zero`; use `planetary_dvec3()` strategy covering ±20 M m per axis
- [X] T021 [P] [US2] Write two `proptest!` property tests in `tests/lod_tests.rs`: `lod_level_in_range` (level always 0–3); `lod_level_is_deterministic` (same distance → same level); test the pure `classify_lod` helper

**Checkpoint**: `cargo run` shows cyan sphere with no drift across 300 frames. `cargo test` passes all floating-origin and LOD tests.

---

## Phase 5: User Story 3 — Controller and Input Readiness (Priority: P2)

**Goal**: On startup the console reports gamepad status. PS4/PS5 controller connection/disconnection is handled gracefully with dead-zone filtering. All action mappings load from `input.ron` and are editable without recompiling.

**Independent Test**: Launch with PS4/PS5 controller connected → console shows `"PS4/PS5 controller connected: <name>"`. Disconnect mid-session → `"keyboard/mouse fallback active"`. Press a mapped key → named action is handled. Delete `assets/config/input.ron` and re-run → defaults applied, no crash. `cargo test` → RON round-trip tests pass.

- [X] T022 [P] [US3] Implement `InputConfig` struct with `deadzone_radius: f32`, `axis_map`, `button_map`, `keyboard_map` (all `HashMap<String, …>`); derive `Resource, Debug, Clone, Serialize, Deserialize`; implement `Default` with canonical bindings; implement `load_input_config()` synchronously from `"assets/config/input.ron"` with graceful `warn!`-and-fallback on error in `src/plugins/input/config.rs`
- [X] T023 [P] [US3] Implement `ActiveGamepad { entity: Option<Entity> }` resource (derive `Resource, Debug, Default`) in `src/plugins/input/resources.rs`
- [X] T024 [P] [US3] Create `assets/config/input.ron` with RON literal containing `deadzone_radius: 0.15` and action→button/key mappings matching `InputConfig::default()` (MoveForward, MoveRight, LookVertical, LookHorizontal axes; PrimaryAction, Jump, Sprint buttons; keyboard fallback keys)
- [X] T025 [US3] Implement two systems in `src/plugins/input/systems.rs`: `setup_input` (Startup — calls `load_input_config()`, inserts `InputConfig` and `ActiveGamepad` resources, logs `"No controller detected — keyboard/mouse fallback active"`); `on_gamepad_event` (Update — reads `GamepadConnectionEvent`; on `Connected` logs `"PS4/PS5 controller connected: {name}"`, sets `active.entity`, inserts `GamepadSettings` with deadzone from `InputConfig`; on `Disconnected` logs fallback message, clears `active.entity`)
- [X] T026 [US3] Implement `NeonInputPlugin` adding `setup_input` to `Startup` and `on_gamepad_event` to `Update` in `src/plugins/input/plugin.rs`
- [X] T027 [US3] Create `src/plugins/input/mod.rs` with `pub mod` for `config`, `resources`, `systems`; `pub use plugin::NeonInputPlugin`
- [X] T028 [P] [US3] Write tests in `tests/input_config_tests.rs`: `test_input_config_default_is_valid` (default has four axis entries, three button entries, five keyboard entries, deadzone 0.15); `test_ron_round_trip` (serialize default to RON string, deserialize, compare deadzone); `test_missing_file_returns_default` (call `load_input_config()` with no file present — replace path temporarily or test fallback path logic)

**Checkpoint**: `cargo run` with and without controller produces correct console output. `cargo test` passes all input tests.

---

## Phase 6: User Story 4 — Per-Frame Diagnostics Visibility (Priority: P3)

**Goal**: FPS, frame-time, and entity count are logged at 1 Hz throughout the session. Tracy profiler can attach without crashing.

**Independent Test**: Run `cargo run` for 10 seconds → at least 9 diagnostic log lines appear, each containing FPS and entity count. `cargo run --features tracy` starts without panic.

- [X] T029 [US4] Add three diagnostic plugins to `CorePlugin::build()` in `src/plugins/core_plugin.rs`: `FrameTimeDiagnosticsPlugin`, `EntityCountDiagnosticsPlugin`, `LogDiagnosticsPlugin { wait_duration: Duration::from_secs(1), filter: None, ..default() }`; add `use std::time::Duration` import
- [X] T030 [P] [US4] Confirm `tracy = ["bevy/trace_tracy"]` feature entry is present and correct in `Cargo.toml`; run `cargo check --features tracy` to verify the feature gate compiles cleanly; also confirm `cargo run --features tracy` starts without panic (fix any issues)

**Checkpoint**: `cargo run` emits ≥ 1 diagnostic line per second with FPS + entity count. `cargo check --features tracy` passes.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Code quality gates — all must pass before the branch is merged.

- [X] T031 Run `cargo fmt --all` and commit any formatting changes; verify `cargo fmt --check` exits 0
- [X] T032 [P] Run `cargo clippy --all-targets -- -D warnings` and fix every lint warning until it exits 0
- [X] T033 [P] Run `cargo test` and confirm all property tests (floating-origin × 3, LOD × 2, input × 3) pass with exit code 0; run `PROPTEST_CASES=1000 cargo test` for extended coverage
- [X] T034 [P] Verify SC-001 through SC-008 from `specs/001-core-foundation/spec.md` manually against a release build (`cargo run --release`): fullscreen within 2 s, console messages within 1 s, sphere visible + stable for 300 frames, FPS ≥ 60, clean exit code 0; additionally verify FR-005: close the window and confirm exit code 0 (not a crash) as a proxy for the primary-monitor fallback contract
- [X] T035 [P] Install `cargo-llvm-cov` (`cargo install cargo-llvm-cov`); run `cargo llvm-cov --lib` and confirm ≥ 85% line coverage on `src/plugins/floating_origin/` and `src/plugins/input/` modules (Constitution §VI); output HTML report to `target/llvm-cov/html/` for CI artifact upload; fail if either module is under threshold

---

## Dependencies (User Story Completion Order)

```
Phase 1 (Setup)
    └─► Phase 2 (Foundational)
            ├─► Phase 3: US1 (Stable Launch) ← MVP
            │       └─► Phase 4: US2 (Origin Stability)
            │               └─► Phase 5: US3 (Input Readiness)
            │                       └─► Phase 6: US4 (Diagnostics)
            │                               └─► Final Phase (Polish)
            └─► [T007, T008 must precede T014c — LodLevel imported by sync_transforms]
```

**Notable cross-phase dependency**: T007 (`LodLevel` component) and T008 (`LodSettings` resource) in Phase 2 are imported by `floating_origin/systems.rs` (specifically by T014c `sync_transforms` and the test spawn). They must compile before T014c is written.

---

## Parallel Execution Opportunities

### Phase 2 (once T001–T003 pass `cargo check`)

```
T004 physics_plugin.rs
T005 voxel_world_plugin.rs  ──── all four can be written simultaneously ────
T006 traversal_plugin.rs
T007 lod/components.rs
T008 lod/resources.rs
```

### Phase 4 (once T011 compiles)

```
T012 floating_origin/components.rs  ─┐
T013 floating_origin/resources.rs   ─┤
T014 spawn_test_scene  [P]          ─┤─► T014c sync_transforms ─► T015 plugin.rs ─► T016 mod.rs
T014b update_origin_frame [P]       ─┘   (depends T012, T013)

T017 lod/systems.rs ─► T018 plugin.rs ─► T019 mod.rs    (separate thread, can run alongside)

T020 tests/floating_origin_tests.rs  ─┐── both test files written in parallel
T021 tests/lod_tests.rs              ─┘
```

### Phase 5 (once T011 compiles — US3 is independent of US2)

```
T022 input/config.rs    ─┐
T023 input/resources.rs ─┤─► T025 input/systems.rs ─► T026 input/plugin.rs ─► T027 mod.rs
T024 assets/input.ron   ─┘

T028 tests/input_config_tests.rs   (can be written alongside T022–T024)
```

---

## Implementation Strategy

### MVP Scope (Phases 1–3 only: T001–T011)

Completing only Phases 1–3 delivers a **compilable, launchable game** with:
- Borderless fullscreen window with correct title (SC-001, SC-002)
- All 8 plugins registered with correct dependency order (SC-007)
- Clean exit on window close (FR-004)
- Console messages confirming core initialized + input status (FR-003, FR-015)
- 64 Hz fixed timestep active (FR-019)

This is the minimum useful deliverable. Every task after T011 adds user-visible correctness proofs.

### Incremental Delivery

1. **T001–T011** (MVP): compiling window opens
2. **T012–T021** (+US2): cyan sphere at planetary distance, no drift (includes T014b, T014c)
3. **T022–T028** (+US3): controller detection + input.ron working
4. **T029–T030** (+US4): 1 Hz diagnostics visible in console
5. **T031–T035** (Polish): branch ready to merge

---

## Summary

| Scope | Count |
|-------|-------|
| Total tasks | 37 |
| Setup (Phase 1) | 3 |
| Foundational (Phase 2) | 5 |
| US1 — Stable Launch (P1) | 3 |
| US2 — Origin Stability (P1) | 12 |
| US3 — Input Readiness (P2) | 7 |
| US4 — Diagnostics (P3) | 2 |
| Polish (Final Phase) | 5 |
| Parallelizable [P] tasks | 21 |
| **MVP scope** (Phases 1–3) | **11** |

**Independent test criteria per story:**
- **US1**: `cargo run` → window + console messages + exit code 0
- **US2**: visible sphere + 300 stable frames + `cargo test` floating-origin/LOD pass
- **US3**: controller hot-plug console output + `cargo test` input tests pass
- **US4**: ≥ 1 diagnostic line/sec with FPS + entity count for 10 s
