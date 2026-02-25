# Implementation Plan: Basic Traversal System

**Branch**: `003-basic-traversal` | **Date**: 2026-02-24 | **Spec**: [spec.md](spec.md)  
**Research**: [research.md](research.md) | **Data Model**: [data-model.md](data-model.md) | **Contracts**: [contracts/traversal_api.md](contracts/traversal_api.md)

---

## Summary

Feature 003 implements the four traversal pillars required by Principle I: on-foot locomotion, ground vehicles (raycast suspension), watercraft (manual buoyancy), and aircraft (helicopter + fixed-wing aerodynamics). All movement is data-driven via RON configs; no physics parameters may be hardcoded. The player capsule uses `avian3d::MoveAndSlide` (kinematic); vehicles use `RigidBody::Dynamic` + the `Forces` QueryData for all force application. Input is split: `Update` reads gamepad/keyboard into `InputState` components; `FixedUpdate` consumes those components to apply forces. All entities write `GlobalPosition(DVec3)` only — `FloatingOriginPlugin` owns `Transform` sync.

---

## Technical Context

**Language/Version**: Rust stable 1.85+, 2024 edition  
**Primary Dependencies**: Bevy 0.18, avian3d 0.6.0-rc.1, ron 0.8, serde, glam 0.30  
**Storage**: RON asset files (synchronous `std::fs::read_to_string` at startup, per existing codebase pattern)  
**Testing**: `cargo test` — unit tests on config deserialization, property tests on physics helpers; quickstart manual test steps in [quickstart.md](quickstart.md)  
**Target Platform**: macOS, Linux, Windows (desktop)  
**Performance Goals**: ≥ 60 FPS on RTX 3060-class hardware with player + 5 vehicles + voxel streaming active (SC-003)  
**Constraints**: No direct `Transform::translation` writes on traversal entities; no `unwrap()` outside bootstrap; `clippy -D warnings` must pass; 85% line coverage on core traversal systems (Principle VI)  
**Scale/Scope**: Single player, local; 4 vehicle categories, ~10 vehicle RON configs at launch

---

## Constitution Check

*GATE: Must pass before implementation. Any ERROR row blocks merge.*

| Principle | Gate | Status | Notes |
|---|---|---|---|
| **I. Project Vision** | All 4 traversal modes (foot, ground, water, air) implemented | ✅ PASS | All four covered by this feature |
| **I. Project Vision** | 60+ FPS on mid-range hardware | ✅ PASS | FixedUpdate/Update split + LOD integration ensure budget; SC-003 measures this |
| **I. Modding** | New vehicle = RON file only, zero Rust changes | ✅ PASS | FR-020, SC-004 — `VehicleRegistry` + `WorldSpawnList` enforce this |
| **II. Architecture** | Pure Bevy ECS — no OOP game-object patterns | ✅ PASS | All state in components; no struct methods holding entity refs |
| **II. Architecture** | `TraversalPlugin` MUST NOT modify VoxelWorldPlugin, FloatingOriginPlugin, or PhysicsPlugin internals | ✅ PASS | FR-024 explicit; plugin reads their public components only |
| **II. Architecture** | Data-driven via `VehicleComponent`, `WatercraftComponent`, `AircraftComponent`, `PlayerLocomotionComponent` | ✅ PASS | Mapped to `VehiclePhysicsConfig` enum variants; matches Principle II intent |
| **II. Architecture** | `GlobalPosition(DVec3)` / floating origin for all entities | ✅ PASS | FR-023; `MoveAndSlide` output added to `GlobalPosition`, never `Transform` |
| **III. Tech Stack** | avian3d 0.6.0-rc.1; no alternative physics lib | ✅ PASS | Confirmed API from source: `MoveAndSlide`, `Forces`, `SpatialQuery` |
| **III. Tech Stack** | `unsafe` forbidden except documented math kernels | ✅ PASS | No unsafe needed in traversal layer |
| **IV. Mandatory Plugins** | `TraversalPlugin` must be present and operational | ✅ PASS | This feature implements it |
| **V. Code Quality** | `cargo fmt` + `cargo clippy -D warnings` | ✅ PASS | Must be verified before PR |
| **V. Code Quality** | Every public item has `///` doc | ✅ PASS | All public structs/systems documented in implementation |
| **V. Code Quality** | No `unwrap()` outside main/bootstrap | ✅ PASS | All asset loading uses `expect()` or `warn!`+default fallback |
| **VI. Testing** | 85% line coverage on traversal systems | ⚠ PLAN | Unit tests for config deserialization, physics helpers; manual tests via quickstart.md |
| **VI. Testing** | Property tests for physics edge cases | ⚠ PLAN | Needed: impact clamp, buoyancy overflow guard, slope angle gating |
| **VII. UX** | Player spawns in varied biome on foot | ✅ PASS | FR-001, SC-001 |
| **VII. UX** | All 4 traversal categories accessible in first session | ✅ PASS | SC-002 |
| **VII. UX** | PS4/PS5 controller auto-detected | ✅ PASS | Extends existing `InputConfig`; FR-027 schedules ensure no input lag |
| **VIII. Modding** | Vehicle defs in RON + Bevy reflection | ✅ PASS | `#[derive(Reflect)]` on all config structs; FR-020 |

