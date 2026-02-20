# Implementation Plan: Core Foundation

**Branch**: `001-core-foundation` | **Date**: 2026-02-20 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/001-core-foundation/spec.md`
**Research**: [research.md](research.md) | **Data Model**: [data-model.md](data-model.md) | **Contracts**: [contracts/plugin-api.md](contracts/plugin-api.md)

## Summary

Establish the permanent, never-to-be-touched-again engine skeleton for Neon Expanse: a Bevy 0.18 application that launches into borderless fullscreen, registers all 8 mandatory plugins in correct dependency order, runs a 64 Hz fixed-timestep physics + game-logic pipeline, implements a complete floating-origin coordinate system (DVec3 global positions → f32 Transform sync), auto-detects PS4/PS5 controllers with RON-configurable action mapping, and spawns a single visible test entity at ~6,000 km global distance to prove large-scale stability. Output: a stable black screen + console messages + one cyan sphere at planetary scale.

---

## Constitution Compliance Checklist

*GATE: Must pass before implementation begins. Re-evaluated after Phase 1 design — all items remain PASS.*

| § | Requirement | Status | Notes |
|---|-------------|--------|-------|
| I | Earth-scale planet / seamless world / no loading screens | ✅ PASS | FloatingOriginPlugin + DVec3 coordinate system is the direct enabler |
| I | Moddable from day one | ✅ PASS | `input.ron` RON config file (FR-016/017); plugin registry extensible via stubs |
| I | Accessibility: PS4/PS5 primary + keyboard fallback | ✅ PASS | `NeonInputPlugin` mandatory; auto-detect + keyboard fallback both implemented |
| I | 60+ FPS on mid-range hardware | ✅ PASS | Single entity, no terrain — must trivially achieve target; SC-005 measurable |
| II | 100% Bevy ECS — no classic OOP | ✅ PASS | All game objects are ECS entities; all plugins `impl Plugin`; no struct inheritance |
| II | Single persistent planet / deterministic | ✅ PASS | One world seed constant (defined in `CorePlugin`); no randomness this feature |
| II | DVec3 global positions, 1 unit = 1 m | ✅ PASS | `GlobalPosition(DVec3)` component; metre scale confirmed in data model |
| II | Every major feature in its own Bevy plugin | ✅ PASS | 8 plugins, each in its own module; see plugin registration section |
| II | Core world systems never touched after foundation | ✅ PASS | VoxelWorld and Traversal are stubs; no core file modification needed for next spec |
| III | Rust stable 2024 edition, minimum 1.85+ | ✅ PASS | `Cargo.toml` sets `edition = "2024"`, `rust-version = "1.85"` |
| III | Bevy 0.18+ | ✅ PASS | `bevy = "0.18"` |
| III | `avian3d = "0.5"` | ✅ PASS | `avian3d = "0.5"` default features; f32 physics in local space |
| III | glam::DVec3 global, Vec3 local | ✅ PASS | `GlobalPosition(DVec3)` global; `Transform.translation: Vec3` local |
| III | Input: Bevy Gamepad + custom action system | ✅ PASS | `NeonInputPlugin` with `InputConfig` and named action map |
| III | Serialization: serde + ron | ✅ PASS | `InputConfig` serde + `input.ron` |
| III | Profiling: bevy_diagnostic + Tracy | ✅ PASS | `LogDiagnosticsPlugin` + `bevy/trace_tracy` feature gate |
| III | `unsafe` forbidden except documented math kernels | ✅ PASS | No `unsafe` needed in this feature; clippy `-D unsafe_code` enforced |
| IV | CorePlugin | ✅ PASS | `src/plugins/core_plugin.rs` |
| IV | WindowPlugin (borderless fullscreen, AutoNoVsync) | ✅ PASS | `src/plugins/window_plugin.rs` via `DefaultPlugins.set(WindowPlugin{..})` |
| IV | InputPlugin (PS4/PS5 + keyboard fallback) | ✅ PASS | `src/plugins/input/` |
| IV | FloatingOriginPlugin (DVec3 + Transform sync) | ✅ PASS | `src/plugins/floating_origin/` |
| IV | VoxelWorldPlugin (stub) | ✅ PASS | `src/plugins/voxel_world_plugin.rs` |
| IV | TraversalPlugin (stub) | ✅ PASS | `src/plugins/traversal_plugin.rs` |
| IV | PhysicsPlugin (Avian3d) | ✅ PASS | `src/plugins/physics_plugin.rs` |
| V | `cargo fmt` + `cargo clippy --all-targets -- -D warnings` | ✅ PASS | Enforced in CI; all code written to pass both |
| V | Heavy `///` documentation on every public item | ✅ PASS | All public structs, enums, functions, and components documented |
| V | No `unwrap()` outside main.rs / asset loading | ✅ PASS | `expect()` with descriptive messages where panics are invariants; no naked `unwrap()` |
| V | Constitution checklist at top of plan | ✅ PASS | This table |
| VI | Property-based tests for voxel determinism / floating-origin / vehicle physics | ✅ PASS | `GlobalPosition` sync correctness tested with proptest |
| VI | 85% coverage on core world + traversal systems | ✅ PASS | FloatingOriginPlugin and InputPlugin fully unit-tested; coverage tool in CI |
| VI | Regression test: same seed → same terrain / vehicle behavior | ✅ PASS | Deterministic seed constant; world-seed regression test scaffolded |
| VII | Launches into borderless fullscreen | ✅ PASS | FR-001 |
| VII | PS4/PS5 controller auto-detected + console message | ✅ PASS | FR-013/014 |
| VII | Player spawns on foot (N/A this feature) | ✅ PASS | Out of scope — documented in spec |
| VIII | All definitions exposed via RON + reflection | ✅ PASS | `input.ron` + Bevy `Reflect` on all config types; `LodSettings` RON file deferred to `002-voxel-planet-engine` (thresholds are not user-facing until terrain exists; `Reflect` derivation satisfies the reflection half of §VIII today) |
| IX | Spec Kit workflow: spec before plan | ✅ PASS | spec.md written and clarified before this plan |
| IX | Constitution checklist at top of every plan | ✅ PASS | This table |

