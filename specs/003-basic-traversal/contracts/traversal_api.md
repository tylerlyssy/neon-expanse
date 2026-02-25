# System Contracts: Basic Traversal (Feature 003)

---

## Contract Format

Each contract lists:
- **Reads** — components/resources queried (immutable)
- **Writes** — components/resources mutated
- **Events in/out** — events consumed or emitted
- **Schedule** — which Bevy schedule + ordering label
- **Invariants** — guarantees callers may rely on

---

## 1. `poll_on_foot_input`

**Schedule**: `Update`, no ordering constraint  
**Purpose**: Translate raw `ButtonInput<GamepadButton>` + `ButtonInput<KeyCode>` + `Axis<GamepadAxis>` → `OnFootInputState` on each player entity not currently a passenger.

| | |
|---|---|
| **Reads** | `ButtonInput<GamepadButton>`, `ButtonInput<KeyCode>`, `Axis<GamepadAxis>`, `InputConfig` (Resource), `Without<PassengerOf>` |
| **Writes** | `OnFootInputState` (per player entity) |
| **Events in** | — |
| **Events out** | — |

**Invariants**:
- `move_dir` is zero or unit-length (normalized after deadzone filter).
- `jump_pressed` is true for exactly one `Update` frame per button press event.
- `enter_exit_pressed` is true for exactly one frame.

---

## 2. `poll_vehicle_input`

**Schedule**: `Update`, no ordering constraint  
**Purpose**: Same as above for entities in `With<PassengerOf>`.

| | |
|---|---|
| **Reads** | `ButtonInput<GamepadButton>`, `ButtonInput<KeyCode>`, `Axis<GamepadAxis>`, `InputConfig` (Resource), `With<PassengerOf>`, `CurrentVehicle` (Resource) |
| **Writes** | `VehicleInputState` (on the current vehicle entity) |
| **Events in** | — |
| **Events out** | — |

**Invariants**:
- `throttle` and `brake` clamped to [0.0, 1.0].
- `steer`, `pitch`, `roll` clamped to [-1.0, 1.0].

---

## 3. `update_enter_exit_prompts`

**Schedule**: `Update`, after `poll_on_foot_input`  
**Purpose**: For each player with `enter_exit_pressed`, find the nearest vehicle within `PlayerLocomotionConfig::interact_radius_m`; update `EnterExitProximity` component.

| | |
|---|---|
| **Reads** | `GlobalPosition` (all entities), `VehicleTag`, `PlayerLocomotionConfig` (Resource) |
| **Writes** | `EnterExitProximity` (vehicle entities) |
| **Events in** | — |
| **Events out** | — |

**Invariants**:
- Only vehicles without `OccupiedBy`, or occupied by the querying player, appear in proximity list.
- Distance is computed in world (f64) space via `GlobalPosition`, not `Transform`.

---

## 4. `enter_exit_vehicle`

**Schedule**: `Update`, after `update_enter_exit_prompts`  
**Purpose**: Process `enter_exit_pressed`; raise `EnterVehicleEvent` or `ExitVehicleEvent` based on current state.

| | |
|---|---|
| **Reads** | `OnFootInputState`, `VehicleInputState`, `PassengerOf` (optional), `EnterExitProximity`, `OccupiedBy` (optional) |
| **Writes** | `CurrentVehicle` (Resource) |
| **Events in** | — |
| **Events out** | `EnterVehicleEvent`, `ExitVehicleEvent` |

**Invariants**:
- One event emitted per press; never both enter and exit in same frame.

---

## 5. `apply_enter_vehicle`

**Schedule**: `Update`, after `enter_exit_vehicle`  
**Purpose**: React to `EnterVehicleEvent`; add `PassengerOf` to player, add `OccupiedBy` to vehicle, set player `Visibility::Hidden`, set player `GravityScale(0.0)`.

| | |
|---|---|
| **Reads** | `EnterVehicleEvent` |
| **Writes** | `Commands` (insert/remove components), `Visibility` (player), `GravityScale` (player) |
| **Events in** | `EnterVehicleEvent` |
| **Events out** | — |

