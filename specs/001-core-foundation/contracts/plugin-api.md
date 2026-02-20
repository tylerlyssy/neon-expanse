# Plugin API Contracts: Core Foundation

**Branch**: `001-core-foundation` | **Date**: 2026-02-20
**Scope**: Public surface of each plugin — what they provide and what they require.

All plugins follow the invariant: **no plugin directly calls into another plugin's internal systems**. Cross-plugin communication is exclusively through Bevy's ECS (shared resources, events, components, and query results).

---

## `CorePlugin`
**Module**: `src/plugins/core_plugin.rs`

### Provides
| Item | Kind | Description |
|------|------|-------------|
| `FixedUpdate` schedule at 64 Hz | Schedule configuration | `Time::<Fixed>::from_hz(64.0)` inserted as resource |
| `LogPlugin` | Bevy built-in | Structured logging; respects `RUST_LOG` env var, default level `info` |
| `FrameTimeDiagnosticsPlugin` | Bevy built-in | Registers FPS + frame-time diagnostic paths |
| `EntityCountDiagnosticsPlugin` | Bevy built-in | Registers entity count diagnostic path |
| `LogDiagnosticsPlugin` | Bevy built-in | Emits all diagnostics to log at 1 Hz |

### Requires
- Must be registered **before** all other plugins (provides the baseline schedules and logging).

### API
None — `CorePlugin` has no public components, resources, or events. It is purely configurational.

---

## `NeonWindowPlugin`
**Module**: `src/plugins/window_plugin.rs`

### Provides
| Item | Kind | Description |
|------|------|-------------|
| `WindowMode::BorderlessFullscreen(MonitorSelection::Primary)` | Window config | Set via `DefaultPlugins.set(WindowPlugin { .. })` |
| `PresentMode::AutoNoVsync` | Window config | No v-sync tearing prevention |
| Title `"Neon Expanse v0.0.1-dev"` | Window config | |

### Requires
- Must be registered via `DefaultPlugins.set(WindowPlugin { .. })` inside `App::new().add_plugins(...)`.
- `CorePlugin` must be registered first (provides the app schedule).

### API
None public.

---

## `NeonInputPlugin`
**Module**: `src/plugins/input/`

### Provides
| Item | Kind | Description |
|------|------|-------------|
| `InputConfig` | `Resource` | Loaded from `assets/config/input.ron`; falls back to defaults |
| `ActiveGamepad` | `Resource` | Tracks currently active gamepad entity or `None` |
| Console message on connect | Side effect | `info!("PS4/PS5 controller connected: {name}")` |
| Console message on disconnect | Side effect | `info!("Controller disconnected — keyboard/mouse fallback active")` |
| Console message on keyboard fallback | Side effect | `info!("No controller detected — keyboard/mouse fallback active")` |
| Dead-zone configuration | Behavior | Inserts `GamepadSettings` with `deadzone_radius` from `InputConfig` on every new gamepad entity |

### Requires
- Bevy's `DefaultPlugins` (provides gamepad backend and `GamepadConnectionEvent`).
- `CorePlugin` (schedules).

### Events Consumed
| Event | Source |
|-------|--------|
| `GamepadConnectionEvent` | Bevy input system |

### API
```rust
// Resource — read by any system needing to query input state
pub struct InputConfig { .. }

// Resource — read to know if a controller is active
pub struct ActiveGamepad { pub entity: Option<Entity> }
```

---

## `FloatingOriginPlugin`
**Module**: `src/plugins/floating_origin/`

### Provides
| Item | Kind | Description |
|------|------|-------------|
| `GlobalPosition(DVec3)` | `Component` | Authoritative world-space position for any entity |
| `FloatingOrigin` | `Component` (ZST) | Marks the reference-viewer entity (camera) |
| `OriginFrame` | `Resource` | Current reference DVec3, updated each `PreUpdate` |
| `TestOriginMarker` | `Component` (ZST) | Tags the diagnostic sphere for test queries |
| Transform sync system | `PostUpdate` | Updates `Transform.translation` for all `GlobalPosition` entities before propagation |
| Startup spawn | Startup system | Spawns camera + test sphere + light at planetary distance |
| Console message | Side effect | `info!("Neon Expanse core initialized")` on first frame |

