# Research: Core Foundation

**Branch**: `001-core-foundation` | **Date**: 2026-02-20
**Phase**: 0 — Pre-design research

---

## Decision 1: Floating-Origin Architecture

**Decision**: Custom `FloatingOriginPlugin` using `glam::DVec3` per-entity global position component + `OriginFrame` resource, **not** the `big_space` crate.

**Rationale**:
- The Constitution (§III) explicitly mandates `glam::DVec3` for global positions — this is the public API contract for every future plugin.
- `big_space` uses an integer grid-cell + f32 offset approach (superior precision for galaxy-scale, but a different data model than what the Constitution specifies).
- A custom plugin keeps the crate graph minimal and makes the global-position contract crystal-clear for all contributors and future plugins.
- The DVec3 approach is entirely sufficient for a 12,742 km planet: maximum distance from origin = ~6,371 km = 6.371 × 10⁶ m. f64 precision at this scale is ~10⁻¹⁰ m — far better than any physical rendering requirement.

**Implementation pattern** (from community research):
1. Each world entity carries `GlobalPosition(DVec3)`.
2. One entity carries `FloatingOrigin` marker (always the camera).
3. An `OriginFrame` resource holds the current reference `DVec3` (updated to match the `FloatingOrigin` entity's `GlobalPosition` every frame).
4. A sync system in `PostUpdate`, ordered `.before(TransformSystem::TransformPropagate)`, computes `Transform.translation = (entity_global - origin).as_vec3()` for every entity with `GlobalPosition`.
5. The camera's own `Transform.translation` becomes `Vec3::ZERO` each frame.

**Alternatives considered**:
- `big_space` crate (aevyrie, v0.12 for Bevy 0.18) — better maximum precision (i128 grid) but different data model; rejected because Constitution locks the DVec3 contract.
- Avian3d f64 mode — physics coordinate space is local; f64 avian is only needed if physics simulates at planetary distances, which is not the case here (physics always operates in local f32 space near the floating origin).

---

## Decision 2: Avian3d Setup

**Decision**: `avian3d = "0.5"` with default features (f32 physics).

**Confirmed API facts**:
- Plugin name: `PhysicsPlugins::default()` (a plugin group, not a single plugin).
- Static rigid body: `RigidBody::Static` — immune to gravity and all forces.
- Timestep: physics runs in Bevy's `FixedPostUpdate` schedule and inherits `Time::<Fixed>`. Setting `Time::<Fixed>::from_hz(64.0)` automatically controls both game logic (`FixedUpdate`) and physics (`FixedPostUpdate`) at 64 Hz — no separate avian configuration required.
- Gravity: on by default (`Gravity(Vec3::NEG_Y * 9.81)`). `RigidBody::Static` entities are unaffected.
- `PhysicsSchedule` is a nested sub-schedule inside `FixedPostUpdate`; user game logic goes in `FixedUpdate`, physics simulation is in the nested schedule — both tick at 64 Hz from the same `Time::<Fixed>` accumulator.
- avian3d 0.5 targets Bevy 0.18 exactly.

**f32 vs f64 physics**:
- Physics bodies always exist in local f32 space near the floating origin. Global planetary coordinates are only the `GlobalPosition(DVec3)` component, never fed into avian3d directly.
- f32 avian3d is correct for this project. The f64 avian feature would only be needed if rigid body simulations spanned multi-kilometre distances without a floating origin — not the case here.

**Test entity physics**: Spawned with `RigidBody::Static` and no `Collider`. Gravity has no effect. Zero performance cost.

---

## Decision 3: Bevy 0.18 API Surface (Breaking Changes from < 0.15)

**All `*Bundle` types removed** (since 0.15). Required-component protocol replaces them:
- `Camera3dBundle` → `Camera3d::default()` (auto-inserts `Camera`, `Projection`, `GlobalTransform`, `VisibilityClass`, etc.)
- `PbrBundle` → `Mesh3d(handle)` + `MeshMaterial3d(mat_handle)` + `Transform`
- `PointLightBundle` → `PointLight { .. }` + `Transform`

**Window setup** (`bevy::window`):
```
WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
PresentMode::AutoNoVsync
Window { title, mode, present_mode, ..default() }
DefaultPlugins.set(WindowPlugin { primary_window: Some(..), ..default() })
```

**Fixed timestep**:
```
Time::<Fixed>::from_hz(64.0)   // insert as Resource
FixedUpdate schedule            // game logic
FixedPostUpdate schedule        // physics (avian3d)
```

**Diagnostics** (`bevy::diagnostic`):
```
FrameTimeDiagnosticsPlugin
EntityCountDiagnosticsPlugin
LogDiagnosticsPlugin { wait_duration: Duration::from_secs(1), filter: None, ..default() }
```

**Gamepad (since 0.15)**: Gamepads are now ECS entities with a `Gamepad` component. The old `Gamepads` resource is gone.
```
GamepadConnectionEvent { gamepad: Entity, connection: GamepadConnection }
GamepadConnection::Connected { name, vendor_id, product_id }
GamepadConnection::Disconnected
Query<(Entity, &Gamepad)>  // to read axis values
gamepad.get(GamepadAxis::LeftStickX) -> Option<f32>
gamepad.left_stick() -> Vec2
GamepadSettings (Component on gamepad entity) — set dead zones per-gamepad
```

**Transform propagation ordering**:
```
TransformSystem::TransformPropagate  // system set in PostUpdate (bevy::transform::TransformSystem)
// Our sync system: .before(TransformSystem::TransformPropagate)
```

---

## Decision 4: Input Configuration via RON

**Decision**: `assets/config/input.ron` loaded at startup via Bevy's asset system (or direct `std::fs::read_to_string` for simplicity at this stage — the asset system adds async complexity for a config file that must be ready before the first frame).

**Pattern**: Startup system reads the file synchronously, parses it into `InputConfig` resource, or falls back to `InputConfig::default()` if absent. This matches the spec's "hardcoded defaults if file absent" requirement without any async asset loading.

**`InputConfig` structure** (stored in the resource):
- `deadzone_radius: f32` — dead zone for all analog axes (default: 0.15)
- `axis_map: HashMap<ActionAxis, GamepadAxis>` — maps named axis actions to gamepad axes
- `button_map: HashMap<ActionButton, GamepadButton>` — maps named actions to gamepad buttons
- `keyboard_map: HashMap<ActionButton, KeyCode>` — keyboard fallback per action

**RON format** (human-readable, moddable):
```ron
InputConfig(
    deadzone_radius: 0.15,
    axis_map: {
        "MoveForward": LeftStickY,
        "MoveRight": LeftStickX,
        "LookVertical": RightStickY,
        "LookHorizontal": RightStickX,
    },
    button_map: {
        "PrimaryAction": South,
        "SecondaryAction": West,
        "Jump": South,
        "Sprint": LeftThumb,
    },
    keyboard_map: {
        "MoveForward": W,
        "MoveRight": D,
        "Jump": Space,
        "Sprint": ShiftLeft,
        "PrimaryAction": E,
    },
)
```

---

## Decision 5: LOD Plugin Design

**Decision**: `LodPlugin` is operational but produces no visible transitions without geometry. It registers a `LodSettings` resource with configurable distance thresholds and a `LodLevel` component stamped on entities.

**Thresholds** (in meters, initial defaults):
| Level | Distance from camera |
|-------|----------------------|
| High (0) | < 1,000 m |
| Medium (1) | 1,000 – 10,000 m |
| Low (2) | 10,000 – 100,000 m |
| Impostor (3) | > 100,000 m |

These values are calibrated for a 12,742 km planet with a player near the surface. They will be refined in the Voxel World Engine spec.

---

## Decision 6: Profiling Hooks

**Decision**: Use Bevy's built-in `TracyPlugin` (feature `bevy/trace_tracy`) as an optional compile-time feature. `puffin` is not added in this feature — it requires `bevy_egui` which is out of scope until a UI spec is written.

**Cargo feature gate**:
```toml
[features]
tracy = ["bevy/trace_tracy"]
```
Run with `cargo run --features tracy` and attach Tracy profiler externally. Default builds have zero profiling overhead.

Bevy's `LogPlugin` (included in `DefaultPlugins`) handles structured logging (env filter via `RUST_LOG`).