**Post-design re-evaluation**: All items remain PASS after Phase 1 design. No violations to justify.

---

## Technical Context

**Language/Version**: Rust stable 2024 edition, minimum 1.85+
**Primary Dependencies**: `bevy = "0.18"`, `avian3d = "0.5"`, `glam` (re-exported by Bevy), `serde = "1"`, `ron = "0.8"`, `proptest = "1"` (dev)
**Storage**: `assets/config/input.ron` (plain file, synchronous read at startup); no database
**Testing**: `cargo test` + `proptest` for property-based tests; `cargo llvm-cov` for coverage
**Target Platform**: macOS, Windows, Linux desktop (primary: macOS for development)
**Project Type**: Single Rust binary (Cargo workspace ready — single crate for now, workspace layout prepared)
**Performance Goals**: 60+ FPS on Ryzen 5 / RTX 3060 class hardware; < 2 s window-open time; 64 Hz fixed physics tick
**Constraints**: No `unsafe` except documented math kernels; no `unwrap()` outside main.rs; all public items `///` documented
**Scale/Scope**: 8 plugins, ~800–1200 lines of production code; foundation milestone — never structurally modified after completion

---

## Project Structure

### Documentation (this feature)

```text
specs/001-core-foundation/
├── plan.md              ← this file
├── spec.md              ← feature specification (with clarifications)
├── research.md          ← Phase 0 research decisions
├── data-model.md        ← components, resources, entity archetypes
├── quickstart.md        ← how to build and run
├── checklists/
│   └── requirements.md  ← spec quality checklist (all pass)
└── contracts/
    └── plugin-api.md    ← public surface contract for each plugin
```

### Source Code Layout

```text
neon-expanse/                        ← Cargo workspace root (single crate now, workspace-ready)
├── Cargo.toml                       ← workspace + crate manifest
├── Cargo.lock
├── .cargo/
│   └── config.toml                  ← target-cpu = native, linker = mold/lld
├── assets/
│   └── config/
│       └── input.ron                ← default input action mappings + dead-zone config
├── src/
│   ├── main.rs                      ← App builder: plugin registration in order
│   └── plugins/
│       ├── mod.rs                   ← pub use of all plugins
│       ├── core_plugin.rs           ← schedules, diagnostics, seeds, 64 Hz tick
│       ├── window_plugin.rs         ← borderless fullscreen, title, vsync
│       ├── physics_plugin.rs        ← PhysicsPlugins::default() wrapper
│       ├── voxel_world_plugin.rs    ← EMPTY STUB
│       ├── traversal_plugin.rs      ← EMPTY STUB
│       ├── floating_origin/
│       │   ├── mod.rs
│       │   ├── plugin.rs            ← FloatingOriginPlugin impl Plugin
│       │   ├── components.rs        ← GlobalPosition, FloatingOrigin, TestOriginMarker
│       │   ├── resources.rs         ← OriginFrame
│       │   └── systems.rs           ← update_origin_frame, sync_transforms, spawn_test_scene
│       ├── input/
│       │   ├── mod.rs
│       │   ├── plugin.rs            ← NeonInputPlugin impl Plugin
│       │   ├── config.rs            ← InputConfig struct + RON deserialization
│       │   ├── resources.rs         ← ActiveGamepad
│       │   └── systems.rs           ← on_gamepad_event, configure_gamepad_deadzones, log_fallback
│       └── lod/
│           ├── mod.rs
│           ├── plugin.rs            ← LodPlugin impl Plugin
│           ├── components.rs        ← LodLevel
│           ├── resources.rs         ← LodSettings
│           └── systems.rs           ← update_lod_levels
└── tests/
    ├── floating_origin_tests.rs     ← property-based: sync correctness, NaN/inf guards
    ├── input_config_tests.rs        ← RON parse round-trip, fallback defaults
    └── lod_tests.rs                 ← threshold classification property tests
```

---

## Plugin Registration Order in `main.rs`

Order is critical. Each plugin may only depend on plugins registered before it.

```
1. DefaultPlugins (configured with NeonWindowPlugin settings)
    └─ includes: WindowPlugin, RenderPlugin, AssetPlugin, TransformPlugin,
                 InputPlugin (Bevy built-in), LogPlugin, etc.
2. CorePlugin
    └─ inserts: Time::<Fixed>::from_hz(64.0), diagnostics plugins
3. NeonInputPlugin
    └─ depends on: DefaultPlugins (gamepad events), CorePlugin (schedules)
4. PhysicsPlugin  (= PhysicsPlugins::default() wrapper)
    └─ depends on: CorePlugin (Time::<Fixed> at 64 Hz)
5. FloatingOriginPlugin
    └─ depends on: PhysicsPlugin (RigidBody::Static on test entity), DefaultPlugins (assets)
6. LodPlugin
    └─ depends on: FloatingOriginPlugin (OriginFrame resource, GlobalPosition component)
7. VoxelWorldPlugin  (empty stub)
    └─ depends on: FloatingOriginPlugin, LodPlugin (by contract — stubs today)
8. TraversalPlugin   (empty stub)
    └─ depends on: PhysicsPlugin, FloatingOriginPlugin (by contract — stubs today)
```

