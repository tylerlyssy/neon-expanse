# Feature Specification: Core Foundation

**Feature Branch**: `001-core-foundation`
**Created**: 2026-02-20
**Status**: Draft
**Input**: User description: "Create the absolute foundational core of the game — permanent skeleton that launches in borderless fullscreen, proves large-scale floating-origin stability with a single test entity at planetary distance, and registers all mandatory core plugins ready for every subsequent feature to build upon."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Stable Launch Experience (Priority: P1)

A developer (or player in a future build) starts the game for the first time. The window opens immediately in borderless fullscreen at native resolution. The window title reads "Neon Expanse v0.0.1-dev". Two messages appear in the console within the first second confirming the engine is alive: one that the core is initialized, and one identifying whether a PS4/PS5 controller is connected or keyboard input is active. The application runs stably and closes cleanly without any crashes or error output.

**Why this priority**: Without a stable, clean launch the entire project has no foundation. Every subsequent feature depends on this working perfectly. This is the first thing any contributor or tester will experience.

**Independent Test**: Run `cargo run`. Verify fullscreen window opens with correct title, two expected console messages appear within one second, and the process exits cleanly when the window is closed with no panics or error codes.

**Acceptance Scenarios**:

1. **Given** the compiled binary is launched, **When** the application starts, **Then** a borderless fullscreen window opens on the primary monitor at native resolution within two seconds.
2. **Given** the application is running, **When** the first frame is rendered, **Then** the window title bar (where visible) and OS task-switcher show "Neon Expanse v0.0.1-dev".
3. **Given** a PS4 or PS5 controller is connected before launch, **When** the application starts, **Then** the console prints a message containing "PS4/PS5 controller connected".
4. **Given** no controller is connected, **When** the application starts, **Then** the console prints a message confirming keyboard/mouse fallback is active.
5. **Given** the application is running, **When** the user closes the window, **Then** the process exits with code 0 and no panic messages appear in the console.

---

### User Story 2 — Planetary-Scale Origin Stability (Priority: P1)

A developer needs proof that the engine can place and track an entity at ~6,000 km from the world origin without any floating-point jitter, incorrect positioning, or visual artifacts. A single visible test marker (a colored sphere or equivalent) is visible on screen at that planetary distance, placed using the floating-origin coordinate system, and remains perfectly still across frames.

**Why this priority**: The floating-origin system is the architectural cornerstone of the entire game. If it is broken or unstable at launch, every single voxel-world, traversal, and physics feature built on top of it will be corrupted. This must be proven correct at the foundation level.

**Independent Test**: Launch the application and observe the test marker visible on screen. Verify it does not drift, flicker, or jitter across at least 300 frames. Confirm via console or diagnostics that the entity's global position reads approximately 6,000,000 meters from world origin and the camera's global position reads approximately 6,000,100 meters (i.e., ~100 m away from the entity).

**Acceptance Scenarios**:

1. **Given** the application is running, **When** the first frame is rendered, **Then** a single visible test marker appears on screen (the camera starts ~100 m from the marker at planetary distance from world origin).
2. **Given** the test marker is rendered, **When** 300 consecutive frames are observed, **Then** the marker does not drift, flicker, jitter, or change position.
3. **Given** the floating-origin system is active, **When** the test entity's global position is read, **Then** it is approximately 6,000,000 m from world origin, and the camera's global position is approximately 6,000,100 m from world origin (i.e., ~100 m offset).
4. **Given** the floating-origin system is active, **When** the camera's local-space Transform is inspected, **Then** values remain within standard float precision range (no values approaching float max or exhibiting NaN).

---

### User Story 3 — Controller and Input Readiness (Priority: P2)

A developer testing the input layer connects a PS4 or PS5 controller after launch. Button presses produce readable, debounced action events in the console or diagnostics panel. The same actions are also triggerable with keyboard keys (configurable fallback). Stick inputs have correct dead-zone filtering so resting sticks produce zero output.

