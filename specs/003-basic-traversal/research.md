# Research: Basic Traversal System (Feature 003)

**Date**: 2026-02-24 | **Branch**: `003-basic-traversal`

---

## 1. Player Character Controller

**Decision**: Use `avian3d::prelude::MoveAndSlide` SystemParam  
**Rationale**: Avian3d 0.6.0-rc.1 ships a full `MoveAndSlide` collide-and-slide algorithm as a `SystemParam`. This is the correct idiomatic approach — not a component but a query helper injected into a system. The player entity is `RigidBody::Kinematic` with a `Collider::capsule`. Each `FixedUpdate` tick the system reads the buffered `InputState`, computes a desired velocity vector, calls `move_and_slide.move_and_slide(entity, velocity * dt, config)`, and the result slides the capsule against voxel terrain colliders.

Key API facts (from source):
- `MoveAndSlide` is `#[derive(SystemParam)]` — added to a system parameter list  
- `move_and_slide(shape, position, rotation, velocity, config, callback)` returns `MoveAndSlideOutput { translation, contacts }`  
- `MoveAndSlideConfig { max_slope_angle, snap_to_ground, up_vector, ... }`  
- Slope limit: `max_slope_angle` as radians  
- Gravity is handled manually: accumulate a vertical velocity component, clear on ground contact  

**Alternatives considered**: `CharacterController` plugin from older versions — not present in 0.6.0-rc.1. Custom shape-cast loop — more work with same result.

---

## 2. Vehicle Physics — Ground (Raycast Suspension)

**Decision**: `RigidBody::Dynamic` + per-wheel raycasts via `SpatialQuery` + `Forces` QueryData  
**Rationale**: Confirmed from source that `Forces` is a `QueryData` helper (not a component) that exposes `apply_force(Vec3)`, `apply_linear_impulse(Vec3)`, `apply_linear_impulse_at_point(force, point)`. Raycast suspension fires one `SpatialQuery::cast_ray` per wheel in the +Y-local direction; spring/damper force is `apply_linear_impulse_at_point` at the wheel world position each FixedUpdate.

Drive torque is `apply_force(forward * torque * throttle)`. Steering is via `AngularVelocity` smoothly targeted to `steer_angle * speed_factor`.

Key API facts:
- `SpatialQuery::cast_ray(origin, dir, max_dist, solid, filter)` → `Option<RayHitData>`
- `Forces` QueryData: `forces.apply_force(v)`, `forces.apply_linear_impulse(v)`, `forces.apply_linear_impulse_at_point(f, p)`, `forces.apply_torque(t)`
- `LinearVelocity(pub Vector)` — mutable component for reading/clamping speed  
- `AngularVelocity(pub Vector)` — mutable component for yaw steering  

**Alternatives considered**: Avian3d `RevoluteJoint` wheel entities — correct physics but high complexity and poor tunability via RON.

---

## 3. Buoyancy (Watercraft)

**Decision**: Manual per-frame buoyancy force via `Forces::apply_force`  
**Rationale**: Avian3d 0.6.0-rc.1 has NO built-in buoyancy component. Manual implementation: read the entity's `Position` (avian3d component), compute submerged depth vs. sea level height, compute `buoyancy_force = density * g * submerged_volume`, apply upward via `forces.apply_force(Vec3::Y * buoyancy_force)`. Also apply water drag: `forces.apply_force(-linear_velocity * drag_coefficient)`.  
Sea level is a flat plane at `PlanetConfig::sea_level_m` height on the +Y axis for MVP.

**Alternatives considered**: Third-party buoyancy crate — too heavy a dependency for MVP. Joint-based water simulation — out of scope.

---

## 4. Aircraft Physics

**Decision**: Custom aerodynamic forces via `Forces` each FixedUpdate  
**Rationale**: Helicopters: `lift_force = mass * gravity * throttle / hover_throttle_fraction` applied up-local. Tilt: rotate the lift vector by the stick input angle. Fixed-wing: `lift = 0.5 * air_density * airspeed^2 * lift_coefficient * wing_area` applied normal to forward vector; below `stall_speed_m_s` lift = 0. Drag applied backward proportional to `airspeed^2 * drag_coefficient`. Pitch/roll/yaw via `forces.apply_torque`.

**Alternatives considered**: Full flight simulation library — out of scope.

---

## 5. Input Integration