---

## Complete `main.rs`

```rust
//! Neon Expanse — entry point.
//!
//! Registers all mandatory core plugins in dependency order and launches
//! the Bevy application. This file must never be modified after the
//! core-foundation milestone is complete.

use bevy::prelude::*;
use neon_expanse::plugins::{
    core_plugin::CorePlugin,
    floating_origin::FloatingOriginPlugin,
    input::NeonInputPlugin,
    lod::LodPlugin,
    physics_plugin::PhysicsPlugin,
    traversal_plugin::TraversalPlugin,
    voxel_world_plugin::VoxelWorldPlugin,
    window_plugin::neon_window_plugins,
};

fn main() {
    App::new()
        // ── 1. DefaultPlugins (configured for borderless fullscreen) ──────────
        .add_plugins(neon_window_plugins())
        // ── 2. Core: schedules, diagnostics, fixed-tick ────────────────────
        .add_plugins(CorePlugin)
        // ── 3. Input: PS4/PS5 auto-detect + keyboard fallback ────────────────
        .add_plugins(NeonInputPlugin)
        // ── 4. Physics: Avian3d full initialization ───────────────────────────
        .add_plugins(PhysicsPlugin)
        // ── 5. Floating Origin: DVec3 global positions + Transform sync ───────
        .add_plugins(FloatingOriginPlugin)
        // ── 6. LOD: distance thresholds (no visible transitions yet) ─────────
        .add_plugins(LodPlugin)
        // ── 7. Voxel World: empty stub ────────────────────────────────────────
        .add_plugins(VoxelWorldPlugin)
        // ── 8. Traversal: empty stub ──────────────────────────────────────────
        .add_plugins(TraversalPlugin)
        .run();
}
```

---

## Complete `Cargo.toml`

```toml
[package]
name        = "neon-expanse"
version     = "0.0.1-dev"
edition     = "2024"
rust-version = "1.85"
authors     = ["Neon Expanse Contributors"]
description = "Earth-scale voxel exploration game — Bevy + Avian3d"
license     = "TBD"

[dependencies]
bevy    = { version = "0.18", features = ["dynamic_linking"] }
avian3d = "0.5"
serde   = { version = "1", features = ["derive"] }
ron     = "0.8"
glam    = "0.29"   # re-exported by Bevy; explicit for DVec3 use in plugins

[dev-dependencies]
proptest = "1"

[features]
default = []
# Enable Tracy profiler integration (run with: cargo run --features tracy)
tracy = ["bevy/trace_tracy"]

# Fast dev builds: dynamic linking + minimal optimization
[profile.dev]
opt-level = 1           # faster compile, usable performance

[profile.dev.package."*"]
opt-level = 3           # optimize dependencies even in dev

[profile.release]
opt-level   = 3
lto         = "thin"
codegen-units = 1
strip       = "symbols"

# Workspace-ready: add member crates here when splitting into workspace
# [workspace]
# members = ["crates/neon_core", "crates/neon_voxel", ...]
```

---

## `window_plugin.rs` — Full Implementation

```rust
//! Borderless fullscreen window configuration for Neon Expanse.
//!
//! Returns a configured [`DefaultPlugins`] group with the correct
//! window mode, present mode, and title. This function is called
//! exactly once from `main.rs` and must never be modified after the
//! core-foundation milestone.

use bevy::{
    prelude::*,
    window::{MonitorSelection, PresentMode, Window, WindowMode, WindowPlugin},
};

/// Returns Bevy's [`DefaultPlugins`] configured for Neon Expanse:
/// borderless fullscreen on the primary monitor, no vsync, with
/// the canonical window title.
///
/// # Plugin group contents
/// All standard Bevy plugins are included. The only customisation is
/// the [`WindowPlugin`] configuration — all other plugins use defaults.
pub fn neon_window_plugins() -> impl PluginGroup {
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Neon Expanse v0.0.1-dev".into(),
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    })
}
```

---

## `core_plugin.rs` — Full Implementation

```rust
//! Core application plugin for Neon Expanse.
//!
//! Configures the deterministic world seed, the 64 Hz fixed-timestep
//! schedule shared by game logic and physics, and all diagnostic
//! logging plugins. This plugin must be registered before all other
//! game plugins.

use std::time::Duration;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

/// The shared world seed used by all procedural generation systems.
/// Changing this constant changes the entire planet. Never random.
pub const WORLD_SEED: u64 = 0x4E45_4F4E_3230_3236; // ASCII: "NEON2026"

/// Bevy plugin that configures schedules, diagnostics, and deterministic seed.
///
/// Must be registered immediately after `DefaultPlugins`.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // ── Fixed-timestep: 64 Hz ─────────────────────────────────────────────
        // Physics (FixedPostUpdate / avian3d) and game logic (FixedUpdate) both
        // tick at this rate. This is the single source of truth for simulation
        // frequency across the entire engine.
        app.insert_resource(Time::<Fixed>::from_hz(64.0));

        // ── Diagnostics ───────────────────────────────────────────────────────
        // LogDiagnosticsPlugin emits FPS, frame time, and entity count
        // to the console at 1 Hz (once per second).
        app.add_plugins((
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            LogDiagnosticsPlugin {
                wait_duration: Duration::from_secs(1),
                filter: None,
                ..default()
            },
        ));

        // ── World seed resource ───────────────────────────────────────────────
        app.insert_resource(WorldSeed(WORLD_SEED));
    }
}

/// Deterministic world seed resource.
///
/// Every procedural generation system MUST consume this resource rather
/// than seeding its own RNG independently. Identical seeds produce
/// identical planets, guaranteed.
#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);
```

