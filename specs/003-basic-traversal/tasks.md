# Tasks: Basic Traversal System

**Feature**: 003-basic-traversal  
**Input**: [`plan.md`](plan.md) · [`spec.md`](spec.md) · [`data-model.md`](data-model.md) · [`contracts/traversal_api.md`](contracts/traversal_api.md) · [`research.md`](research.md) · [`quickstart.md`](quickstart.md)

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (operates on different files, no incomplete dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4)
- File paths are exact; use `src/plugins/traversal/` for all new traversal source

---

## Phase 1: Setup

**Purpose**: Restructure the traversal plugin stub into the proper module tree and scaffold all empty files so the project compiles throughout development.

- [X] T001 Rename `src/plugins/traversal_plugin.rs` → `src/plugins/traversal/mod.rs`; update `mod traversal_plugin` → `mod traversal` import in `src/main.rs` (or wherever it is registered)
- [X] T002 [P] Create empty module stub `src/plugins/traversal/components.rs` with `// TODO: components` and `pub use` in `src/plugins/traversal/mod.rs`
- [X] T003 [P] Create empty module stub `src/plugins/traversal/resources.rs` with `// TODO: resources` and `pub use` in `src/plugins/traversal/mod.rs`
- [X] T004 [P] Create empty module stub `src/plugins/traversal/events.rs` with `// TODO: events` and `pub use` in `src/plugins/traversal/mod.rs`
- [X] T005 [P] Create empty module stub `src/plugins/traversal/config.rs` with `// TODO: config structs` and `pub use` in `src/plugins/traversal/mod.rs`
- [X] T006 [P] Create `src/plugins/traversal/systems/mod.rs` plus empty stub files: `spawn.rs`, `input.rs`, `enter_exit.rs`, `locomotion.rs`, `ground_vehicle.rs`, `watercraft.rs`, `aircraft.rs`, `impact.rs`, `camera_follow.rs`, `lod.rs`; add `mod systems` to `src/plugins/traversal/mod.rs`
- [X] T007 [P] Create `assets/player/` and `assets/vehicles/` directories; add placeholder `assets/player/locomotion.ron` and `assets/vehicles/world_spawns.ron` (empty `WorldSpawnList(entries:[])`) so startup does not panic before assets are written

**Checkpoint**: `cargo build` compiles with zero errors after all stubs are in place.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: All shared config types, components, resources, events, and startup infrastructure that every user story depends on. No user story work can begin until this phase is complete.

**⚠️ CRITICAL**: Phases 3–6 all depend on this phase being complete.

- [X] T008 Implement all RON-serializable config structs in `src/plugins/traversal/config.rs`: `PlayerLocomotionConfig` (with `Default`), `WheelConfig`, `GroundVehicleConfig`, `WatercraftConfig`, `HelicopterConfig`, `FixedWingConfig`, `VehiclePhysicsConfig` (enum), `VehicleConfig`, `VehicleSpawnEntry`, `WorldSpawnList` — all with `#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]`
- [X] T009 [P] Implement all ECS components in `src/plugins/traversal/components.rs`: `PlayerTag`, `PlayerLocomotionState` (with `Default`), `OnFootInputState` (with `Default`), `VehicleInputState` (with `Default`), `VehicleTag` (enum), `PassengerOf(Entity)`, `OccupiedBy(Entity)`, `EnterExitProximity` (with `Default`) — all with `#[derive(Component, Reflect)]`
- [X] T010 [P] Implement all resources in `src/plugins/traversal/resources.rs`: `CurrentVehicle` (with `Default`), `VehicleRegistry` (with `Default`) — with `#[derive(Resource, Reflect)]`
- [X] T011 [P] Implement all events in `src/plugins/traversal/events.rs`: `ImpactEvent { entity, impact_speed_m_s, contact_normal: Vec3 }`, `EnterVehicleEvent { player, vehicle }`, `ExitVehicleEvent { player, vehicle }` — with `#[derive(Event, Debug)]`; see `data-model.md §4` for full field list
- [X] T012 Implement `startup_load_configs` system in `src/plugins/traversal/systems/spawn.rs` — reads `assets/player/locomotion.ron` → inserts `PlayerLocomotionConfig`; reads `assets/vehicles/*.ron` → builds `VehicleRegistry`; reads `assets/vehicles/world_spawns.ron` → inserts `WorldSpawnList`; uses `warn!` + default fallback on any missing/invalid file (depends on T008, T010)
- [X] T013 [P] Extend `InputConfig::default()` in `src/plugins/input/config.rs` — add axis actions `"ThrottleForward"` (RightZ/R2), `"Brake"` (LeftZ/L2), `"SteerRight"` (LeftStickX); button action `"EnterExitVehicle"` (North/Triangle); keyboard fallbacks `"ThrottleForward"` → W, `"Brake"` → S, `"EnterExitVehicle"` → F
- [X] T014 Wire foundational types into `TraversalPlugin::build` in `src/plugins/traversal/mod.rs` — `init_resource` for `CurrentVehicle`/`VehicleRegistry`/`WorldSpawnList`; `add_event` for all three events; `register_type` for all components, resources, and config structs; add `Startup` systems `startup_load_configs` (depends on T009–T013)

