# Feature Specification: Basic Traversal System

**Feature Branch**: `003-basic-traversal`
**Created**: 2026-02-24
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 — On-Foot Locomotion (Priority: P1)

The player spawns on the voxel planet surface in a varied starting biome and can immediately move around on foot with the PS4/PS5 controller (or keyboard). Walking, running, jumping, and basic slope climbing all work out of the box against the voxel terrain collision surface. Entering water switches to swimming mode.

**Why this priority**: Every other traversal mode depends on the player being an entity that exists in the world with a working character controller. On-foot movement is the most frequently used traversal mode and is the foundation all other user stories build on.

**Independent Test**: Launch the game, spawn on the planet surface, walk in all four directions, run, jump off a ledge, land on a slope without falling through. Walk into a body of water and confirm swimming mode activates. Delivers a fully playable, if limited, game immediately.

**Acceptance Scenarios**:

1. **Given** the game has just launched, **When** the startup sequence completes, **Then** the player entity exists at a valid surface position in a varied biome (not blank/debug geometry).
2. **Given** the player is on flat ground, **When** the left stick is pushed forward, **Then** the player moves forward at walking speed (~5 m/s) and stays flush with the ground.
3. **Given** the player is walking, **When** the Sprint button is held, **Then** speed increases to running pace (~10 m/s).
4. **Given** the player is on the ground, **When** the Jump button is pressed, **Then** the player launches upward and returns to the surface within 2 s.
5. **Given** the player is on a slope less than 45°, **When** moving up it, **Then** the player climbs naturally without sliding back or clipping through.
6. **Given** the player walks off a ledge, **When** airborne, **Then** gravity pulls them down at a realistic rate and they land on the next surface without bouncing into space.
7. **Given** the player enters a body of water more than 50% capsule depth, **When** moving, **Then** swimming controls activate and buoyancy keeps them at the surface.

---

### User Story 2 — Ground Vehicle Entry, Driving, and Exit (Priority: P2)

The player approaches a parked car or off-road truck, sees an enter prompt, boards it, drives across varied terrain including hills and rough patches, and exits cleanly.

**Why this priority**: Ground vehicles are the primary mid-range traversal mode. They validate the enter/exit lifecycle, wheel physics on voxel terrain colliders, and the data-driven vehicle config pipeline that watercraft and aircraft reuse.

**Independent Test**: Spawn a car 20 m from the player, walk to it, enter, drive 500 m over terrain with at least one hill, exit, watch the car remain stationary. Delivers playable land exploration independently.

**Acceptance Scenarios**:

1. **Given** the player is within 3 m of a ground vehicle, **When** no button is pressed, **Then** an enter/exit prompt is visible on screen.
2. **Given** the enter prompt is showing, **When** the Enter/Exit button is pressed, **Then** the player attaches to the vehicle, the character mesh is hidden, and the vehicle accepts driver input.
3. **Given** the player is driving, **When** the right trigger is held, **Then** the vehicle accelerates to its configured top speed.
4. **Given** the vehicle is at speed, **When** the left trigger is pressed, **Then** the vehicle decelerates; held braking reverses slowly.
5. **Given** the left stick is steered while moving, **Then** the vehicle turns proportionally.
6. **Given** the vehicle drives over a hill at speed, **Then** it crests naturally and stays on the surface on descent.
7. **Given** the player presses Enter/Exit while inside, **Then** the player is placed beside the vehicle at a safe eject position; vehicle physics continue independently.
8. **Given** the vehicle hits a solid voxel wall at high speed, **Then** it stops (no tunnelling); an `ImpactEvent` is emitted.

---

### User Story 3 — Watercraft Traversal (Priority: P3)

The player enters a motorboat or small ship, motors across open water with correct buoyancy and steering, and exits at a shoreline.

**Why this priority**: Water covers a substantial portion of an Earth-like planet. Watercraft make ocean crossing viable and validate the buoyancy physics path.