**Decision**: Extend existing `InputConfig` action map with traversal-specific actions; buffered `InputState` component per player/vehicle  
**Rationale**: The existing `InputConfig` uses `HashMap<String, GamepadButton/Axis>`. New traversal actions to add:
- Axes: `"ThrottleForward"` (R2/LeftStick Y), `"Brake"` (L2), `"SteerRight"` (LeftStick X), `"LookH"` (RightStick X), `"LookV"` (RightStick Y), `"Throttle"` (LeftStick Y for aircraft/boat)
- Buttons: `"EnterExitVehicle"` (Triangle/Y = `GamepadButton::North`), `"Jump"` (X/A = `GamepadButton::South`), `"Sprint"` (L3 = `GamepadButton::LeftThumb`)

`InputState` is a per-entity `Component` polled in `Update`, consumed in `FixedUpdate`. On-foot and vehicle contexts write different fields of `InputState`.

**Alternatives considered**: Separate input config per vehicle type — over-engineered for MVP; the action names are sufficient discriminators.

---

## 6. Floating-Origin Integration

**Decision**: All traversal entities write to `GlobalPosition(DVec3)` only; never touch `Transform`  
**Rationale**: FloatingOriginPlugin's `sync_transforms` in PostUpdate converts every `GlobalPosition` to a local `Transform.translation = (GlobalPosition - OriginFrame.position).as_vec3()`. MoveAndSlide output (`translation: Vec3`, local space) is added to the entity's `GlobalPosition`. Vehicle motion likewise: convert computed delta from f32 local to f64 and add to `GlobalPosition`.

Player should become the `FloatingOrigin` entity on spawn (replacing the debug camera), or alternatively the existing camera should track the player position by writing to its own `GlobalPosition`. For MVP: **player does NOT hold FloatingOrigin** — the orbital camera is kept for now; a "camera-follows-player" follow system writes the camera's `GlobalPosition` to shadow the player +3 m behind / +1.5 m up.

**Alternatives considered**: Transfer `FloatingOrigin` to player — architecturally correct but requires removing it from the camera, which breaks the current view. Deferred to a camera-system polish sprint.

---

## 7. Vehicle World Spawning (RON spawn-list)

**Decision**: `assets/vehicles/world_spawns.ron` → `Vec<VehicleSpawnEntry>` loaded at `PostStartup`  
**Rationale**: `VehicleSpawnEntry { config: String, position: (f64, f64, f64) }` maps a config filename (relative to `assets/vehicles/`) to a world position. Loaded by `TraversalPlugin::startup_spawn_vehicles`. Absent/invalid entries skipped with `warn!`. Zero Rust changes to add new vehicles.

---

## 8. LOD Integration

**Decision**: `LodLevel` component on vehicle entities; `TraversalPlugin` queries `LodSettings` to cull far vehicles  
**Rationale**: Vehicles at LOD-3 distance (> 100 km) are despawned and re-spawned when the viewer approaches. For MVP, if a vehicle entity's distance exceeds `lod_settings.thresholds[2]` it is given `Visibility::Hidden`. Full LOD mesh-swap is a follow-up.

---

## 9. Schedule Placement

| System | Schedule | Ordering |
|---|---|---|
| `poll_traversal_input` | `Update` | free |
| `update_enter_exit_prompts` | `Update` | after `poll_traversal_input` |
| `enter_exit_vehicle` | `Update` | after `update_enter_exit_prompts` |
| `locomotion_system` | `FixedUpdate` | PhysicsSet::StepSimulation::before |
| `ground_vehicle_system` | `FixedUpdate` | PhysicsSet::StepSimulation::before |
| `watercraft_system` | `FixedUpdate` | PhysicsSet::StepSimulation::before |
| `aircraft_system` | `FixedUpdate` | PhysicsSet::StepSimulation::before |
| `camera_follow_system` | `PostUpdate` | before TransformSystems::Propagate |
| `vehicle_lod_system` | `Update` | free |

---

## 10. Asset RON Loading

**Decision**: Synchronous `std::fs::read_to_string` at startup (same pattern as `PlanetConfig` / `InputConfig`)  
**Rationale**: Bevy 0.18's async asset system requires handles and loading states, which complicates startup sequencing. All other configs in this codebase use synchronous RON loading. Vehicle configs and player locomotion config will do the same: loaded in `Startup`, inserted as `Resource` or stored in a `Vec<VehicleConfig>` resource.