**Post-design re-check**: ✅ No new violations introduced by Phase 1 design. `⚠ PLAN` items become `tasks.md` entries in `speckit.tasks`.

---

## Project Structure

### Documentation (this feature)

```text
specs/003-basic-traversal/
├── plan.md                         ← this file
├── spec.md                         ← complete (27 FRs, 4 USs, 6 SCs)
├── research.md                     ← Phase 0 (written)
├── data-model.md                   ← Phase 1 (written)
├── quickstart.md                   ← Phase 1 (written)
├── contracts/
│   └── traversal_api.md            ← Phase 1 (written)
├── checklists/
│   └── requirements.md             ← exists
└── tasks.md                        ← Phase 2 (speckit.tasks — not yet created)
```

### Source Code

```text
src/
├── main.rs                         (register TraversalPlugin — already stubbed)
└── plugins/
    └── traversal/                  (rename traversal_plugin.rs → traversal/mod.rs)
        ├── mod.rs                  (TraversalPlugin impl — registers everything)
        ├── components.rs           (PlayerLocomotionState, OnFootInputState,
        │                            VehicleInputState, VehicleTag, PassengerOf,
        │                            OccupiedBy, EnterExitProximity, PlayerTag)
        ├── resources.rs            (CurrentVehicle, VehicleRegistry, WorldSpawnList)
        ├── events.rs               (ImpactEvent, EnterVehicleEvent, ExitVehicleEvent)
        ├── config.rs               (PlayerLocomotionConfig, VehicleConfig,
        │                            VehiclePhysicsConfig, GroundVehicleConfig,
        │                            WatercraftConfig, HelicopterConfig, FixedWingConfig,
        │                            WheelConfig, VehicleSpawnEntry)
        └── systems/
            ├── mod.rs              (re-exports all system fns)
            ├── spawn.rs            (spawn_player, startup_load_configs,
            │                        startup_spawn_vehicles)
            ├── input.rs            (poll_on_foot_input, poll_vehicle_input)
            ├── enter_exit.rs       (update_enter_exit_prompts, enter_exit_vehicle,
            │                        apply_enter_vehicle, apply_exit_vehicle)
            ├── locomotion.rs       (locomotion_system)
            ├── ground_vehicle.rs   (ground_vehicle_system)
            ├── watercraft.rs       (watercraft_system)
            ├── aircraft.rs         (helicopter_system, fixed_wing_system)
            ├── impact.rs           (impact_detection_system)
            └── camera_follow.rs    (camera_follow_system)

assets/
├── player/
│   └── locomotion.ron
└── vehicles/
    ├── world_spawns.ron
    ├── car.ron
    ├── truck.ron
    ├── motorboat.ron
    ├── airplane.ron
    └── helicopter.ron
```

---

## Phase 1: Component & Resource Design