**Independent Test**: Spawn a motorboat at a shoreline, enter, motor 200 m, turn, return, exit. No sinking, no tunnelling through the seabed.

**Acceptance Scenarios**:

1. **Given** a motorboat is on the ocean surface with no input, **When** frames pass, **Then** it floats at the water line and bobs gently.
2. **Given** the player applies forward throttle, **Then** the boat accelerates along its heading.
3. **Given** the boat approaches a shoreline at speed, **When** it runs aground, **Then** it stops without tunnelling.
4. **Given** the player exits the boat at sea, **Then** swimming mode activates immediately.

---

### User Story 4 — Flying Vehicle Traversal (Priority: P4)

The player enters a helicopter or airplane, takes off from any reasonably flat surface, flies at altitude over the planet, lands, and exits.

**Why this priority**: Aircraft provide the fastest global-scale traversal and deliver the "fly over mountains" experience central to the project vision. Sequenced last because stable ground and water traversal are prerequisites.

**Independent Test**: Spawn a helicopter on flat terrain, enter, take off vertically, fly 1 km, land, exit. Framerate stays 60+ FPS throughout.

**Acceptance Scenarios**:

1. **Given** a helicopter is on the ground with the player aboard and throttle is increased past the hover threshold, **Then** it lifts off and hovers stably.
2. **Given** the helicopter is hovering, **When** the left stick is tilted, **Then** it translates in the corresponding direction.
3. **Given** throttle is reduced below hover threshold, **Then** the helicopter descends at a controlled rate and lands on the surface.
4. **Given** an airplane is at full throttle and the player pitches up at sufficient speed, **Then** the aircraft lifts off and climbs.
5. **Given** any aircraft is at 10,000 m altitude, **Then** the planet is visible below, streaming continues, and framerate is ≥ 60 FPS.

---

### Edge Cases

- What happens when the player approaches the edge of a loaded chunk before the next chunk arrives? The player is held at the last valid surface contact position until terrain loads.
- What happens when a vehicle is spawned partially inside terrain? It is displaced upward to the nearest clear surface on spawn.
- What happens while the player is inside a vehicle and the chunk is evicted by the streaming system? The vehicle and player retain their `GlobalPosition`; they are never reset to `DVec3::ZERO`.
- What happens at a coastline when the terrain surface is at sea level? The player transitions to swimming the moment the capsule is more than 50% submerged.
- What happens if the player enters a vehicle while mid-air? Entry is permitted; the player snaps to the vehicle seat transform.
- What happens if a ground vehicle reaches a vertical rock face? It stops on contact; it does not climb walls.
- What happens if the player attempts to enter a second vehicle while already occupying one? The action is ignored; a hint prompts the player to exit their current vehicle first.

---

## Requirements *(mandatory)*

### Functional Requirements

#### On-Foot Locomotion

- **FR-001**: The system MUST spawn the player entity at a valid terrain surface position derived from the world seed's designated spawn coordinate at startup.
- **FR-002**: The player character controller MUST implement walking (~5 m/s), running (~10 m/s), jumping (configurable impulse), and falling under gravity.
- **FR-003**: The player MUST maintain surface contact on slopes up to `max_walk_slope_degrees` from `PlayerLocomotionConfig`; surfaces steeper than this are impassable walls. Ledge-grab and mantling are explicitly out of scope for this feature.
- **FR-004**: The player MUST transition to swimming state when more than 50% of the capsule volume is below the configured sea-level height; swimming controls replace walking controls and buoyancy prevents sinking.
- **FR-005**: All locomotion parameters (walk speed, run speed, jump impulse, max slope angle, swim speed, capsule radius and height) MUST be read from `assets/player/locomotion.ron` at startup; hardcoding any value is forbidden.

#### Enter / Exit System

