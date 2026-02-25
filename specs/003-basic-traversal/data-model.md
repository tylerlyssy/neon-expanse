# Data Model: Basic Traversal System (Feature 003)

**Date**: 2026-02-24 | **Branch**: `003-basic-traversal`

---

## 1. Components

### `PlayerLocomotionState`
```rust
#[derive(Component, Default, Reflect)]
pub struct PlayerLocomotionState {
    /// Accumulated vertical velocity (m/s) — gravity added per tick, zeroed on ground.
    pub vertical_velocity: f32,
    /// True when the slope angle under the player is walkable.
    pub is_grounded: bool,
    /// True for one FixedUpdate tick after jumping.
    pub jumped_this_frame: bool,
    /// True when capsule is more than 50% submerged below sea_level_m. Replaces walking
    /// controls with swimming controls and enables buoyancy. Covers FR-004.
    pub is_swimming: bool,
}
```

### `OnFootInputState`
```rust
#[derive(Component, Default, Reflect)]
pub struct OnFootInputState {
    /// Normalized movement direction in XZ plane (world space).
    pub move_dir: Vec2,
    /// Normalized look input (h, v) from right stick / mouse.
    pub look_delta: Vec2,
    /// Jump requested this frame.
    pub jump_pressed: bool,
    /// Sprint held.
    pub sprint_held: bool,
    /// Enter/exit vehicle requested.
    pub enter_exit_pressed: bool,
}
```

### `VehicleInputState`
```rust
#[derive(Component, Default, Reflect)]
pub struct VehicleInputState {
    /// Throttle [0.0, 1.0].
    pub throttle: f32,
    /// Brake [0.0, 1.0].
    pub brake: f32,
    /// Steering [-1.0, 1.0] (left stick X for ground/water; rudder yaw for aircraft).
    pub steer: f32,
    /// Pitch input [-1.0, 1.0] (right stick Y for aircraft).
    pub pitch: f32,
    /// Roll input [-1.0, 1.0] (right stick X for aircraft).
    pub roll: f32,
    /// Enter/exit requested this frame.
    pub enter_exit_pressed: bool,
}
```

### `VehicleTag`
```rust
#[derive(Component, Reflect)]
pub enum VehicleTag {
    GroundVehicle,
    Watercraft,
    Helicopter,
    FixedWing,
}
```

### `PassengerOf`
```rust
/// Added to the player entity while they occupy a vehicle.
#[derive(Component, Reflect)]
pub struct PassengerOf(pub Entity);
```

### `OccupiedBy`
```rust
/// Added to a vehicle entity while a player is inside.
#[derive(Component, Reflect)]
pub struct OccupiedBy(pub Entity);
```

### `WheelConfig`
```rust
/// Per-wheel data, stored as a child component or in-array on VehiclePhysicsConfig.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct WheelConfig {
    /// Offset from vehicle body origin (local space, metres).
    pub local_offset: Vec3,
    /// Length of suspension ray (metres).
    pub ray_length: f32,
    /// Natural suspension length (metres).
    pub rest_length: f32,
    /// Spring stiffness (N/m).
    pub spring_stiffness: f32,
    /// Damper coefficient (N·s/m).
    pub damper: f32,
}
```

### `EnterExitProximity`
```rust
/// Added to vehicle entities; tracks whether a player is ≤ interact_radius metres away.
#[derive(Component, Default, Reflect)]
pub struct EnterExitProximity {
    pub nearby_players: SmallVec<[Entity; 2]>,
}
```

---

## 2. Resources

### `CurrentVehicle`
```rust
#[derive(Resource, Default, Reflect)]
pub struct CurrentVehicle(pub Option<Entity>);
```

### `PlayerLocomotionConfig`
Loaded from `assets/player/locomotion.ron`:
```rust
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct PlayerLocomotionConfig {
    pub walk_speed_m_s: f32,          // 5.0
    pub sprint_speed_m_s: f32,        // 10.0
    pub jump_impulse_m_s: f32,        // 6.0
    pub gravity_m_s2: f32,            // -20.0
    pub max_slope_angle_deg: f32,     // 45.0
    pub interact_radius_m: f32,       // 3.0
    pub capsule_radius_m: f32,        // 0.35
    pub capsule_height_m: f32,        // 1.8
    /// Maximum horizontal speed while swimming (m/s). Covers FR-004 and FR-005.
    pub swim_speed_m_s: f32,          // 3.0
    /// Y-axis height of the flat ocean surface used for swimming/buoyancy detection.
    pub sea_level_m: f32,             // 0.0
}
```

### `VehicleRegistry`
```rust
/// Loaded at startup from assets/vehicles/. Maps config filename → deserialized VehicleConfig.
#[derive(Resource, Default)]
pub struct VehicleRegistry(pub Vec<VehicleConfig>);
```

### `WorldSpawnList`
```rust
/// Loaded from assets/vehicles/world_spawns.ron.
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct WorldSpawnList {
    pub entries: Vec<VehicleSpawnEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSpawnEntry {
    /// Filename relative to assets/vehicles/, e.g. "car.ron".
    pub config: String,
    /// World position (f64 for floating-origin precision).
    pub position: (f64, f64, f64),
    /// Optional Euler rotation in degrees (yaw only for ground vehicles).
    pub yaw_deg: Option<f32>,
}
```