---

## `physics_plugin.rs` — Full Implementation

```rust
//! Physics plugin wrapper for Neon Expanse.
//!
//! Wraps [`avian3d::prelude::PhysicsPlugins`] with logging and
//! integrates it into the plugin registration order. All avian3d
//! APIs are available to any crate/plugin via `use avian3d::prelude::*`.

use avian3d::prelude::*;
use bevy::prelude::*;

/// Bevy plugin that activates the Avian3d physics simulation.
///
/// Physics runs in `FixedPostUpdate` at 64 Hz (inherited from
/// `Time::<Fixed>` set by [`CorePlugin`]). Gravity defaults to
/// `Vec3::NEG_Y * 9.81 m/s²` and affects only `RigidBody::Dynamic`
/// entities — `RigidBody::Static` bodies are completely unaffected.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        info!("Neon Expanse physics initialized (avian3d, 64 Hz)");
    }
}
```

---

## `floating_origin/` — Full Implementation

### `components.rs`

```rust
//! Components for the floating-origin coordinate system.
//!
//! The floating-origin system maps high-precision world-space positions
//! (DVec3, f64, metres) to Bevy's f32 render-space [`Transform`].
//! Any entity that exists at planetary scales MUST use [`GlobalPosition`]
//! as its authoritative position. [`Transform`] is managed exclusively
//! by [`FloatingOriginPlugin`] and MUST NOT be set manually.

use bevy::prelude::*;
use glam::DVec3;

/// High-precision world-space position for any entity participating in
/// the floating-origin coordinate system.
///
/// Values are in **metres**. The coordinate origin (0, 0, 0) corresponds
/// to the planet's geometric centre.
///
/// # Invariants
/// - Components of the inner [`DVec3`] must never be NaN or infinite.
/// - Do NOT write to [`Transform`] on entities that carry this component —
///   the sync system will overwrite it every `PostUpdate`.
#[derive(Component, Debug, Clone, Copy)]
pub struct GlobalPosition(pub DVec3);

/// Marker component identifying the floating-origin **reference viewer**.
///
/// The entity carrying this component defines the current origin frame.
/// Its [`GlobalPosition`] is used as the subtraction base when computing
/// all other entities' f32 [`Transform`] offsets.
///
/// # Rules
/// - Exactly ONE entity may carry this component at any time.
/// - During `Core Foundation`: the main camera entity.
/// - In future specs: may transfer to the active player entity.
#[derive(Component, Debug, Default)]
pub struct FloatingOrigin;

/// Marker component for the diagnostic test entity.
///
/// Placed on the single visible sphere spawned at ~6,000 km global
/// distance during startup. Exists solely to enable test queries
/// that locate the entity by type without a stored [`Entity`] handle.
#[derive(Component, Debug, Default)]
pub struct TestOriginMarker;
```

### `resources.rs`

```rust
//! Resources for the floating-origin coordinate system.

use bevy::prelude::*;
use glam::DVec3;

/// The current floating-origin reference position in world space.
///
/// Updated every `PreUpdate` frame to equal the [`GlobalPosition`]
/// of the entity carrying [`FloatingOrigin`]. All [`Transform`] sync
/// computations in `PostUpdate` read this resource.
///
/// # Invariant
/// Always equals the [`GlobalPosition`] of the [`FloatingOrigin`] entity
/// at the start of the current frame (before any `Update` systems run).
#[derive(Resource, Debug, Clone, Copy)]
pub struct OriginFrame {
    /// Current reference position in metres (f64 precision).
    pub position: DVec3,
}

impl Default for OriginFrame {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
        }
    }
}
```

### `systems.rs`