**Checkpoint**: `cargo build` compiles; `startup_load_configs` runs without panic on startup; all types visible in Bevy inspector (dev_tools build).

---

## Phase 3: User Story 1 — On-Foot Locomotion (Priority: P1) 🎯 MVP

**Goal**: Player capsule spawns at world origin, moves with WASD/left-stick, sprints, jumps, respects 45° slope limit, all parameters driven by `locomotion.ron`.

**Independent Test**: Launch game → white capsule appears at (0, 2, 0) → WASD moves it → Space jumps → Shift sprints → walking into a 50° wall is blocked → no terrain clipping. See [quickstart.md Steps 1–4](quickstart.md).

- [X] T015 [P] [US1] Write `assets/player/locomotion.ron` with all `PlayerLocomotionConfig` fields: `walk_speed_m_s: 5.0`, `sprint_speed_m_s: 10.0`, `jump_impulse_m_s: 6.0`, `gravity_m_s2: -20.0`, `max_slope_angle_deg: 45.0`, `interact_radius_m: 3.0`, `capsule_radius_m: 0.35`, `capsule_height_m: 1.8`, `swim_speed_m_s: 3.0`, `sea_level_m: 0.0`
- [X] T016 [US1] Implement `spawn_player` system in `src/plugins/traversal/systems/spawn.rs` — reads spawn coordinate from `PlanetConfig.spawn_position: DVec3` (falls back to `DVec3::new(0.0, 2.0, 0.0)` with a `warn!` if `PlanetConfig` is absent); spawns `PlayerBundle` at that `GlobalPosition` using `PlayerLocomotionConfig` capsule dimensions; `RigidBody::Kinematic`, `LockedAxes::ROTATION_LOCKED`, `GravityScale(0.0)` (manual gravity in locomotion system); no position may be hardcoded without a config fallback — covers FR-001
- [X] T017 [US1] Implement `poll_on_foot_input` in `src/plugins/traversal/systems/input.rs` — reads `ButtonInput<GamepadButton>`, `ButtonInput<KeyCode>`, `Axis<GamepadAxis>`, `InputConfig`; applies deadzone; writes `OnFootInputState` each `Update` frame; queries `Without<PassengerOf>`
- [X] T018 [US1] Implement `locomotion_system` in `src/plugins/traversal/systems/locomotion.rs` — `MoveAndSlide` SystemParam; accumulates `vertical_velocity` with gravity; applies `jump_impulse` on `jump_pressed` (guarded by `jumped_this_frame`); calls `move_and_slide()` with `max_slope_angle`; adds `output.translation.as_dvec3()` to `GlobalPosition`; runs `FixedUpdate` before `PhysicsSet::StepSimulation`
- [X] T018a [US1] Add `swim_speed_m_s: f32` and `sea_level_m: f32` to `PlayerLocomotionConfig` in `src/plugins/traversal/config.rs` (fields already declared in `data-model.md §2`); update T015's `locomotion.ron` values to include them. Covers FR-005.
- [X] T018b [US1] Implement swimming state in `locomotion_system` (`src/plugins/traversal/systems/locomotion.rs`): check `global_pos.y < sea_level_m + capsule_half_height`; set `PlayerLocomotionState::is_swimming = true`; apply upward buoyancy force `ρgV` (ρ = 1025.0 kg/m³, V = capsule_radius² × π × capsule_height); replace walk/sprint speed with `swim_speed_m_s`-capped horizontal force; cap `vertical_velocity` at 0.0 to prevent sinking. Covers FR-004.
- [X] T019 [US1] Implement `camera_follow_system` in `src/plugins/traversal/systems/camera_follow.rs` — queries player `GlobalPosition`; writes camera entity `GlobalPosition` to player pos + `DVec3::new(0.0, 1.5, 3.0)` (behind and above); runs `PostUpdate` before `TransformSystem::TransformPropagate`
- [X] T020 [US1] Register US1 systems in `src/plugins/traversal/mod.rs` — `Startup`: `spawn_player.after(startup_load_configs)`; `Update`: `poll_on_foot_input`; `FixedUpdate`: `locomotion_system.before(PhysicsSet::StepSimulation)`; `PostUpdate`: `camera_follow_system.before(TransformSystem::TransformPropagate)`