**Why this priority**: Input correctness underpins all traversal features. Getting dead zones and action mapping right at the foundation prevents breakage and rework when locomotion is added in the next spec.

**Independent Test**: With a controller connected, press the primary action button and observe a console action event logged. Tilt the left stick barely past a small threshold and release; confirm the event fires above threshold and stops below it. Repeat the same actions with keyboard keys.

**Acceptance Scenarios**:

1. **Given** a PS4/PS5 controller is connected, **When** any face button is pressed, **Then** a corresponding named action event is emitted and observable in diagnostics within one frame.
2. **Given** a controller is in use, **When** an analog stick is held within the dead-zone radius, **Then** the output action value reads exactly zero.
3. **Given** a controller is in use, **When** an analog stick is pushed beyond the dead-zone, **Then** a proportional non-zero action value is produced.
4. **Given** no controller is connected, **When** a mapped keyboard key is pressed, **Then** the same named action event is emitted as if the corresponding controller button were pressed.

---

### User Story 4 — Per-Frame Diagnostics Visibility (Priority: P3)

A developer running the application can see real-time frame-time, frame rate (FPS), and entity count either in the console output at a regular interval or via an always-available in-process profiling hook. When a profiling tool (such as Tracy or puffin) is connected externally, the application exposes frame timing data without crashing or degrading performance significantly.

**Why this priority**: Observable performance data is essential from day one for validating the 60+ FPS target and catching regressions early. However, the game is still functional without this visible, making it lower priority than launch stability and origin correctness.

**Independent Test**: Run the application for 10 seconds. Observe that frame-time or FPS figures appear in console output approximately once per second (at least 8 lines in 10 seconds), and that entity count is reported alongside each entry. Attach an external profiling tool and confirm the process does not crash.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** five seconds have elapsed, **Then** at least five lines of diagnostic output containing frame-time or FPS have been written to the console (one per second).
2. **Given** the application is running, **When** diagnostics output is observed, **Then** entity count is reported alongside frame timing.
3. **Given** a supported external profiler is attached, **When** the application runs, **Then** frame data is streamed and the application does not crash or degrade below 30 FPS.

---

### Edge Cases

- What happens when the primary monitor is not detected at launch? The application must fall back to windowed mode on any available display rather than crashing.
- What happens if no audio device is present? The application must launch successfully since there is no audio in this spec; no audio errors should surface.
- What happens if a controller is disconnected mid-session? The input system must gracefully fall back to keyboard without restarting.
- What happens when the floating-origin entity position is exactly at world origin (zero)? The system must still function correctly without division-by-zero or degenerate cases.
- What happens if `input.ron` is absent on first launch? The application MUST apply hardcoded default bindings and dead-zone values without prompting the user or crashing; a default `input.ron` MUST ship alongside the binary.

## Clarifications

### Session 2026-02-20

- Q: How are input action mappings and dead-zone thresholds stored and made configurable? → A: RON config file (`input.ron`) loaded at startup; stores action→button bindings and dead-zone values; user/mod-editable without recompiling.
- Q: Where is the camera placed at startup relative to the test entity and world origin? → A: Camera's initial global position is placed ~100 m from the test entity (~6,000,000 m from world origin); entity's local transform is a tiny f32-safe offset; proves the DVec3→f32 sync pipeline at planetary coordinates.
- Q: What fixed-timestep rate should the physics/traversal schedule run at? → A: 64 Hz (every ~15.6 ms) — matches Avian3d default, deterministic, smooth vehicle behaviour, low overhead; configurable as a single constant without architectural change.
- Q: Is the Physics plugin fully initialized or an empty stub in this feature? → A: Physics fully initialized (broadphase + gravity active); test entity is explicitly marked as static / no-collision so it produces zero positional drift; proves the plugin runs without panic at no simulation cost.
- Q: What is the canonical diagnostic log frequency — once per second or once per 5 seconds? → A: Once per second (1 Hz) — matches User Story 4 acceptance scenario; standard for real-time developer diagnostics; negligible CPU cost.