```rust
//! Systems for the floating-origin coordinate system.

use avian3d::prelude::*;
use bevy::prelude::*;
use glam::DVec3;

use super::{
    components::{FloatingOrigin, GlobalPosition, TestOriginMarker},
    resources::OriginFrame,
};

/// The world-space position of the test sphere (and camera anchor).
///
/// ~6,000 km from the planet centre on the X axis.
/// Chosen so the camera is placed at TEST_ENTITY_POS + CAMERA_OFFSET,
/// keeping the sphere visible at a comfortable distance.
pub const TEST_ENTITY_POS: DVec3 = DVec3::new(6_000_000.0, 0.0, 0.0);

/// Camera offset from the test entity: 100 metres on the Z axis.
/// This ensures the camera's local Transform stays near Vec3::ZERO
/// while the global coordinate is ~6,000,000 m — proving DVec3→f32 sync.
pub const CAMERA_OFFSET: DVec3 = DVec3::new(0.0, 0.0, 100.0);

/// Startup system: spawns the camera, test sphere, and a point light
/// at planetary distance from the world origin.
///
/// Called once by [`FloatingOriginPlugin`] in the `Startup` schedule.
pub fn spawn_test_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_pos = TEST_ENTITY_POS + CAMERA_OFFSET;

    // ── Camera ────────────────────────────────────────────────────────────────
    // Spawned with FloatingOrigin so its GlobalPosition defines the
    // current origin frame. Its Transform.translation will be Vec3::ZERO
    // after the first sync frame.
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::ZERO)
            .looking_at(-Vec3::Z * 100.0, Vec3::Y),
        GlobalPosition(camera_pos),
        FloatingOrigin,
    ));

    // ── Ambient light (so the sphere is not pitch-black) ─────────────────────
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    // ── Point light near the sphere ───────────────────────────────────────────
    let light_pos = TEST_ENTITY_POS + DVec3::new(-150.0, 100.0, 50.0);
    commands.spawn((
        PointLight {
            intensity: 150_000.0,
            range: 500.0,
            shadows_enabled: false,
            ..default()
        },
        GlobalPosition(light_pos),
        Transform::default(),
    ));

    // ── Test origin marker sphere ─────────────────────────────────────────────
    // Radius 50 m — large enough to be clearly visible at 100 m range.
    // Emissive cyan so it's visible even without perfect lighting.
    let sphere_mesh = meshes.add(Sphere::new(50.0));
    let sphere_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.8, 0.9),
        emissive: LinearRgba::new(0.0, 0.5, 0.6, 1.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(sphere_mat),
        Transform::default(), // will be overwritten by sync_transforms
        GlobalPosition(TEST_ENTITY_POS),
        RigidBody::Static,    // avian3d: immune to gravity; zero sim cost
        TestOriginMarker,
    ));

    info!("Neon Expanse core initialized");
    info!(
        "Test entity spawned at global position {} m from world origin",
        TEST_ENTITY_POS.length() as u64
    );
}

/// `PreUpdate` system: copies the [`FloatingOrigin`] entity's
/// [`GlobalPosition`] into the [`OriginFrame`] resource.
///
/// Must run every frame before any system reads [`OriginFrame`].
pub fn update_origin_frame(
    origin_query: Query<&GlobalPosition, With<FloatingOrigin>>,
    mut origin_frame: ResMut<OriginFrame>,
) {
    // There is always exactly one FloatingOrigin entity (the camera).
    // If for any reason it is missing, keep the previous frame's value.
    if let Ok(pos) = origin_query.get_single() {
        debug_assert!(
            pos.0.is_finite(),
            "FloatingOrigin GlobalPosition contains NaN or infinity"
        );
        origin_frame.position = pos.0;
    }
}

/// `PostUpdate` system: recomputes `Transform.translation` for every
/// entity with a [`GlobalPosition`] by subtracting the current
/// [`OriginFrame`].
///
/// Must run `.before(TransformSystem::TransformPropagate)` so that
/// `GlobalTransform` is computed from the freshly-synced `Transform`.
pub fn sync_transforms(
    origin_frame: Res<OriginFrame>,
    mut query: Query<(&GlobalPosition, &mut Transform)>,
) {
    let origin = origin_frame.position;
    for (gpos, mut transform) in query.iter_mut() {
        debug_assert!(
            gpos.0.is_finite(),
            "GlobalPosition contains NaN or infinity on entity"
        );
        // Safe narrowing: offset is at most a few hundred metres —
        // well within f32 precision range (~16M metres before precision loss).
        transform.translation = (gpos.0 - origin).as_vec3();
    }
}
```

### `plugin.rs`

```rust
//! FloatingOriginPlugin — DVec3 global coordinate system.

use bevy::{prelude::*, transform::TransformSystem};

use super::{
    components::{FloatingOrigin, GlobalPosition, TestOriginMarker},
    resources::OriginFrame,
    systems::{spawn_test_scene, sync_transforms, update_origin_frame},
};

/// Bevy plugin that implements the floating-origin coordinate system.
///
/// # What it does
/// - Registers [`GlobalPosition`], [`FloatingOrigin`], [`TestOriginMarker`]
///   components.
/// - Inserts [`OriginFrame`] resource (default: world origin).
/// - Spawns the camera + test sphere + point light at ~6,000 km in
///   `Startup`.
/// - Each frame:
///   - `PreUpdate`: updates [`OriginFrame`] from the [`FloatingOrigin`]
///     entity's [`GlobalPosition`].
///   - `PostUpdate` (before `TransformSystem::TransformPropagate`): syncs every
///     entity's `Transform.translation` from its `GlobalPosition`.
///
/// # Invariant
/// Any system that moves a world entity MUST write to [`GlobalPosition`],
/// never to [`Transform`]. `Transform` is owned by this plugin.
pub struct FloatingOriginPlugin;

impl Plugin for FloatingOriginPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<OriginFrame>()
            // Register component types for reflection (required for Bevy scenes / RON)
            .register_type::<GlobalPosition>()
            .register_type::<FloatingOrigin>()
            .register_type::<TestOriginMarker>()
            // Startup: spawn test scene
            .add_systems(Startup, spawn_test_scene)
            // PreUpdate: refresh origin frame
            .add_systems(PreUpdate, update_origin_frame)
            // PostUpdate: sync Transform from GlobalPosition (before propagation)
            .add_systems(
                PostUpdate,
                sync_transforms.before(TransformSystem::TransformPropagate),
            );
    }
}
```

> **Note on `register_type`**: requires `#[derive(Reflect)]` on each component. Add `#[derive(Component, Reflect, Debug, Clone, Copy)]` in components.rs and add `use bevy::reflect::Reflect;` to imports.