> Full details in [data-model.md](data-model.md). Summary here for plan completeness.

### New Components

| Component | Purpose |
|---|---|
| `PlayerLocomotionState` | Vertical velocity, is_grounded, jumped_this_frame |
| `OnFootInputState` | Buffered per-frame on-foot input (move_dir, look_delta, jump, sprint, enter_exit) |
| `VehicleInputState` | Buffered per-frame vehicle input (throttle, brake, steer, pitch, roll, enter_exit) |
| `VehicleTag` | Enum discriminant — GroundVehicle / Watercraft / Helicopter / FixedWing |
| `PassengerOf(Entity)` | On player; player is passenger of this vehicle entity |
| `OccupiedBy(Entity)` | On vehicle; this vehicle is occupied by this player entity |
| `EnterExitProximity` | On vehicle; nearby player list (SmallVec<[Entity; 2]>) |
| `PlayerTag` | Marker for player entity queries |

### New Resources

| Resource | Source | Purpose |
|---|---|---|
| `PlayerLocomotionConfig` | `assets/player/locomotion.ron` | All on-foot movement parameters |
| `VehicleRegistry` | `assets/vehicles/*.ron` | Vec of all loaded VehicleConfigs |
| `WorldSpawnList` | `assets/vehicles/world_spawns.ron` | Spawn positions for all world vehicles |
| `CurrentVehicle` | runtime | Option<Entity> — currently occupied vehicle |

### New Events

| Event | Emitter | Consumer |
|---|---|---|
| `EnterVehicleEvent` | `enter_exit_vehicle` | `apply_enter_vehicle` |
| `ExitVehicleEvent` | `enter_exit_vehicle` | `apply_exit_vehicle` |
| `ImpactEvent` | `impact_detection_system`, vehicle systems | UI / audio (later features) |

---

## Phase 2: System Registration Order

```rust
// TraversalPlugin::build(app):
app
    // Resources
    .init_resource::<CurrentVehicle>()
    .init_resource::<VehicleRegistry>()
    .init_resource::<WorldSpawnList>()
    // Events
    .add_event::<EnterVehicleEvent>()
    .add_event::<ExitVehicleEvent>()
    .add_event::<ImpactEvent>()
    // Reflect
    .register_type::<PlayerLocomotionState>()
    .register_type::<OnFootInputState>()
    .register_type::<VehicleInputState>()
    .register_type::<VehicleTag>()
    .register_type::<PassengerOf>()
    .register_type::<OccupiedBy>()
    .register_type::<PlayerLocomotionConfig>()
    .register_type::<VehicleConfig>()
    // Startup
    .add_systems(Startup, (
        startup_load_configs,
        spawn_player.after(startup_load_configs),
    ))
    .add_systems(PostStartup, startup_spawn_vehicles)
    // Update — input polling + enter/exit
    .add_systems(Update, (
        poll_on_foot_input,
        poll_vehicle_input,
        update_enter_exit_prompts.after(poll_on_foot_input),
        enter_exit_vehicle.after(update_enter_exit_prompts),
        apply_enter_vehicle.after(enter_exit_vehicle),
        apply_exit_vehicle.after(enter_exit_vehicle),
        vehicle_lod_system,
    ))
    // FixedUpdate — all force application
    .add_systems(FixedUpdate, (
        locomotion_system,
        ground_vehicle_system,
        watercraft_system,
        helicopter_system,
        fixed_wing_system,
    ).before(PhysicsSet::StepSimulation))
    .add_systems(FixedUpdate,
        impact_detection_system.after(PhysicsSet::StepSimulation))
    // PostUpdate — camera follow
    .add_systems(PostUpdate,
        camera_follow_system.before(TransformSystem::TransformPropagate));
```

---

## Phase 3: Asset RON Schemas

### `assets/player/locomotion.ron`

```ron
PlayerLocomotionConfig(
    walk_speed_m_s: 5.0,
    sprint_speed_m_s: 10.0,
    jump_impulse_m_s: 6.0,
    gravity_m_s2: -20.0,
    max_slope_angle_deg: 45.0,
    interact_radius_m: 3.0,
    capsule_radius_m: 0.35,
    capsule_height_m: 1.8,
)
```