## Requirements *(mandatory)*

### Functional Requirements

**Launch & Window**

- **FR-001**: The application MUST launch directly into a borderless fullscreen window on the primary monitor at native resolution, with no intermediate menu or loading screen.
- **FR-002**: The window title MUST display "Neon Expanse v0.0.1-dev" at all times while the application is running.
- **FR-003**: The application MUST print "Neon Expanse core initialized" to the console within the first rendered frame.
- **FR-004**: The application MUST exit cleanly with exit code 0 when the window is closed; no panics, no error output, no hanging processes.
- **FR-005**: If the primary monitor cannot be detected, the application MUST fall back to a windowed mode on any available display rather than crashing.

**Plugin Architecture**

- **FR-006**: All mandatory core plugins (Core, Window, Input, FloatingOrigin, LOD, Physics, and stub plugins for VoxelWorld and Traversal) MUST be registered and initialized in dependency order before the first frame.
- **FR-007**: Each plugin MUST be implemented as a self-contained, independently registerable unit — no plugin may directly call into another plugin's internal systems.
- **FR-008**: VoxelWorld and Traversal plugins MUST exist as empty, registered stubs with no functional systems so subsequent features can be added without touching the plugin registry.

**Floating Origin & Large-Scale Coordinates**

- **FR-009**: The world MUST support high-precision global coordinates capable of representing positions across planetary distances (at minimum ~12,742 km diameter) without observable positional imprecision, at a scale of 1 unit = 1 meter.
- **FR-010**: All entities with a global position MUST have their local render-space transform automatically kept in sync with the floating origin every frame — no manual sync code outside the FloatingOrigin plugin.
- **FR-011**: A single test entity (visible marker) MUST be spawned at approximately 6,000,000 m from the world origin, with the camera spawned ~100 m away from the entity (also at ~6,000,000 m from world origin), so that the floating-origin DVec3→f32 sync pipeline is exercised at full planetary scale.
- **FR-012**: The test entity MUST remain visually stable (no jitter, no drift, no NaN values) across all rendered frames for the entire session.

**Input**

- **FR-013**: The input system MUST auto-detect a connected PS4 or PS5 controller on startup without any user configuration.
- **FR-014**: Upon controller detection, the console MUST print a message containing "PS4/PS5 controller connected".
- **FR-015**: If no controller is detected, the console MUST print a message confirming keyboard/mouse fallback is active.
- **FR-016**: Analog stick inputs MUST apply dead-zone filtering with thresholds loaded from `input.ron` at startup; inputs within the dead-zone MUST produce a zero action value; the thresholds MUST be user-editable in `input.ron` without recompiling.
- **FR-017**: The input system MUST load action-to-button/key mappings from `input.ron` at startup; the same named actions MUST be triggerable via controller or keyboard fallback as defined in the RON file; mappings MUST be user/mod-editable without recompiling.
- **FR-018**: If a controller is disconnected during a session, the input system MUST gracefully switch to keyboard/mouse fallback without requiring a restart.

**Scheduling & Timing**

- **FR-019**: A fixed-timestep schedule running at 64 Hz (one step every ~15.625 ms) MUST be registered and operational, ready to accept physics and traversal systems in later features. The rate MUST be expressed as a single named constant so it can be adjusted without structural changes.
- **FR-020**: Variable-timestep (per-frame) and fixed-timestep schedules MUST coexist without interfering with each other.

**Diagnostics & Profiling**

- **FR-021**: Frame-time and FPS diagnostics MUST be logged to the console at a rate of once per second (1 Hz) throughout the session.
- **FR-022**: Entity count MUST be included in the diagnostic output alongside frame timing.
- **FR-023**: The application MUST expose profiling data to externally attached profiling tools without crashing or degrading to fewer than 30 FPS.

**LOD System**