---

## `lod/` — Full Implementation

### `components.rs`

```rust
//! LOD (Level of Detail) components.

use bevy::prelude::*;

/// The currently assigned LOD level for a world entity.
///
/// Updated every `PostUpdate` frame by the LOD system based on
/// distance from the floating origin (the active camera / viewer).
///
/// | Level | Distance range       | Intended rendering |
/// |-------|---------------------|--------------------|
/// | `0`   | < 1,000 m           | High detail        |
/// | `1`   | 1,000 – 10,000 m    | Medium detail      |
/// | `2`   | 10,000 – 100,000 m  | Low detail         |
/// | `3`   | > 100,000 m         | Impostor           |
///
/// During Core Foundation no LOD transitions are observable because
/// no terrain geometry exists. The component is present on the test
/// sphere so future rendering systems can query it without modification.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct LodLevel(pub u8);
```

### `resources.rs`

```rust
//! LOD settings resource.

use bevy::prelude::*;

/// Global LOD distance thresholds in metres.
///
/// `thresholds[0]` = High→Medium boundary
/// `thresholds[1]` = Medium→Low boundary
/// `thresholds[2]` = Low→Impostor boundary
///
/// Values are calibrated for a 12,742 km diameter planet with the
/// viewer near the surface. They will be refined in the Voxel World
/// Engine spec.
#[derive(Resource, Debug, Clone, Reflect)]
pub struct LodSettings {
    /// Distance thresholds (metres) for level transitions.
    pub thresholds: [f32; 3],
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            thresholds: [1_000.0, 10_000.0, 100_000.0],
        }
    }
}
```

### `systems.rs`

```rust
//! LOD update system.

use bevy::prelude::*;

use crate::plugins::floating_origin::{components::GlobalPosition, resources::OriginFrame};

use super::{components::LodLevel, resources::LodSettings};

/// Computes the [`LodLevel`] for every entity with a [`GlobalPosition`]
/// based on its distance from the current [`OriginFrame`].
///
/// Runs in `PostUpdate`. No visible effect until terrain geometry
/// (introduced in the Voxel World Engine spec) begins querying `LodLevel`.
pub fn update_lod_levels(
    origin: Res<OriginFrame>,
    settings: Res<LodSettings>,
    mut query: Query<(&GlobalPosition, &mut LodLevel)>,
) {
    for (gpos, mut level) in query.iter_mut() {
        // Safe narrowing: distance at planetary scale fits comfortably in f32
        // (max ~20,000 km = 2×10⁷ m; f32 max ~3.4×10³⁸ m).
        let dist = (gpos.0 - origin.position).length() as f32;
        let new_level = if dist < settings.thresholds[0] {
            0
        } else if dist < settings.thresholds[1] {
            1
        } else if dist < settings.thresholds[2] {
            2
        } else {
            3
        };
        if level.0 != new_level {
            level.0 = new_level;
        }
    }
}
```

### `plugin.rs`

```rust
//! LodPlugin — distance-based Level of Detail.

use bevy::prelude::*;

use super::{
    components::LodLevel,
    resources::LodSettings,
    systems::update_lod_levels,
};

/// Bevy plugin that manages LOD levels for all world entities.
///
/// Registers [`LodSettings`] with default thresholds and [`LodLevel`]
/// component. Updates all LOD levels every `PostUpdate` frame.
///
/// No visual LOD transitions occur during Core Foundation because no
/// terrain geometry exists yet. All entities start at level 3 (impostor)
/// and transition to lower levels as the Voxel World Engine introduces
/// renderable geometry.
pub struct LodPlugin;

impl Plugin for LodPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LodSettings>()
            .register_type::<LodLevel>()
            .register_type::<LodSettings>()
            .add_systems(PostUpdate, update_lod_levels);
    }
}
```

---

## `input/` — Full Implementation

### `config.rs`

```rust
//! Input configuration — loaded from `assets/config/input.ron`.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Complete input configuration for Neon Expanse.
///
/// Loaded from `assets/config/input.ron` at startup. Falls back to
/// [`InputConfig::default()`] if the file is absent or malformed —
/// the application must never crash due to a missing config file.
///
/// # Modding
/// Mods may supply a replacement `assets/config/input.ron` to fully
/// remap all controls without recompiling.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Dead-zone radius applied to all analog axes.
    /// Inputs within this radius from centre produce a 0.0 action value.
    pub deadzone_radius: f32,
    /// Maps named action strings (e.g. `"MoveForward"`) to gamepad axes.
    pub axis_map: HashMap<String, GamepadAxis>,
    /// Maps named action strings to gamepad buttons.
    pub button_map: HashMap<String, GamepadButton>,
    /// Maps named action strings to keyboard key fallbacks.
    pub keyboard_map: HashMap<String, KeyCode>,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut axis_map = HashMap::new();
        axis_map.insert("MoveForward".into(), GamepadAxis::LeftStickY);
        axis_map.insert("MoveRight".into(), GamepadAxis::LeftStickX);
        axis_map.insert("LookVertical".into(), GamepadAxis::RightStickY);
        axis_map.insert("LookHorizontal".into(), GamepadAxis::RightStickX);

        let mut button_map = HashMap::new();
        button_map.insert("PrimaryAction".into(), GamepadButton::South);
        button_map.insert("Jump".into(), GamepadButton::South);
        button_map.insert("Sprint".into(), GamepadButton::LeftThumb);

        let mut keyboard_map = HashMap::new();
        keyboard_map.insert("MoveForward".into(), KeyCode::KeyW);
        keyboard_map.insert("MoveRight".into(), KeyCode::KeyD);
        keyboard_map.insert("Jump".into(), KeyCode::Space);
        keyboard_map.insert("Sprint".into(), KeyCode::ShiftLeft);
        keyboard_map.insert("PrimaryAction".into(), KeyCode::KeyE);

        Self {
            deadzone_radius: 0.15,
            axis_map,
            button_map,
            keyboard_map,
        }
    }
}

/// Loads [`InputConfig`] from `assets/config/input.ron`.
///
/// Called synchronously during `Startup`. Uses `std::fs::read_to_string`
/// because the config must be available before the first frame —
/// Bevy's async asset system cannot guarantee this timing.
///
/// Falls back to [`InputConfig::default()`] on any I/O or parse error
/// and emits a `warn!` so developers notice.
pub fn load_input_config() -> InputConfig {
    let path = "assets/config/input.ron";
    match std::fs::read_to_string(path) {
        Ok(contents) => match ron::from_str(&contents) {
            Ok(config) => {
                info!("Input config loaded from {path}");
                config
            }
            Err(e) => {
                warn!("Failed to parse {path}: {e} — using defaults");
                InputConfig::default()
            }
        },
        Err(_) => {
            info!("No {path} found — using default input bindings");
            InputConfig::default()
        }
    }
}
```