### `assets/vehicles/world_spawns.ron`

```ron
WorldSpawnList(
    entries: [
        VehicleSpawnEntry(
            config: "car.ron",
            position: (10.0, 0.5, 0.0),
            yaw_deg: Some(0.0),
        ),
        VehicleSpawnEntry(
            config: "motorboat.ron",
            position: (0.0, 0.0, 50.0),
            yaw_deg: None,
        ),
        VehicleSpawnEntry(
            config: "helicopter.ron",
            position: (-20.0, 1.0, -20.0),
            yaw_deg: None,
        ),
        VehicleSpawnEntry(
            config: "airplane.ron",
            position: (100.0, 0.5, 0.0),
            yaw_deg: Some(0.0),
        ),
    ]
)
```

### `assets/vehicles/car.ron`

```ron
VehicleConfig(
    name: "Scout Car",
    physics: GroundVehicle(GroundVehicleConfig(
        mass_kg: 1200.0,
        max_speed_m_s: 30.0,
        engine_force_n: 5000.0,
        brake_force_n: 8000.0,
        max_steer_angle_deg: 35.0,
        wheels: [
            WheelConfig( local_offset: ( 1.0, 0.0,  1.7), ray_length: 0.7, rest_length: 0.5,
                         spring_stiffness: 25000.0, damper: 2000.0 ),
            WheelConfig( local_offset: (-1.0, 0.0,  1.7), ray_length: 0.7, rest_length: 0.5,
                         spring_stiffness: 25000.0, damper: 2000.0 ),
            WheelConfig( local_offset: ( 1.0, 0.0, -1.7), ray_length: 0.7, rest_length: 0.5,
                         spring_stiffness: 25000.0, damper: 2000.0 ),
            WheelConfig( local_offset: (-1.0, 0.0, -1.7), ray_length: 0.7, rest_length: 0.5,
                         spring_stiffness: 25000.0, damper: 2000.0 ),
        ],
    )),
)
```

### `assets/vehicles/motorboat.ron`

```ron
VehicleConfig(
    name: "Scout Boat",
    physics: Watercraft(WatercraftConfig(
        mass_kg: 800.0,
        max_speed_m_s: 15.0,
        thrust_force_n: 3000.0,
        turn_torque_n_m: 1500.0,
        buoyancy_volume_m3: 4.0,
        water_drag: 150.0,
        sea_level_m: 0.0,
    )),
)
```

### `assets/vehicles/helicopter.ron`

```ron
VehicleConfig(
    name: "Scout Helicopter",
    physics: Helicopter(HelicopterConfig(
        mass_kg: 2000.0,
        max_ascent_m_s: 15.0,
        max_lateral_m_s: 20.0,
        hover_throttle: 0.5,
        tilt_sensitivity: 15.0,
        yaw_torque_n_m: 3000.0,
    )),
)
```

### `assets/vehicles/airplane.ron`

```ron
VehicleConfig(
    name: "Scout Plane",
    physics: FixedWing(FixedWingConfig(
        mass_kg: 3000.0,
        max_thrust_n: 12000.0,
        lift_coefficient: 1.2,
        drag_coefficient: 0.05,
        wing_area_m2: 16.0,
        stall_speed_m_s: 30.0,
        max_pitch_torque_n_m: 4000.0,
        max_roll_torque_n_m: 5000.0,
        max_yaw_torque_n_m: 2000.0,
    )),
)
```

---

## Phase 4: Key Code Snippets

### 4a — Player Locomotion System