- **FR-006**: When the player is within `enter_radius_m` of a vehicle entity marked as `Interactable`, an enter/exit hint MUST be displayed in the UI.
- **FR-007**: Pressing the Enter/Exit button MUST immediately hide the player mesh, suspend player physics, record the occupancy in the `CurrentVehicle` resource, and begin routing driver input to the target vehicle.
- **FR-008**: Pressing Enter/Exit while inside a vehicle MUST re-enable player physics, clear `CurrentVehicle`, reveal the player mesh, and place the player at the vehicle's `exit_offset` — always in a terrain-clear position.
- **FR-009**: The system MUST reject a second entry attempt when `CurrentVehicle` is already populated.

#### Ground Vehicles

- **FR-010**: Ground vehicles MUST use a **raycast suspension** model: one downward ray per wheel measures terrain distance, and a configurable spring/damper force is applied to the vehicle rigid body. Wheel count, ray length, spring stiffness, and damper coefficient MUST be declared in the vehicle's RON file.
- **FR-011**: All ground vehicle physics parameters (mass, drive torque, top speed, wheel count, suspension stiffness, friction coefficients) MUST be declared in the vehicle's RON file.
- **FR-012**: A ground vehicle impacting an obstacle above `impact_threshold_m_s` MUST have its linear velocity clamped to zero and MUST emit an `ImpactEvent` (carrying entity, impact speed, contact normal). The vehicle is immediately driveable again; no health or disabled state is tracked in this feature.
- **FR-013**: Ground vehicles MUST interact with `Collider` components generated by `VoxelWorldPlugin`; no terrain-specific special-casing in vehicle code is permitted.

#### Watercraft

- **FR-014**: Watercraft MUST experience an upward buoyancy force proportional to their submerged volume, maintaining stable flotation at the configured `draft_m` waterline.
- **FR-015**: Watercraft propulsion MUST be a configurable forward thrust force capped at `max_water_speed_m_s`.
- **FR-016**: Watercraft MUST collide with voxel terrain colliders (seabed, coastlines) the same way ground vehicles do.

#### Flying Vehicles

- **FR-017**: Helicopters MUST receive a configurable vertical lift force that counters gravity when throttle is at or above `hover_throttle_fraction`; translational movement is produced by tilting the lift vector.
- **FR-018**: Fixed-wing aircraft MUST generate lift proportional to airspeed and a configured `lift_coefficient`; below `stall_speed_m_s` lift drops to zero.
- **FR-019**: All aircraft MUST collide with voxel terrain colliders and, on contact above `impact_threshold_m_s`, clamp linear velocity to zero and emit an `ImpactEvent`. No health or disabled state is tracked in this feature.

#### Data-Driven Configuration

- **FR-020**: Every vehicle type MUST be defined by a RON file in `assets/vehicles/` conforming to the `VehicleConfig` schema; adding a new vehicle requires only a new RON file plus a mesh asset — no Rust source changes.
- **FR-021**: `PlayerLocomotionConfig` MUST be loaded from `assets/player/locomotion.ron`; if the file is absent or malformed the system MUST fall back to built-in defaults and emit a `warn!`.
- **FR-022**: If any vehicle's RON file is absent or invalid, that vehicle is skipped with a `warn!`; the session continues with all remaining valid vehicles.

- **FR-027**: Input reading systems (gamepad axes, button presses) MUST run in `Update` and write their results to a per-entity `InputState` component each frame. Physics force application systems (suspension forces, thrust, buoyancy, character impulse) MUST run in `FixedUpdate` and consume `InputState`. No force application logic is permitted in `Update`.

#### Vehicle World Placement

- **FR-026**: Vehicle entities MUST be placed in the world by reading `assets/vehicles/world_spawns.ron` at startup; this file maps vehicle config RON filenames to `DVec3` global positions. No vehicle position is hardcoded in Rust. Absent or malformed entries are skipped with a `warn!`.

#### Integration Requirements

- **FR-023**: All traversal entities (player, vehicles) MUST carry `GlobalPosition(DVec3)` and MUST NOT write directly to `Transform::translation`; `FloatingOriginPlugin` owns that sync.
- **FR-024**: `TraversalPlugin` MUST register all systems in the correct Bevy schedule slots and MUST NOT modify `VoxelWorldPlugin`, `FloatingOriginPlugin`, or `PhysicsPlugin` internals.
- **FR-025**: Vehicles MUST integrate with `LodPlugin`; at LOD-3 range a vehicle MUST switch to a low-poly representation or be culled to preserve frame budget.