### Requires
- `CorePlugin` (schedules, fixed timestep).
- `PhysicsPlugin` must be registered before startup systems run (for `RigidBody::Static` on test entity).
- `bevy::asset` (mesh + material handles for test sphere).

### System Ordering
```
PreUpdate:  update_origin_frame  (reads FloatingOrigin entity's GlobalPosition → writes OriginFrame)
PostUpdate: sync_transforms      (before TransformSystem::TransformPropagate)
```

### API
```rust
// Stamp on any entity that should be tracked in world space
#[derive(Component)]
pub struct GlobalPosition(pub DVec3);

// Stamp on exactly one entity (the camera / active viewer)
#[derive(Component)]
pub struct FloatingOrigin;

// Current reference frame — read-only for all external systems
#[derive(Resource)]
pub struct OriginFrame { pub position: DVec3 }
```

**Invariant**: Any system that moves an entity MUST update `GlobalPosition`, not `Transform.translation` directly. `Transform` is managed exclusively by `FloatingOriginPlugin`.

---

## `LodPlugin`
**Module**: `src/plugins/lod/`

### Provides
| Item | Kind | Description |
|------|------|-------------|
| `LodSettings` | `Resource` | Three distance thresholds (metres); `[1_000.0, 10_000.0, 100_000.0]` default |
| `LodLevel(u8)` | `Component` | Current computed LOD level for each entity |
| LOD update system | `PostUpdate` | Computes `LodLevel` for all entities with `GlobalPosition`; runs after `sync_transforms` |

### Requires
- `FloatingOriginPlugin` (reads `OriginFrame` and `GlobalPosition`).

### System Ordering
```
PostUpdate: update_lod_levels  (after sync_transforms, before or after TransformSystem::TransformPropagate)
```

### API
```rust
#[derive(Resource)]
pub struct LodSettings { pub thresholds: [f32; 3] }

#[derive(Component)]
pub struct LodLevel(pub u8);
```

---

## `PhysicsPlugin`
**Module**: `src/plugins/physics_plugin.rs`

### Provides
| Item | Kind | Description |
|------|------|-------------|
| `PhysicsPlugins::default()` | Bevy plugin group | Full avian3d physics: broadphase, narrowphase, solver, `FixedPostUpdate` schedule |
| `RigidBody` | `Component` (re-export) | `avian3d::prelude::RigidBody` (Static / Dynamic / Kinematic) |
| `Gravity` resource | Active default | `Vec3::NEG_Y * 9.81 m/s²` |

### Requires
- `CorePlugin` (for `Time::<Fixed>` at 64 Hz, which physics inherits).

### API
Re-exports `avian3d::prelude::*` for convenience. All avian3d APIs are available to any plugin that adds a `use avian3d::prelude::*;`.

---

## `VoxelWorldPlugin` (Stub)
**Module**: `src/plugins/voxel_world_plugin.rs`

### Provides
Nothing — empty stub. Registers successfully and produces no systems.

### Requires
- No hard requirements at this stage.

### Future contract (next spec)
This plugin will own: chunk streaming, dual marching cubes mesher, procedural generation pipeline, biome overlays.

---

## `TraversalPlugin` (Stub)
**Module**: `src/plugins/traversal_plugin.rs`

### Provides
Nothing — empty stub. Registers successfully and produces no systems.

### Requires
- No hard requirements at this stage.

### Future contract (next spec after voxel world)
This plugin will own: `PlayerLocomotionComponent`, `VehicleComponent`, `WatercraftComponent`, `AircraftComponent`, all movement and input-consumption systems.