```rust
/// Applies kinematic character movement via avian3d MoveAndSlide.
/// Runs in FixedUpdate, before PhysicsSet::StepSimulation.
pub fn locomotion_system(
    mut query: Query<
        (Entity, &OnFootInputState, &mut PlayerLocomotionState, &mut GlobalPosition),
        Without<PassengerOf>,
    >,
    config: Res<PlayerLocomotionConfig>,
    time: Res<Time<Fixed>>,
    mut move_and_slide: MoveAndSlide,
) {
    let dt = time.delta_secs();
    for (entity, input, mut state, mut global_pos) in &mut query {
        let speed = if input.sprint_held { config.sprint_speed_m_s } else { config.walk_speed_m_s };
        let horizontal = Vec3::new(input.move_dir.x, 0.0, -input.move_dir.y) * speed;

        if state.is_grounded && input.jump_pressed && !state.jumped_this_frame {
            state.vertical_velocity = config.jump_impulse_m_s;
            state.jumped_this_frame = true;
        } else if state.is_grounded {
            state.vertical_velocity = 0.0;
            state.jumped_this_frame = false;
        }
        state.vertical_velocity += config.gravity_m_s2 * dt;
        let desired_vel = horizontal + Vec3::Y * state.vertical_velocity;

        let slide_config = MoveAndSlideConfig {
            max_slope_angle: config.max_slope_angle_deg.to_radians(),
            up_vector: Vec3::Y,
            snap_to_ground: state.is_grounded,
            ..default()
        };
        let output = move_and_slide.move_and_slide(entity, desired_vel * dt, slide_config);
        state.is_grounded = output.is_grounded();
        global_pos.0 += output.translation.as_dvec3();
    }
}
```

### 4b — Ground Vehicle Raycast Suspension

```rust
/// Raycast spring/damper suspension + drive force for ground vehicles.
pub fn ground_vehicle_system(
    mut query: Query<
        (Entity, &VehicleInputState, &GroundVehicleConfig,
         &LinearVelocity, &Rotation, &mut GlobalPosition),
        With<VehicleTag>,
    >,
    spatial_query: SpatialQuery,
    mut forces: Query<Forces>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (entity, input, config, lin_vel, rotation, mut global_pos) in &mut query {
        let Ok(mut f) = forces.get_mut(entity) else { continue };
        let forward = rotation.0 * Vec3::NEG_Z;
        let up      = rotation.0 * Vec3::Y;
        let body_vel = lin_vel.0;

        for wheel in &config.wheels {
            let wheel_world = global_pos.0.as_vec3() + (rotation.0 * wheel.local_offset);
            let filter = SpatialQueryFilter::default().exclude_entities([entity]);
            if let Some(hit) = spatial_query.cast_ray(
                wheel_world, -up, wheel.ray_length, true, &filter,
            ) {
                let compression = wheel.rest_length - hit.distance;
                if compression > 0.0 {
                    let vel_at_wheel = body_vel.dot(up);
                    let spring_f = wheel.spring_stiffness * compression
                                 - wheel.damper * vel_at_wheel;
                    f.apply_linear_impulse_at_point(up * spring_f * dt, wheel_world);
                }
            }
        }

        let speed = body_vel.dot(forward);
        let drive = if speed.abs() < config.max_speed_m_s {
            forward * config.engine_force_n * input.throttle
        } else { Vec3::ZERO };
        let brake = -body_vel.normalize_or_zero() * config.brake_force_n * input.brake;
        f.apply_force(drive + brake);
        let steer_rad = input.steer * config.max_steer_angle_deg.to_radians();
        f.apply_torque(up * steer_rad * speed * 100.0);
    }
}
```

### 4c — Watercraft Buoyancy

```rust
pub fn watercraft_system(
    query: Query<
        (Entity, &VehicleInputState, &WatercraftConfig,
         &LinearVelocity, &Rotation, &GlobalPosition),
        With<VehicleTag>,
    >,
    mut forces: Query<Forces>,
) {
    const WATER_DENSITY: f32 = 1025.0;
    const GRAVITY: f32 = 9.81;
    for (entity, input, config, lin_vel, rotation, global_pos) in &query {
        let Ok(mut f) = forces.get_mut(entity) else { continue };
        let depth = config.sea_level_m - global_pos.0.y as f32;
        let submerged = (depth / 1.2).clamp(0.0, 1.0);
        let buoyancy = WATER_DENSITY * GRAVITY * config.buoyancy_volume_m3 * submerged;
        f.apply_force(Vec3::Y * buoyancy);
        f.apply_force(-lin_vel.0 * config.water_drag);
        let forward = rotation.0 * Vec3::NEG_Z;
        let speed = lin_vel.0.dot(forward);
        if speed.abs() < config.max_speed_m_s {
            f.apply_force(forward * config.thrust_force_n * input.throttle);
        }
        f.apply_torque(Vec3::Y * config.turn_torque_n_m * input.steer);
    }
}
```