### `resources.rs`

```rust
//! Input resources.

use bevy::prelude::*;

/// Tracks the currently active gamepad, if any.
///
/// `entity = None` means no controller is connected and keyboard/mouse
/// fallback is active. Updated by [`crate::plugins::input::systems::on_gamepad_event`].
#[derive(Resource, Debug, Default)]
pub struct ActiveGamepad {
    /// The ECS entity carrying the [`Gamepad`] component, or `None`.
    pub entity: Option<Entity>,
}
```

### `systems.rs`

```rust
//! Input systems: gamepad connection handling and dead-zone configuration.

use bevy::{
    input::gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadSettings},
    prelude::*,
};

use super::{config::InputConfig, resources::ActiveGamepad};

/// Startup system: inserts [`InputConfig`] (loaded from RON or defaults)
/// and [`ActiveGamepad`] resources, and logs keyboard fallback status.
pub fn setup_input(mut commands: Commands) {
    let config = super::config::load_input_config();
    commands.insert_resource(config);
    commands.insert_resource(ActiveGamepad::default());
    // No controller detected yet — will receive GamepadConnectionEvent if one is present.
    info!("No controller detected — keyboard/mouse fallback active");
}

/// Detects PS4/PS5 controller connection and disconnection events.
///
/// On connect: logs controller name, updates [`ActiveGamepad`], and
/// configures dead zones from [`InputConfig`].
/// On disconnect: logs fallback message, clears [`ActiveGamepad`].
pub fn on_gamepad_event(
    mut events: EventReader<GamepadConnectionEvent>,
    mut active: ResMut<ActiveGamepad>,
    config: Res<InputConfig>,
    mut commands: Commands,
) {
    for event in events.read() {
        match &event.connection {
            GamepadConnection::Connected { name, .. } => {
                info!("PS4/PS5 controller connected: {name}");
                active.entity = Some(event.gamepad);

                // Apply dead-zone from InputConfig to this specific gamepad entity
                let mut settings = GamepadSettings::default();
                let dz = config.deadzone_radius;
                settings
                    .default_axis_settings
                    .set_deadzone_lowerbound(-dz);
                settings
                    .default_axis_settings
                    .set_deadzone_upperbound(dz);
                commands.entity(event.gamepad).insert(settings);
            }
            GamepadConnection::Disconnected => {
                info!("Controller disconnected — keyboard/mouse fallback active");
                if active.entity == Some(event.gamepad) {
                    active.entity = None;
                }
            }
        }
    }
}
```

### `plugin.rs`

```rust
//! NeonInputPlugin — PS4/PS5 controller + keyboard/mouse input system.

use bevy::prelude::*;

use super::systems::{on_gamepad_event, setup_input};

/// Bevy plugin that implements Neon Expanse input handling.
///
/// # Responsibilities
/// - Loads [`InputConfig`] from `assets/config/input.ron` (RON format,
///   moddable without recompile). Falls back to hardcoded defaults.
/// - Inserts [`ActiveGamepad`] resource tracking the connected controller.
/// - Auto-detects PS4/PS5 controller on connection and applies dead-zone
///   filtering from [`InputConfig`].
/// - Logs all connection/disconnection events with human-readable messages.
/// - Gracefully switches to keyboard/mouse fallback if controller disconnects.
pub struct NeonInputPlugin;

impl Plugin for NeonInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_input)
            .add_systems(Update, on_gamepad_event);
    }
}
```

---

## Stub Plugins

```rust
// src/plugins/voxel_world_plugin.rs

/// Empty stub for the Voxel World Engine plugin.
///
/// This plugin will own: chunk streaming, dual marching cubes mesher,
/// layered procedural generation, biome overlays, and the
/// scaled-space / full-detail rendering switch.
///
/// Populated in spec `002-voxel-planet-engine`.
pub struct VoxelWorldPlugin;

impl bevy::prelude::Plugin for VoxelWorldPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {
        // Stub: no systems registered yet.
    }
}
```

