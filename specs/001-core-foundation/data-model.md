# Data Model: Core Foundation

**Branch**: `001-core-foundation` | **Date**: 2026-02-20

---

## Components

### `GlobalPosition(DVec3)`
**Plugin**: `FloatingOriginPlugin`
**Module**: `src/plugins/floating_origin/components.rs`

Carries the authoritative world-space position of any entity that participates in the floating-origin coordinate system. Values are in meters; coordinate origin is the planet's geometric centre (or first spawn point — seeded consistently).

| Field | Type | Description |
|-------|------|-------------|
| `0` | `DVec3` | Global position in metres, f64 precision |

**Usage**: Stamped on every entity that must exist at planetary distances: the test origin marker, future terrain chunks, vehicles, the player. The `Transform.translation` of these entities is derived from this component each frame — it MUST NOT be set manually outside `FloatingOriginPlugin`.

**Validation rules**:
- Must not contain NaN or infinity.
- Magnitude must not exceed the planet's half-circumference (~20,015 km = 2.0015 × 10⁷ m) during normal operation.

---

### `FloatingOrigin`
**Plugin**: `FloatingOriginPlugin`
**Module**: `src/plugins/floating_origin/components.rs`

Zero-sized marker component. The single entity carrying this marker is the **reference viewer** — the entity whose `GlobalPosition` defines the current floating-origin frame. Exactly one entity may carry this at any time.

During Core Foundation: the camera entity.
Future: may transfer to the active player entity.

---

### `LodLevel(u8)`
**Plugin**: `LodPlugin`
**Module**: `src/plugins/lod/components.rs`

Stores the currently assigned LOD level for an entity. Updated every frame by the LOD system based on distance from the `FloatingOrigin` entity.

| Value | Distance range | Intended use |
|-------|---------------|--------------|
| `0` | < 1,000 m | High detail |
| `1` | 1,000 – 10,000 m | Medium detail |
| `2` | 10,000 – 100,000 m | Low detail |
| `3` | > 100,000 m | Impostor |

Not observable during Core Foundation (no geometry). Component is present on the test marker so future geometry systems can query it without modification.

---

### `TestOriginMarker`
**Plugin**: `FloatingOriginPlugin` startup system
**Module**: `src/plugins/floating_origin/systems.rs`

Zero-sized marker component. Identifies the single diagnostic entity spawned at planetary distance to prove floating-origin correctness. Used by tests and diagnostic queries to locate the entity by type rather than by stored `Entity` handle.

---

## Resources

### `WorldSeed`
**Plugin**: `CorePlugin`
**Module**: `src/plugins/core_plugin.rs`

```
pub struct WorldSeed(pub u64);
pub const WORLD_SEED: u64 = 0x4E45_4F4E_3230_3236; // ASCII: "NEON2026"
```

Deterministic world seed. Every procedural generation system MUST consume this resource rather than seeding its own RNG. Identical seeds produce identical planets.

**Invariant**: Value is a compile-time constant; must never change after the core-foundation milestone.

---

### `OriginFrame`
**Plugin**: `FloatingOriginPlugin`
**Module**: `src/plugins/floating_origin/plugin.rs`

```
pub struct OriginFrame {
    pub position: DVec3,   // current reference origin in world space
}
```

Updated every `PreUpdate` frame to match the `GlobalPosition` of the entity carrying `FloatingOrigin`. All `Transform` sync computations in `PostUpdate` read this resource.

**Invariant**: Always equal to the `GlobalPosition` of the `FloatingOrigin` entity as of the start of the current frame.

---

### `InputConfig`
**Plugin**: `InputPlugin`
**Module**: `src/plugins/input/config.rs`

Loaded from `assets/config/input.ron` at startup; falls back to `InputConfig::default()` if file is absent.

```
pub struct InputConfig {
    pub deadzone_radius: f32,                                // default: 0.15
    pub axis_map: HashMap<String, GamepadAxis>,             // action name → axis
    pub button_map: HashMap<String, GamepadButton>,         // action name → button
    pub keyboard_map: HashMap<String, KeyCode>,             // action name → key fallback
}
```

**Serde derives**: `serde::Deserialize`, `serde::Serialize`. RON format.

---

### `ActiveGamepad`
**Plugin**: `InputPlugin`
**Module**: `src/plugins/input/plugin.rs`

```
pub struct ActiveGamepad {
    pub entity: Option<Entity>,  // None = keyboard/mouse fallback active
}
```

Set when a gamepad connection event fires. The `InputPlugin` detects if the connected device is a PS4/PS5 controller by checking `GamepadConnection::Connected { name, .. }` — the device name contains "DualShock", "DualSense", or "PS4"/"PS5".

---

### `LodSettings`
**Plugin**: `LodPlugin`
**Module**: `src/plugins/lod/plugin.rs`

```
pub struct LodSettings {
    pub thresholds: [f32; 3],   // distances in metres at which to transition
                                // index 0 = High→Medium, 1 = Medium→Low, 2 = Low→Impostor
                                // defaults: [1_000.0, 10_000.0, 100_000.0]
}
```

---

## State Transitions

### Controller Hot-Plug FSM

```
        ┌─────────────────────────────────────────────────────┐
        │                   at startup                        │
        ▼                                                     │
  ┌───────────┐  GamepadConnectionEvent    ┌───────────────┐  │
  │  Keyboard │ ─────(Connected)─────────► │ PS4/PS5 Active│  │
  │  Fallback │ ◄────(Disconnected)──────── └───────────────┘  │
  └───────────┘                                                │
        └───────────────────(startup, no gamepad)─────────────┘
```

Transition fires a `console info!()` message in both directions (FR-014, FR-015, FR-018).

---

## Entity Archetypes

### Camera Entity (spawned by `FloatingOriginPlugin` startup)
```
Camera3d
Transform                // always Vec3::ZERO after first sync frame
GlobalTransform          // auto-inserted by Camera3d required components
GlobalPosition(DVec3)    // ~(6_000_100.0, 0.0, 0.0)
FloatingOrigin           // marks this as the reference viewer
```

### Test Origin Marker (spawned by `FloatingOriginPlugin` startup)
```
Mesh3d(Handle<Mesh>)                        // Sphere with radius 50 m
MeshMaterial3d(Handle<StandardMaterial>)    // emissive cyan
Transform                                   // recomputed each frame: (-100, 0, 0) relative to camera
GlobalTransform                             // propagated from Transform
GlobalPosition(DVec3)                       // (6_000_000.0, 0.0, 0.0)
RigidBody::Static                           // avian3d: immune to gravity
LodLevel(3)                                 // starts at impostor; no impostor rendering yet
TestOriginMarker                            // zero-size identifier
```

### Point Light (spawned alongside test marker for visibility)
```
PointLight { intensity: 150_000.0, range: 500.0, ..default() }
Transform    // (-150.0, 100.0, 0.0) relative to camera — near the sphere
GlobalPosition(DVec3)  // (5_999_950.0, 100.0, 0.0)
```