### 4d — Helicopter Lift

```rust
pub fn helicopter_system(
    query: Query<
        (Entity, &VehicleInputState, &HelicopterConfig, &Mass, &Rotation),
        With<VehicleTag>,
    >,
    mut forces: Query<Forces>,
) {
    const GRAVITY: f32 = 9.81;
    for (entity, input, config, mass, rotation) in &query {
        let Ok(mut f) = forces.get_mut(entity) else { continue };
        let hover_lift = mass.0 * GRAVITY;
        let lift = hover_lift * (input.throttle / config.hover_throttle.max(0.01));
        let local_up = rotation.0 * Vec3::Y;
        f.apply_force(local_up * lift);
        let tilt_scale = config.tilt_sensitivity.to_radians() * lift * 0.3;
        f.apply_force(Vec3::new(input.roll, 0.0, -input.pitch) * tilt_scale);
        f.apply_torque(Vec3::Y * config.yaw_torque_n_m * input.steer);
    }
}
```

### 4e — Fixed-Wing Aerodynamics

```rust
pub fn fixed_wing_system(
    query: Query<
        (Entity, &VehicleInputState, &FixedWingConfig, &LinearVelocity, &Rotation),
        With<VehicleTag>,
    >,
    mut forces: Query<Forces>,
) {
    const AIR_DENSITY: f32 = 1.225;
    for (entity, input, config, lin_vel, rotation) in &query {
        let Ok(mut f) = forces.get_mut(entity) else { continue };
        let forward  = rotation.0 * Vec3::NEG_Z;
        let up       = rotation.0 * Vec3::Y;
        let airspeed = lin_vel.0.length();
        f.apply_force(forward * config.max_thrust_n * input.throttle);
        if airspeed >= config.stall_speed_m_s {
            let q    = 0.5 * AIR_DENSITY * airspeed * airspeed;
            let lift = q * config.lift_coefficient * config.wing_area_m2;
            let drag = q * config.drag_coefficient * config.wing_area_m2;
            f.apply_force(up * lift - forward * drag);
        }
        let pitch = rotation.0 * Vec3::X * config.max_pitch_torque_n_m * input.pitch;
        let roll  = rotation.0 * Vec3::Z * config.max_roll_torque_n_m  * input.roll;
        let yaw   = rotation.0 * Vec3::Y * config.max_yaw_torque_n_m   * input.steer;
        f.apply_torque(pitch + roll + yaw);
    }
}
```

### 4f — Enter/Exit Lifecycle

```rust
pub fn apply_enter_vehicle(
    mut commands: Commands,
    mut events: EventReader<EnterVehicleEvent>,
    mut current_vehicle: ResMut<CurrentVehicle>,
) {
    for ev in events.read() {
        commands.entity(ev.player)
            .insert(PassengerOf(ev.vehicle))
            .insert(Visibility::Hidden)
            .insert(GravityScale(0.0));
        commands.entity(ev.vehicle).insert(OccupiedBy(ev.player));
        current_vehicle.0 = Some(ev.vehicle);
    }
}

pub fn apply_exit_vehicle(
    mut commands: Commands,
    mut events: EventReader<ExitVehicleEvent>,
    vehicle_positions: Query<&GlobalPosition>,
    mut player_positions: Query<&mut GlobalPosition, Without<VehicleTag>>,
    mut current_vehicle: ResMut<CurrentVehicle>,
) {
    for ev in events.read() {
        if let Ok(vpos) = vehicle_positions.get(ev.vehicle) {
            if let Ok(mut ppos) = player_positions.get_mut(ev.player) {
                ppos.0 = vpos.0 + DVec3::new(0.0, 0.5, 2.5);
            }
        }
        commands.entity(ev.player)
            .remove::<PassengerOf>()
            .insert(Visibility::Inherited)
            .insert(GravityScale(1.0));
        commands.entity(ev.vehicle).remove::<OccupiedBy>();
        current_vehicle.0 = None;
    }
}
```