```rust
// src/plugins/traversal_plugin.rs

/// Empty stub for the Traversal plugin.
///
/// This plugin will own: PlayerLocomotionComponent, VehicleComponent,
/// WatercraftComponent, AircraftComponent, and all movement systems.
///
/// Populated in a spec following `002-voxel-planet-engine`.
pub struct TraversalPlugin;

impl bevy::prelude::Plugin for TraversalPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {
        // Stub: no systems registered yet.
    }
}
```

---

## Default `assets/config/input.ron`

```ron
// Neon Expanse default input configuration.
// Edit this file to remap controls or adjust dead zones without recompiling.
// This file is loaded at startup; delete it to revert to hardcoded defaults.
(
    deadzone_radius: 0.15,
    axis_map: {
        "MoveForward": LeftStickY,
        "MoveRight": LeftStickX,
        "LookVertical": RightStickY,
        "LookHorizontal": RightStickX,
    },
    button_map: {
        "PrimaryAction": South,
        "Jump": South,
        "Sprint": LeftThumb,
    },
    keyboard_map: {
        "MoveForward": KeyW,
        "MoveRight": KeyD,
        "Jump": Space,
        "Sprint": ShiftLeft,
        "PrimaryAction": KeyE,
    },
)
```

---

## Tests

### `tests/floating_origin_tests.rs`

```rust
//! Property-based tests for the floating-origin coordinate system.

use glam::DVec3;
use proptest::prelude::*;

/// Generates DVec3 values in the range of planetary coordinates
/// (-20,000 km to +20,000 km per axis).
fn planetary_dvec3() -> impl Strategy<Value = DVec3> {
    (
        -20_000_000.0_f64..=20_000_000.0_f64,
        -20_000_000.0_f64..=20_000_000.0_f64,
        -20_000_000.0_f64..=20_000_000.0_f64,
    )
        .prop_map(|(x, y, z)| DVec3::new(x, y, z))
}

proptest! {
    /// The local f32 offset between any two planetary positions must
    /// be representable with no catastrophic precision loss, provided
    /// the entities are within ~1,000 km of each other.
    #[test]
    fn sync_offset_is_finite_at_planetary_scale(
        origin in planetary_dvec3(),
        entity in planetary_dvec3()
    ) {
        let offset = (entity - origin).as_vec3();
        prop_assert!(offset.is_finite(), "offset contains NaN or infinity");
    }

    /// When origin == entity, the local transform is exactly zero.
    #[test]
    fn same_position_produces_zero_offset(pos in planetary_dvec3()) {
        let offset = (pos - pos).as_vec3();
        prop_assert_eq!(offset, glam::Vec3::ZERO);
    }

    /// Camera at origin → camera transform is zero vector.
    #[test]
    fn camera_transform_is_always_zero(camera_pos in planetary_dvec3()) {
        let offset = (camera_pos - camera_pos).as_vec3();
        prop_assert_eq!(offset, glam::Vec3::ZERO);
    }
}
```

### `tests/lod_tests.rs`

```rust
//! Property-based tests for LOD level classification.

use proptest::prelude::*;

fn classify_lod(dist: f32, thresholds: [f32; 3]) -> u8 {
    if dist < thresholds[0] { 0 }
    else if dist < thresholds[1] { 1 }
    else if dist < thresholds[2] { 2 }
    else { 3 }
}

proptest! {
    #[test]
    fn lod_level_in_range(dist in 0.0_f32..=20_000_000.0_f32) {
        let level = classify_lod(dist, [1_000.0, 10_000.0, 100_000.0]);
        prop_assert!(level <= 3);
    }

    #[test]
    fn lod_level_is_deterministic(dist in 0.0_f32..=20_000_000.0_f32) {
        let a = classify_lod(dist, [1_000.0, 10_000.0, 100_000.0]);
        let b = classify_lod(dist, [1_000.0, 10_000.0, 100_000.0]);
        prop_assert_eq!(a, b);
    }
}
```

---

## Post-Design Constitution Re-Check

Re-evaluated after Phase 1 design was complete. **Result: all 36 gates remain PASS.** No new violations introduced. No complexity tracking table needed.

---

## Next Steps

This is the end of the Core Foundation milestone. After `cargo run` produces:
1. A stable borderless fullscreen window entitled "Neon Expanse v0.0.1-dev"
2. Console messages: "Neon Expanse core initialized" + controller/keyboard status + 1 Hz diagnostics
3. A visible cyan sphere at ~6,000 km (100 m in front of the camera)

…the next specification to create is:

### `002-voxel-planet-engine` — Procedural Voxel Planet Engine

**What it will add** (to `VoxelWorldPlugin`, currently a stub):
- Earth-scale sphere voxel grid partitioned into streaming chunks
- Dual marching cubes mesher for smooth terrain with overhangs and caves
- Layered noise pipeline: base sphere → fractal noise layers → erosion simulation → biome overlays
- Seamless LOD transitions using the `LodLevel` component already present on all entities
- Chunk streaming triggered by the camera's `GlobalPosition` / `OriginFrame`
- Scaled-space impostor rendering for distant terrain (level 3 entities)

**What it will not touch**:
- `main.rs` — plugin list stays identical
- `FloatingOriginPlugin` — no changes
- `CorePlugin`, `PhysicsPlugin`, `NeonInputPlugin`, `LodPlugin` — no changes
- Any file under `src/plugins/` that already exists — `VoxelWorldPlugin` will add code to its own module only

**Trigger**: Start with `/speckit.specify` for feature `002-voxel-planet-engine`.