**Checkpoint**: User Story 1 fully functional and independently testable. Player walks, runs, jumps; no terrain clipping; camera follows. Quickstart Steps 1–4 pass.

---

## Phase 4: User Story 2 — Ground Vehicle Entry, Driving, Exit (Priority: P2)

**Goal**: Player walks to a parked car/truck within 3 m, enters it, drives across terrain with raycast suspension, and exits cleanly beside the vehicle.

**Independent Test**: Spawn car at (10, 0.5, 0) → walk player within 3 m → press F/Triangle → player disappears, car accepts input → drive 100 m over a hill → press F again → player re-appears 2.5 m in front of car. See [quickstart.md Steps 5–7](quickstart.md).

- [X] T021 [P] [US2] Write `assets/vehicles/car.ron` — `VehicleConfig` with `GroundVehicleConfig`: `mass_kg: 1200.0`, `max_speed_m_s: 30.0`, `engine_force_n: 5000.0`, `brake_force_n: 8000.0`, `max_steer_angle_deg: 35.0`, `impact_threshold_m_s: 20.0`, 4 `WheelConfig` entries at ±1.0 m lateral / ±1.7 m fore-aft
- [X] T022 [P] [US2] Write `assets/vehicles/truck.ron` — `VehicleConfig` with `GroundVehicleConfig`: `mass_kg: 3000.0`, `max_speed_m_s: 20.0`, `engine_force_n: 9000.0`, `brake_force_n: 12000.0`, `max_steer_angle_deg: 28.0`, `impact_threshold_m_s: 20.0`, 4 `WheelConfig` entries
- [X] T023 [US2] Update `assets/vehicles/world_spawns.ron` — add entries for `car.ron` at `(10.0, 0.5, 0.0)` and `truck.ron` at `(25.0, 0.5, 0.0)`
- [X] T024 [US2] Implement `startup_spawn_vehicles` in `src/plugins/traversal/systems/spawn.rs` — iterates `WorldSpawnList`; looks up each `config` in `VehicleRegistry`; spawns appropriate bundle (`GroundVehicleBundle`, etc.) with `GlobalPosition` from entry; logs `warn!` for unknown config names; registers in `PostStartup`
- [X] T025 [US2] Implement `poll_vehicle_input` in `src/plugins/traversal/systems/input.rs` — reads same `InputConfig` sources; writes `VehicleInputState` on the vehicle entity tracked by `CurrentVehicle`; queries `With<PassengerOf>`; clamps throttle/brake to `[0.0, 1.0]`, steer/pitch/roll to `[-1.0, 1.0]`
- [X] T026 [US2] Implement the four enter/exit systems in `src/plugins/traversal/systems/enter_exit.rs`: `update_enter_exit_prompts` (proximity scan using `GlobalPosition` distance vs `interact_radius_m`); `enter_exit_vehicle` (emits `EnterVehicleEvent` or `ExitVehicleEvent`); `apply_enter_vehicle` (inserts `PassengerOf`, `Visibility::Hidden`, `GravityScale(0.0)`, `OccupiedBy`); `apply_exit_vehicle` (removes components, places player at vehicle pos + `DVec3::new(0.0, 0.5, 2.5)`, clears `CurrentVehicle`)
- [X] T026a [US2] Implement enter/exit UI hint in `src/plugins/traversal/systems/enter_exit.rs` — when `EnterExitProximity.nearby_players` is non-empty, spawn or update a `bevy_ui::Text` entity tagged `EnterExitHintMarker` with text "[F] / [Triangle] Enter"; despawn when `nearby_players` empties. Covers FR-006 (prompt MUST be displayed).
- [X] T027 [US2] Implement `ground_vehicle_system` in `src/plugins/traversal/systems/ground_vehicle.rs` — per-wheel `SpatialQuery::cast_ray` downward; spring/damper `Forces::apply_linear_impulse_at_point`; drive force and brake via `Forces::apply_force`; yaw steering via `Forces::apply_torque`; runs `FixedUpdate` before `PhysicsSet::StepSimulation`
- [X] T028 [US2] Implement `impact_detection_system` in `src/plugins/traversal/systems/impact.rs` — caches `LinearVelocity` magnitude before `PhysicsSet::StepSimulation`; reads `impact_threshold_m_s` from the entity's config component (`GroundVehicleConfig`, `HelicopterConfig`, or `FixedWingConfig` — never hardcoded); if delta exceeds threshold, emits `ImpactEvent { entity, impact_speed_m_s: delta, contact_normal }` and sets `LinearVelocity` to zero; runs `FixedUpdate` after `PhysicsSet::StepSimulation`
- [X] T029 [US2] Register US2 systems in `src/plugins/traversal/mod.rs` — `PostStartup`: `startup_spawn_vehicles`; `Update`: `poll_vehicle_input`, `update_enter_exit_prompts.after(poll_on_foot_input)`, `enter_exit_vehicle`, `apply_enter_vehicle.after(enter_exit_vehicle)`, `apply_exit_vehicle.after(enter_exit_vehicle)`; `FixedUpdate`: `ground_vehicle_system.before(PhysicsSet::StepSimulation)`, `impact_detection_system.after(PhysicsSet::StepSimulation)`