### 4g — Startup Config Loading

```rust
pub fn startup_load_configs(mut commands: Commands) {
    let locomotion_config = std::fs::read_to_string("assets/player/locomotion.ron")
        .ok()
        .and_then(|s| ron::from_str::<PlayerLocomotionConfig>(&s).ok())
        .unwrap_or_else(|| {
            warn!("locomotion.ron missing or invalid — using built-in defaults");
            PlayerLocomotionConfig::default()
        });
    commands.insert_resource(locomotion_config);

    let mut registry = VehicleRegistry::default();
    if let Ok(dir) = std::fs::read_dir("assets/vehicles/") {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "ron")
                && path.file_name().is_some_and(|n| n != "world_spawns.ron")
            {
                match std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| ron::from_str::<VehicleConfig>(&s).ok())
                {
                    Some(cfg) => registry.0.push(cfg),
                    None => warn!("Vehicle config {:?} invalid — skipping", path),
                }
            }
        }
    }
    commands.insert_resource(registry);

    let spawn_list = std::fs::read_to_string("assets/vehicles/world_spawns.ron")
        .ok()
        .and_then(|s| ron::from_str::<WorldSpawnList>(&s).ok())
        .unwrap_or_else(|| {
            warn!("world_spawns.ron missing — no vehicles will be placed");
            WorldSpawnList::default()
        });
    commands.insert_resource(spawn_list);
}
```

---

## Phase 5: Input Config Extension

Extend `src/plugins/input/config.rs` `InputConfig::default()` — backward-compatible additions only:

```rust
// Vehicle axes
axis_map.insert("ThrottleForward".into(), GamepadAxis::RightZ);
axis_map.insert("Brake".into(),          GamepadAxis::LeftZ);
axis_map.insert("SteerRight".into(),     GamepadAxis::LeftStickX);

// Vehicle buttons
button_map.insert("EnterExitVehicle".into(), GamepadButton::North); // Triangle/Y

// Keyboard fallbacks
keyboard_map.insert("ThrottleForward".into(), KeyCode::KeyW);
keyboard_map.insert("Brake".into(),           KeyCode::KeyS);
keyboard_map.insert("EnterExitVehicle".into(), KeyCode::KeyF);
```

---

## Phase 6: FloatingOrigin Integration Rules

1. **Never write `Transform::translation`** on traversal entities — `FloatingOriginPlugin::sync_transforms` in `PostUpdate` owns that.
2. **`MoveAndSlide` delta** — output is `Vec3` (local). Cast with `.as_dvec3()`, add to `GlobalPosition.0`.
3. **Dynamic vehicle bodies** — for MVP, vehicles are placed near the origin (world_spawns positions). Avian3d `Position` (f32) ≈ `GlobalPosition` (f64) within the 1 000 m local frame. Full bidirectional `Position` ↔ `GlobalPosition` sync is a follow-up polish task.
4. **SC-006 compliance** — `FloatingOriginPlugin` shifts the local frame when the camera drifts past its threshold; `TraversalPlugin` needs no special logic for this.

---

## Phase 7: LOD Integration

```rust
/// Culls distant vehicles to preserve frame budget (FR-025, SC-003).
pub fn vehicle_lod_system(
    mut query: Query<(&GlobalPosition, &mut Visibility), With<VehicleTag>>,
    camera_pos: Query<&GlobalPosition, With<Camera>>,
    lod_settings: Res<LodSettings>,
) {
    let Ok(cam_pos) = camera_pos.get_single() else { return };
    for (vpos, mut vis) in &mut query {
        let dist = (vpos.0 - cam_pos.0).length() as f32;
        *vis = if dist > lod_settings.thresholds[2] {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
```