- **FR-024**: An LOD (Level of Detail) system with at least two configurable distance thresholds MUST be registered and operational, even though no LOD transitions are observable until terrain geometry is introduced in the next feature.
- **FR-025**: The test entity (origin marker) MUST be spawned with an explicit static / non-dynamic physics marker so the active physics simulation never applies gravity or forces to it, ensuring zero positional drift and proving that static scene objects coexist correctly with a live physics world.

### Key Entities

- **Test Origin Marker**: A single visible entity (e.g., colored sphere or billboard) placed at a global position of approximately 6,000,000 m from world origin. The camera is spawned ~100 m from this entity (also at planetary distance), so the entity's local render-space offset is tiny and f32-safe — this specifically exercises the DVec3→f32 sync at extreme coordinates. Carries no game logic; marked explicitly as static/non-dynamic so the active physics world never moves it. Exists solely as a floating-origin correctness proof.
- **Named Input Action**: A logical named action (e.g., "MoveForward", "PrimaryAction") mapped from one or more physical inputs (controller buttons/axes, keyboard keys). The action carries a scalar or boolean value after dead-zone processing.
- **LOD Threshold**: A distance value (in meters) defining the boundary between two detail levels. At least two thresholds must be configurable without source changes.
- **InputConfig**: A configuration record loaded from `input.ron` on startup, containing the full action-to-input mapping table and per-axis dead-zone thresholds. Ships with a default `input.ron`; if the file is absent, hardcoded defaults are used. Moddable — third-party mods may supply their own `input.ron`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The application window is visible and in borderless fullscreen within 2 seconds of launching the binary on target hardware (Ryzen 5 / RTX 3060 class).
- **SC-002**: Both expected console messages ("core initialized" and controller/keyboard status) appear within 1 second of the window opening.
- **SC-003**: The test entity at ~6,000 km is visible on screen and shows zero positional drift across 300 consecutive frames (verifiable by inspecting transform values).
- **SC-004**: Pressing any mapped controller input within 100ms of detection produces an action event in diagnostics; the same is true for keyboard fallback.
- **SC-005**: Frame rate on target hardware stays at or above 60 FPS with only the test entity and no terrain present.
- **SC-006**: Disconnecting and reconnecting a controller mid-session does not drop frame rate below 30 FPS or produce any console error output.
- **SC-007**: All mandatory core plugins are registered, the 64 Hz fixed-timestep schedule is active, and the application exits cleanly (code 0) with no warnings escalated to errors and no panics.
- **SC-008**: After this feature is complete, the next feature ("Procedural Voxel Planet Engine") can be started and built upon this foundation without requiring any structural changes to the files established by this feature.

## Assumptions

- The operating system correctly reports the primary monitor and native resolution; abnormal multi-monitor or virtual display setups are not in scope.
- "Borderless fullscreen" means the window covers the entire primary monitor with no OS window chrome; true exclusive fullscreen is not required.
- The test entity does not need to be aesthetically polished — a solid-colored sphere or debug marker is sufficient.
- LOD thresholds are numerical constants stored in a configuration-friendly location; the exact values will be refined in the voxel terrain spec.
- No audio system is needed; audio-related errors must not surface at launch.
- The "PS4/PS5 controller" detection relies on the controller being connected via USB or recognized Bluetooth profile; exotic third-party controller compatibility is out of scope.
- A default `input.ron` file ships alongside the binary; the application falls back to hardcoded defaults if the file is absent and never requires the user to create the file manually.
- The physics world is initialized with default gravity active from the first frame; this is intentional — the test entity is explicitly static so it is unaffected, and future features inherit a production-ready physics world.

## Out of Scope

- Any voxel terrain, procedural generation, or mesh rendering.
- Player character, locomotion, or any form of movement.
- Ground vehicles, watercraft, or aircraft.
- Any in-game UI, menus, or HUD.
- Networking or multiplayer infrastructure.
- Save/load systems.
- Audio.

## Dependencies

- This feature has no predecessor feature dependencies.
- All subsequent features (Voxel Planet Engine, Traversal, Vehicles) depend on this feature being complete and its plugins never requiring structural changes again.