**Checkpoint**: User Story 2 fully functional. Player can enter and exit a car, drive over terrain, experience suspension, and be stopped by an impact. Quickstart Steps 5–7 pass.

---

## Phase 5: User Story 3 — Watercraft Traversal (Priority: P3)

**Goal**: Player enters a motorboat at sea level, it floats via manual buoyancy, moves with throttle/steer, and stops at a shoreline without tunnelling.

**Independent Test**: Spawn motorboat at (0, 0, 50) → enter → R2/W throttle → boat moves forward → A/D steers → exit at sea → player floats (swimming placeholder ok). See [quickstart.md Step 8](quickstart.md).

- [X] T030 [P] [US3] Write `assets/vehicles/motorboat.ron` — `VehicleConfig` with `WatercraftConfig`: `mass_kg: 800.0`, `max_speed_m_s: 15.0`, `thrust_force_n: 3000.0`, `turn_torque_n_m: 1500.0`, `buoyancy_volume_m3: 4.0`, `water_drag: 150.0`, `sea_level_m: 0.0`
- [X] T031 [US3] Add motorboat entry to `assets/vehicles/world_spawns.ron`: `VehicleSpawnEntry(config: "motorboat.ron", position: (0.0, 0.0, 50.0), yaw_deg: None)`
- [X] T032 [US3] Implement `watercraft_system` in `src/plugins/traversal/systems/watercraft.rs` — reads `GlobalPosition.0.y` vs `sea_level_m`; computes `depth.clamp(0.0, 1.0)` submerged fraction; applies `Forces::apply_force(Vec3::Y * buoyancy_force)`; applies water drag `-linear_velocity * water_drag`; applies forward thrust capped at `max_speed_m_s`; applies yaw torque; runs `FixedUpdate` before `PhysicsSet::StepSimulation`
- [X] T033 [US3] Register `watercraft_system` in `src/plugins/traversal/mod.rs` under `FixedUpdate` before `PhysicsSet::StepSimulation`

**Checkpoint**: User Story 3 fully functional. Boat floats at sea level, moves with input, stops at shoreline. Quickstart Step 8 passes.

---

## Phase 6: User Story 4 — Flying Vehicle Traversal (Priority: P4)

**Goal**: Player enters a helicopter or airplane, takes off, flies at altitude, and lands — all at ≥ 60 FPS.

**Independent Test**: Spawn helicopter at (-20, 1, -20) → enter → R2 throttle past 0.5 → lifts off → left stick tilts/translates → reduce throttle → descends and lands. Spawn airplane at (100, 0.5, 0) → full throttle → reaches stall_speed → pitch up → climbs. See [quickstart.md Steps 9–10](quickstart.md).