---

## Edge Cases & Mitigations

| Scenario | Mitigation |
|---|---|
| Vehicle spawns inside terrain | `startup_spawn_vehicles` sweeps ray upward from spawn; shifts Y by 0.1 m until clear (max 50 iterations) |
| Player enters vehicle mid-air | Entry permitted; `GravityScale(0.0)` prevents falling through vehicle |
| Chunk evicted while player is in vehicle | `GlobalPosition` retained on entity; chunk reload restores terrain around entity |
| `world_spawns.ron` missing | `warn!`, `WorldSpawnList::default()` (empty) — session continues |
| Vehicle RON invalid | `warn!` per file, skip — other vehicles unaffected (FR-022) |
| Player tries to enter occupied vehicle | `enter_exit_vehicle` checks `OccupiedBy` — ignores press |
| Aircraft hits terrain above impact threshold | `ImpactEvent` emitted + `LinearVelocity` → `Vec3::ZERO`; craft rests on terrain |
| Fixed-wing below stall speed | Lift = 0, drag still applies; aircraft falls ballistically |
| Water surface missing / sea_level misconfigured | `depth.clamp(0.0, 1.0)` — no negative buoyancy |
| Wheel count = 0 | Logged as `error!` at spawn; vehicle spawned but suspension skipped |
| Double jump attempt | `jumped_this_frame` flag prevents re-triggering until grounded again |

---

## Testing Plan

### Unit Tests

| Test | Target | Assert |
|---|---|---|
| `test_locomotion_config_defaults` | `PlayerLocomotionConfig::default()` | All fields in valid range |
| `test_vehicle_config_deserialize_car` | `car.ron` | Deserializes without error; 4 wheels |
| `test_vehicle_config_deserialize_boat` | `motorboat.ron` | `sea_level_m = 0.0` |
| `test_buoyancy_zero_depth` | buoyancy fn | Force = 0 when depth ≤ 0 |
| `test_buoyancy_full_submersion` | buoyancy fn | Force = ρgV at depth = 1 |
| `test_fixed_wing_stall` | lift fn | Lift = 0 when airspeed < stall_speed |
| `test_impact_clamp` | impact fn | Velocity clamped to zero above threshold |
| `test_slope_gating` | slope fn | Forward blocked on 50° surface |
| `test_enter_exit_components` | apply_enter/exit | PassengerOf / OccupiedBy added/removed correctly |

### Property Tests

| Property | Invariant |
|---|---|
| `prop_buoyancy_never_negative` | Force Y ≥ 0 for any depth input |
| `prop_global_pos_near_zero` | `Transform.translation.length() ≤ 1 000 m` after arbitrary travel |
| `prop_wheel_spring_stable` | No NaN / infinite force at any compression in [0.0, 1.0] |

### Manual / Integration Tests

See [quickstart.md](quickstart.md) for the full 12-step procedure covering all vehicle types, PS4 controller, FloatingOrigin precision, and 60 FPS gate.

---

## Next Steps

Out of scope for Feature 003; must be tracked:

1. **Camera plugin** — Third-person camera with spring arm and obstacle avoidance; follow-up feature.
2. **Swimming state** — FR-004 on-foot → swimming transition needs a `swimming_system` + `SwimmingConfig`; deferred to Feature 003a or 004.
3. **Vehicle LOD mesh-swap** — MVP hides/shows whole mesh; proper multi-LOD via Bevy scenes is Feature 005.
4. **Sound & VFX** — `ImpactEvent` and `EnterVehicleEvent` need audio/particle consumers; Feature 006+.
5. **FloatingOrigin full sync** — Bidirectional `avian3d::Position` ↔ `GlobalPosition` for Dynamic bodies at extreme distances; polish sprint.
6. **Enter/exit UI prompt** — `EnterExitProximity` provides data; HUD widget is Feature 007.
7. **CI coverage gate** — `cargo-llvm-cov` to enforce 85% (Principle VI); dev-ops feature.
8. **Multiplayer vehicle state** — Out of scope per spec Assumptions; networking milestone.