---

## 6. `apply_exit_vehicle`

**Schedule**: `Update`, after `enter_exit_vehicle`  
**Purpose**: React to `ExitVehicleEvent`; remove `PassengerOf`, remove `OccupiedBy`, set `Visibility::Inherited`, restore `GravityScale(1.0)`, place player at vehicle exit position.

| | |
|---|---|
| **Reads** | `ExitVehicleEvent`, `GlobalPosition` (vehicle) |
| **Writes** | `Commands`, `GlobalPosition` (player), `Visibility` (player), `GravityScale` (player), `CurrentVehicle` (Resource → cleared) |
| **Events in** | `ExitVehicleEvent` |
| **Events out** | — |

**Invariants**:
- Player exit position = vehicle `GlobalPosition` + vehicle forward * 2.5 m (world space offset).

---

## 7. `locomotion_system`

**Schedule**: `FixedUpdate`, `PhysicsSet::StepSimulation::before`  
**Purpose**: Kinematic player movement — called only for entities with `PlayerLocomotionState` and `Without<PassengerOf>`.

| | |
|---|---|
| **Reads** | `OnFootInputState`, `PlayerLocomotionState`, `PlayerLocomotionConfig` (Resource) |
| **Writes** | `PlayerLocomotionState` (vertical_velocity, is_grounded), `GlobalPosition` |
| **Physics params** | `MoveAndSlide` (SystemParam) |
| **Events in** | — |
| **Events out** | — |

**Invariants**:
- `MoveAndSlide` result applied to `GlobalPosition`, not `Transform`.
- Jump impulse applied once (`jumped_this_frame` flag prevents double-jump).
- Slope steeper than `max_slope_angle_deg` treated as wall (no forward movement).

---

## 8. `ground_vehicle_system`

**Schedule**: `FixedUpdate`, `PhysicsSet::StepSimulation::before`  
**Purpose**: Raycast suspension + drive forces for `VehicleTag::GroundVehicle`.

| | |
|---|---|
| **Reads** | `VehicleInputState`, `GroundVehicleConfig`, `LinearVelocity`, `GlobalPosition`, `Rotation` |
| **Writes** | `GlobalPosition` (sync after `Forces`-driven PhysicsStep) |
| **Physics params** | `Forces` (QueryData), `SpatialQuery` (SystemParam) |
| **Events in** | — |
| **Events out** | `ImpactEvent` (on hard land) |

**Algorithm sketch**:
```
for each wheel:
  cast_ray(wheel_world_pos, -up, ray_length)
  compression = ray_length - hit_distance
  spring_force = spring_stiffness * compression - damper * velocity_dot_up
  forces.apply_linear_impulse_at_point(Vec3::Y * spring_force, wheel_world_pos)

drive_force = engine_force_n * throttle * forward_dir
brake_force = -linear_velocity.normalize_or_zero() * brake_force_n * brake
forces.apply_force(drive_force + brake_force)
```

---

## 9. `watercraft_system`

**Schedule**: `FixedUpdate`, `PhysicsSet::StepSimulation::before`  
**Purpose**: Buoyancy + water drag + thrust for `VehicleTag::Watercraft`.

| | |
|---|---|
| **Reads** | `VehicleInputState`, `WatercraftConfig`, `LinearVelocity`, `GlobalPosition` |
| **Physics params** | `Forces` (QueryData) |
| **Events out** | `ImpactEvent` |

**Algorithm sketch**:
```
depth_below_sea = sea_level - position.y
submerged_frac = clamp(depth_below_sea / hull_draft, 0.0, 1.0)
buoyancy = water_density * g * volume * submerged_frac
forces.apply_force(Vec3::Y * buoyancy)
forces.apply_force(-linear_velocity * water_drag)
thrust = Vec3::from(rotation * Vec3::Z) * thrust_force_n * throttle
forces.apply_force(thrust)
yaw_torque = Vec3::Y * turn_torque * steer
forces.apply_torque(yaw_torque)
```

---

## 10. `helicopter_system`