- [X] T034 [P] [US4] Write `assets/vehicles/helicopter.ron` — `HelicopterConfig`: `mass_kg: 2000.0`, `max_ascent_m_s: 15.0`, `max_lateral_m_s: 20.0`, `hover_throttle: 0.5`, `tilt_sensitivity: 15.0`, `yaw_torque_n_m: 3000.0`, `impact_threshold_m_s: 15.0`
- [X] T035 [P] [US4] Write `assets/vehicles/airplane.ron` — `FixedWingConfig`: `mass_kg: 3000.0`, `max_thrust_n: 12000.0`, `lift_coefficient: 1.2`, `drag_coefficient: 0.05`, `wing_area_m2: 16.0`, `stall_speed_m_s: 30.0`, `max_pitch_torque_n_m: 4000.0`, `max_roll_torque_n_m: 5000.0`, `max_yaw_torque_n_m: 2000.0`, `impact_threshold_m_s: 25.0`
- [X] T036 [US4] Add helicopter and airplane entries to `assets/vehicles/world_spawns.ron`: helicopter at `(-20.0, 1.0, -20.0)`, airplane at `(100.0, 0.5, 0.0)`
- [X] T037 [US4] Implement `helicopter_system` in `src/plugins/traversal/systems/aircraft.rs` — computes `hover_lift = mass * 9.81`; scales by `throttle / hover_throttle`; applies `local_up * lift` via `Forces::apply_force`; applies lateral tilt force from steer/pitch inputs × `tilt_sensitivity`; applies yaw torque; runs `FixedUpdate` before `PhysicsSet::StepSimulation`
- [X] T038 [US4] Implement `fixed_wing_system` in `src/plugins/traversal/systems/aircraft.rs` — computes `airspeed = lin_vel.length()`; applies thrust `forward * max_thrust * throttle`; if `airspeed >= stall_speed`: applies aerodynamic lift (`0.5 * 1.225 * v² * Cl * A`) along `local_up` and drag along `-forward`; applies pitch/roll/yaw torques; runs `FixedUpdate` before `PhysicsSet::StepSimulation`
- [X] T039 [US4] Register `helicopter_system` and `fixed_wing_system` in `src/plugins/traversal/mod.rs` under `FixedUpdate` before `PhysicsSet::StepSimulation`

**Checkpoint**: User Story 4 fully functional. Helicopter hovers and translates; fixed-wing lifts after reaching stall speed; both land without clipping; 60 FPS maintained. Quickstart Steps 9–10 pass.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: LOD integration, documentation, code quality gates, tests, and final validation.

- [X] T040 [P] Implement `vehicle_lod_system` in `src/plugins/traversal/systems/lod.rs` — queries `GlobalPosition` on all `VehicleTag` entities; computes distance to camera `GlobalPosition`; sets `Visibility::Hidden` if `dist > lod_settings.thresholds[2]`, else `Visibility::Inherited`; register in `mod.rs` under `Update`
- [X] T041 [P] Add `///` doc comments to every `pub` item in `src/plugins/traversal/` (all structs, enums, fields, system functions) — required by Principle V
- [X] T042 [P] Write unit tests in `src/plugins/traversal/config.rs` (or `tests/` module): `test_locomotion_config_deserialize` (locomotion.ron round-trips), `test_vehicle_config_car` (car.ron → 4 wheels), `test_vehicle_config_motorboat` (sea_level_m = 0.0), `test_vehicle_config_helicopter`, `test_vehicle_config_airplane`
- [X] T043 [P] Write unit/property tests in `src/plugins/traversal/systems/` — `test_buoyancy_zero_depth` (force = 0 when depth ≤ 0), `test_buoyancy_full_submersion` (force = ρgV), `test_fixed_wing_stall` (lift = 0 below stall speed), `test_impact_clamp` (velocity → zero above threshold), `test_slope_blocked` (no forward motion on 50° surface), `test_enter_exit_components` (PassengerOf/OccupiedBy added and removed correctly)
- [X] T044 Run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings`; fix all reported warnings to achieve clean clippy output
- [X] T045 Execute all 12 steps in [`quickstart.md`](quickstart.md) manually; confirm each pass criterion; record any failures as follow-up issues

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 completion — **BLOCKS all user stories**
- **Phases 3–6 (User Stories)**: All depend on Phase 2; can proceed sequentially P1→P2→P3→P4 or in parallel if staffed
- **Phase 7 (Polish)**: Depends on all desired user story phases being complete

### User Story Dependencies

| Story | Depends On | Notes |
|---|---|---|
| US1 (P1) | Phase 2 | No dependency on other stories — pure on-foot |
| US2 (P2) | Phase 2 | Reuses `startup_load_configs` from Phase 2; independently testable |
| US3 (P3) | Phase 2 | Shares `enter_exit.rs` infrastructure introduced in US2 (T026) — implement US2 first or extract enter/exit to foundational |
| US4 (P4) | Phase 2 | Same enter/exit reuse as US3 — implement US2 before US4 |

> **Note**: US3 and US4 reuse the enter/exit systems built in US2 (T026). The safest sequencing is Phase 2 → US1 → US2 → US3 → US4. If parallelising, promote T026 enter/exit into Phase 2.

### Within Each User Story

- RON asset files (marked [P]) can be written in parallel with infrastructure tasks
- Components → resources → systems → registration (sequential within story)
- Register systems last in `mod.rs` after all system fns compile

---

## Parallel Execution Examples

### Phase 2 — Foundational
```
Can run in parallel:
  T009  Implement components.rs
  T010  Implement resources.rs
  T011  Implement events.rs
  T013  Extend InputConfig