---

## 3. Config Structs (RON)

### `VehicleConfig` (sum type)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct VehicleConfig {
    pub name: String,
    pub physics: VehiclePhysicsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum VehiclePhysicsConfig {
    GroundVehicle(GroundVehicleConfig),
    Watercraft(WatercraftConfig),
    Helicopter(HelicopterConfig),
    FixedWing(FixedWingConfig),
}
```

### `GroundVehicleConfig`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct GroundVehicleConfig {
    pub mass_kg: f32,
    pub max_speed_m_s: f32,
    pub engine_force_n: f32,
    pub brake_force_n: f32,
    pub max_steer_angle_deg: f32,
    pub wheels: Vec<WheelConfig>,   // exactly 4 for MVP
    /// Velocity-delta threshold (m/s) above which an ImpactEvent is emitted. Covers FR-011.
    pub impact_threshold_m_s: f32,  // 20.0
}
```

### `WatercraftConfig`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct WatercraftConfig {
    pub mass_kg: f32,
    pub max_speed_m_s: f32,
    pub thrust_force_n: f32,
    pub turn_torque_n_m: f32,
    pub buoyancy_volume_m3: f32,    // approximate submerged hull volume
    pub water_drag: f32,            // linear drag coefficient
    pub sea_level_m: f32,           // y-axis height (flat ocean MVP)
}
```

### `HelicopterConfig`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct HelicopterConfig {
    pub mass_kg: f32,
    pub max_ascent_m_s: f32,
    pub max_lateral_m_s: f32,
    pub hover_throttle: f32,        // throttle fraction required to hover (0.5 typical)
    pub tilt_sensitivity: f32,      // degrees of tilt per unit stick input
    pub yaw_torque_n_m: f32,
    /// Velocity-delta threshold (m/s) above which an ImpactEvent is emitted. Covers FR-019.
    pub impact_threshold_m_s: f32,  // 15.0
}
```

### `FixedWingConfig`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct FixedWingConfig {
    pub mass_kg: f32,
    pub max_thrust_n: f32,
    pub lift_coefficient: f32,
    pub drag_coefficient: f32,
    pub wing_area_m2: f32,
    pub stall_speed_m_s: f32,
    pub max_pitch_torque_n_m: f32,
    pub max_roll_torque_n_m: f32,
    pub max_yaw_torque_n_m: f32,
    /// Velocity-delta threshold (m/s) above which an ImpactEvent is emitted. Covers FR-019.
    pub impact_threshold_m_s: f32,  // 25.0
}
```

---

## 4. Events

```rust
#[derive(Event, Debug)]
pub struct ImpactEvent {
    /// The vehicle entity that impacted.
    pub entity: Entity,
    /// Speed at impact (m/s).
    pub impact_speed_m_s: f32,
    /// Contact normal in world space (outward from the collider surface hit). Covers FR-012.
    pub contact_normal: Vec3,
}

#[derive(Event, Debug)]
pub struct EnterVehicleEvent {
    pub player: Entity,
    pub vehicle: Entity,
}

#[derive(Event, Debug)]
pub struct ExitVehicleEvent {
    pub player: Entity,
    pub vehicle: Entity,
}
```

---

## 5. Entity Bundles

### `PlayerBundle`
```rust
#[derive(Bundle)]
pub struct PlayerBundle {
    pub tag: PlayerTag,
    pub state: PlayerLocomotionState,
    pub input: OnFootInputState,
    pub global_pos: GlobalPosition,
    pub collider: Collider,
    pub rigid_body: RigidBody,           // Kinematic
    pub gravity_scale: GravityScale,     // 0.0 — manual gravity
    pub locked_axes: LockedAxes,         // lock rotation
    pub visibility: Visibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
```

### `GroundVehicleBundle`
```rust
#[derive(Bundle)]
pub struct GroundVehicleBundle {
    pub tag: VehicleTag,   // VehicleTag::GroundVehicle
    pub config: GroundVehicleConfig,
    pub input: VehicleInputState,
    pub proximity: EnterExitProximity,
    pub global_pos: GlobalPosition,
    pub collider: Collider,              // box or compound
    pub rigid_body: RigidBody,           // Dynamic
    pub mass: Mass,
    pub linear_velocity: LinearVelocity,
    pub visibility: Visibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
```

---

## 6. State Transitions

```
Player state machine (per-entity):
  OnFoot  ──[EnterVehicleEvent]──►  InVehicle
  InVehicle ──[ExitVehicleEvent]──► OnFoot

VehicleOccupancy:
  Vacant ──[EnterVehicleEvent]──► Occupied(player_entity)
  Occupied ──[ExitVehicleEvent]──► Vacant
```

Player is "InVehicle" when `PassengerOf` component exists.  
Systems gate on `With<PassengerOf>` vs `Without<PassengerOf>`.

---

## 7. Validation Rules

| Rule | Source |
|---|---|
| Vehicle mass must be > 0.0 | Deserialization guard |
| Wheel count must be 1..=8 | Warn if 0 |
| stall_speed_m_s > 0.0 | Assert |
| hover_throttle in (0.0, 1.0) | Clamp |
| interact_radius_m > 0.0 | Minimum 0.5 |