**Schedule**: `FixedUpdate`, `PhysicsSet::StepSimulation::before`  
**Purpose**: Helicopter lift + tilt for `VehicleTag::Helicopter`.

| | |
|---|---|
| **Reads** | `VehicleInputState`, `HelicopterConfig`, `LinearVelocity`, `Rotation` |
| **Physics params** | `Forces` (QueryData) |

**Algorithm sketch**:
```
gravity_force = mass * gravity_m_s2  // negative
hover_force = -gravity_force / hover_throttle
lift = hover_force * throttle
local_up = rotation * Vec3::Y
forces.apply_force(local_up * lift)

// Lateral control via tilt
lateral = Vec3::new(roll_input, 0.0, -pitch_input) * tilt_sensitivity
forces.apply_force(lateral * lift * 0.5)

// Yaw
forces.apply_torque(Vec3::Y * yaw_torque * steer)
```

---

## 11. `fixed_wing_system`

**Schedule**: `FixedUpdate`, `PhysicsSet::StepSimulation::before`  
**Purpose**: Aerodynamic lift + drag + torques for `VehicleTag::FixedWing`.

| | |
|---|---|
| **Reads** | `VehicleInputState`, `FixedWingConfig`, `LinearVelocity`, `Rotation` |
| **Physics params** | `Forces` (QueryData) |
| **Events out** | `ImpactEvent` |

**Algorithm sketch**:
```
airspeed = linear_velocity.length()
lift = if airspeed >= stall_speed {
  0.5 * air_density * airspeed^2 * lift_coefficient * wing_area
} else { 0.0 }
drag = 0.5 * air_density * airspeed^2 * drag_coefficient * wing_area
forward = rotation * Vec3::Z
up_local = rotation * Vec3::Y

forces.apply_force(forward * max_thrust * throttle)
forces.apply_force(up_local * lift - forward * drag)

pitch_t = rotation * Vec3::X * max_pitch_torque * pitch_input
roll_t  = rotation * Vec3::Z * max_roll_torque  * roll_input
yaw_t   = rotation * Vec3::Y * max_yaw_torque   * steer
forces.apply_torque(pitch_t + roll_t + yaw_t)
```

---

## 12. `impact_detection_system`

**Schedule**: `FixedUpdate`, after `PhysicsSet::StepSimulation`  
**Purpose**: Detect sudden velocity drop after physics step; emit `ImpactEvent` if relevant.

| | |
|---|---|
| **Reads** | `LinearVelocity`, `VehicleTag` (or `PlayerLocomotionState`) |
| **Writes** | buffered `ImpactEvent` |

**Invariants**:
- Minimum impact speed threshold: 20 m/s (configurable per vehicle type, FR-019).
- `ImpactEvent` for player triggers velocity-to-zero clamp, not damage (FR-019).

---

## 13. `camera_follow_system`

**Schedule**: `PostUpdate`, before Bevy transform propagation  
**Purpose**: Move camera's `GlobalPosition` to track the player.

| | |
|---|---|
| **Reads** | `GlobalPosition` (player entity), `Rotation` (player or vehicle) |
| **Writes** | `GlobalPosition` (camera entity) |

**Invariants**:
- Camera never reads `Transform` — only `GlobalPosition`.
- Offset: +3.0 m behind in XZ, +1.5 m in Y (world space).

---

## 14. `spawn_player`

**Schedule**: `Startup`  
**Purpose**: Spawn the player capsule entity at origin (0, 2, 0) world position.

| | |
|---|---|
| **Reads** | `PlayerLocomotionConfig` (Resource) |
| **Writes** | `Commands` |

---

## 15. `startup_spawn_vehicles`

**Schedule**: `PostStartup`  
**Purpose**: Read `WorldSpawnList`; for each entry look up `VehicleRegistry`, spawn entity with appropriate bundle.

| | |
|---|---|
| **Reads** | `WorldSpawnList` (Resource), `VehicleRegistry` (Resource) |
| **Writes** | `Commands` |

**Invariants**:
- Unknown `config` string → `warn!` and skip.
- Vehicle entity starts with `Visibility::Visible`, no `OccupiedBy`.