Then sequential:
  T008  Implement config.rs           (no deps)
  T012  startup_load_configs          (after T008, T010)
  T014  Wire TraversalPlugin::build   (after T009–T013)
```

### Phase 3 — User Story 1
```
Can run in parallel:
  T015  Write locomotion.ron

Then sequential:
  T016  spawn_player
  T017  poll_on_foot_input
  T018  locomotion_system
  T019  camera_follow_system
  T020  Register US1 in mod.rs
```

### Phase 4 — User Story 2
```
Can run in parallel:
  T021  car.ron
  T022  truck.ron

Then sequential:
  T023  world_spawns.ron (car + truck entries)
  T024  startup_spawn_vehicles
  T025  poll_vehicle_input
  T026  enter_exit.rs systems
  T027  ground_vehicle_system
  T028  impact_detection_system
  T029  Register US2 in mod.rs
```

### Phase 7 — Polish
```
Can run in parallel:
  T040  vehicle_lod_system
  T041  Documentation
  T042  Unit tests
  T043  Property tests

Then sequential:
  T044  cargo fmt + clippy
  T045  Quickstart validation
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (**CRITICAL — blocks all stories**)
3. Complete Phase 3: User Story 1 (T015–T020)
4. **STOP and VALIDATE**: Quickstart Steps 1–4
5. Playable walking/jumping on voxel terrain — ship this as internal MVP

### Incremental Delivery

| Milestone | Phases | What ships |
|---|---|---|
| Internal MVP | 1 + 2 + 3 | Walk, run, jump, slope gating |
| Land Vehicles | + 4 | Enter/exit + drive cars/trucks |
| Ocean Crossing | + 5 | Motorboat buoyancy + water travel |
| Full Traversal | + 6 | Helicopters + fixed-wing aircraft |
| Production | + 7 | LOD, docs, tests, clippy clean |

---

## Summary

| Phase | Tasks | Parallel Opportunities |
|---|---|---|
| Phase 1: Setup | T001–T007 (7 tasks) | T002–T007 all parallelisable |
| Phase 2: Foundational | T008–T014 (7 tasks) | T009, T010, T011, T013 parallelisable |
| Phase 3: US1 On-Foot | T015–T020 (6 tasks) | T015 parallelisable |
| Phase 4: US2 Ground Vehicle | T021–T029 (9 tasks) | T021, T022 parallelisable |
| Phase 5: US3 Watercraft | T030–T033 (4 tasks) | T030 parallelisable |
| Phase 6: US4 Aircraft | T034–T039 (6 tasks) | T034, T035 parallelisable |
| Phase 7: Polish | T040–T045 (6 tasks) | T040, T041, T042, T043 parallelisable |
| **Total** | **45 tasks** | **15 parallel opportunities** |

**Suggested MVP scope**: Phases 1 + 2 + 3 = **20 tasks** → fully playable on-foot traversal.