---

### Key Entities

- **PlayerEntity**: The player's physical capsule — `GlobalPosition`, `PlayerLocomotionState`, optional `PassengerOf(Entity)` component, character collider.
- **VehicleEntity**: Any drivable object — `VehicleConfigHandle`, `VehicleType` (Ground / Water / Air), `VehicleState` (Parked / Occupied), `RigidBody`, `GlobalPosition`, `Interactable` marker.
- **PlayerLocomotionConfig**: Resource loaded from RON — all on-foot movement parameters.
- **VehicleConfig**: Asset type loaded per-vehicle RON — physics params, enter radius, exit offset, vehicle type discriminant, impact threshold.
- **CurrentVehicle**: Single-entry resource (`Option<Entity>`) — which vehicle entity the player currently occupies.
- **ImpactEvent**: ECS event emitted on collision above impact threshold — carries entity ID, impact speed, and contact normal.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After launching the game the player is visible on a varied voxel planet surface within 5 seconds and can walk in any direction without falling through terrain.
- **SC-002**: The player can enter and exit all four vehicle categories (car, motorboat, airplane, helicopter) within a single session without a restart or panic.
- **SC-003**: Continuous traversal across all modes maintains 60+ FPS for at least 10 consecutive minutes without a stall, freeze, or visible terrain seam.
- **SC-004**: Adding a new vehicle type requires only a new RON file in `assets/vehicles/` and a mesh asset; zero Rust source changes are required.
- **SC-005**: No vehicle or player entity passes through voxel terrain colliders at any speed reachable through normal gameplay controls.
- **SC-006**: `Transform.translation` for any traversal entity stays within ± 1,000 m of `Vec3::ZERO` throughout any traversal session, verified by the existing floating-origin property tests.

---

## Clarifications

### Session 2026-02-24

- Q: What wheel suspension model should ground vehicles use — raycast or Avian3d revolute joints? → A: Raycast suspension (one ray per wheel, spring/damper force applied to the rigid body; params in RON).
- Q: How do vehicles get placed in the world — hardcoded spawn, RON spawn-list, or procedural placement? → A: RON spawn-list (`assets/vehicles/world_spawns.ron`) maps vehicle config filenames to global DVec3 positions; loaded by TraversalPlugin at startup.
- Q: Does "basic climbing" include ledge-grab/mantling on overhangs, or slope-angle gating only? → A: Slope-angle only; steeper than `max_walk_slope_degrees` is impassable. Ledge-grab/mantling is explicitly out of scope for this feature.
- Q: What happens to a vehicle after a crash impact — velocity-zero only, or a simple HP system? → A: Velocity-zero only; the vehicle's velocity is clamped to zero, an `ImpactEvent` is emitted, and the vehicle is immediately driveable again. No health points or disabled state in this feature.
- Q: Should traversal input reading and physics force application both run in `FixedUpdate`, or split across `Update` + `FixedUpdate`? → A: Input is read every frame in `Update` and written to an `InputState` component; all physics forces (suspension, thrust, buoyancy, character impulse) are applied in `FixedUpdate` by consuming that component.

## Assumptions

- Water surface detection uses a flat-plane approximation at the sea-level height from `PlanetConfig`; dynamic wave simulation is out of scope.
- Vehicle meshes are simple geometric primitives (boxes, cylinders) for MVP; artist-quality models are a follow-up deliverable.
- The spawn coordinate is the terrain surface point above the world-seed-deterministic equatorial position; a spawn-selection UI is out of scope.
- Multiplayer and networked vehicle state are out of scope; all state is single-player local.
- Avian3d's `MoveAndSlide` SystemParam (not a component) drives the player capsule; it is invoked inside `locomotion_system`. A separate custom character controller struct is not required.
