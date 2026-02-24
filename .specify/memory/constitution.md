<!--
SYNC IMPACT REPORT
==================
Version change: (template) → 1.0.0 (initial ratification)
Modified principles: N/A — first population from template placeholders
Added sections:
  - Core Principles (8 named principles: I. Project Vision, II. Architecture,
    III. Technical Stack, IV. Mandatory Plugins, V. Code Quality,
    VI. Testing & Quality, VII. User Experience, VIII. Modding & Extensibility)
  - Code Quality & Workflow Enforcement (§2, replaces SECTION_2 placeholder)
  - Governance (§3, populated from template skeleton + Evolution rules)
Removed sections: None (all template placeholders replaced)
Templates requiring updates:
  ✅ .specify/memory/constitution.md — this file (written 2026-02-19)
  ⚠  .specify/templates/plan-template.md — Constitution Check gates should cite
     the 8 principles by name; no structural change needed, filled at plan-time
  ⚠  .specify/templates/spec-template.md — verify no agent-specific references remain
  ⚠  .specify/templates/tasks-template.md — task categories should reflect
     observability/testing discipline from Principle VI
Follow-up TODOs: None — all placeholders resolved.
-->

# Neon Expanse Constitution

**Version**: 1.0.1 | **Ratified**: 2026-02-19 | **Last Amended**: 2026-02-20
**Project Name**: Neon Expanse
**Enforced by**: All `/speckit.*` commands, plans, tasks, code reviews, and PRs

## Core Principles

### I. Project Vision & Non-Negotiable Core Values

- The project MUST build a single, persistent, Earth-scale planet (~12,742 km diameter)
  that feels completely seamless and alive — modelled after Light No Fire's world engine.
- Core gameplay loop MUST be pure joyful traversal and exploration across all modes:
  - Player on foot (walking, running, jumping, swimming, basic climbing)
  - Ground vehicles (cars, trucks, off-road vehicles)
  - Watercraft (boats, ships, hovercraft)
  - Flying vehicles (planes, helicopters, jets, gliders, drones)
- The planet MUST feel like one continuous world — no loading screens, no visible seams,
  no instancing is permitted under any circumstance.
- Fun MUST take priority over realism when they conflict. Exploration MUST feel freeing
  and rewarding at all times.
- The game MUST be fully moddable from day one — players MUST be able to add new vehicles,
  biomes, or entire traversal modes with zero engine changes required.
- Accessibility is mandatory: color-blind modes, remappable controls, PS4/PS5 controller
  as the primary input (with keyboard/mouse fallback), and scalable UI MUST all ship.
- Performance target: 60+ FPS on mid-range hardware (Ryzen 5 / RTX 3060 class) while
  streaming voxel terrain, multiple vehicles, and physics across large distances.

*Rationale*: Every downstream decision — architecture, rendering strategy, input design —
flows from this vision. Weakening any item here degrades the core identity of the project.

### II. Architectural Principles (Immutable)

- The codebase MUST be 100% Bevy ECS. Classic OOP patterns for game objects are forbidden.
- There MUST be exactly one persistent planet: one shared world seed, fully deterministic.
- The world engine MUST be voxel-based, inspired by Light No Fire:
  - Dual marching cubes voxel meshing is REQUIRED for smooth terrain with overhangs,
    caves, and natural features.
  - Generation MUST be layered (base sphere → noise layers → erosion → biome overlays).
  - Chunk streaming, LOD, and floating origin (DVec3 global positions, 1 unit = 1 m)
    MUST be seamless and always active.
- Rendering MUST follow a hybrid strategy:
  - At distance / high altitude: cheap scaled-space representation.
  - At close range: full dynamic dual marching cubes voxel mesh.
- Traversal systems MUST be data-driven via `VehicleComponent`, `WatercraftComponent`,
  `AircraftComponent`, and `PlayerLocomotionComponent`.
- Every major feature MUST live in its own Bevy plugin.
- Core world systems (voxel engine, floating origin, LOD, streaming) MUST NOT be
  structurally modified after the foundation milestone is complete.

*Rationale*: Immutable architecture prevents accumulated technical debt that would make
the seamless-world guarantee impossible to maintain at scale.

### III. Technical Stack (Locked)

- Language: Rust stable, 2024 edition, minimum 1.85+.
- Engine: Bevy 0.18+ (latest stable at time of implementation).
- Physics: `avian3d = "0.5"` (or latest compatible) for characters, vehicles, boats,
  aircraft, and destruction.
- Voxel / Terrain:
  - Custom dual marching cubes mesher (no third-party replacement permitted).
  - Layered noise via the `fastnoise-lite` crate (v1.1, pure-Rust, cross-platform deterministic) plus erosion simulation. *(Amended 2026-02-20: replaced `noise` crate per Feature 002 Clarification Q1 — determinism guarantee required for golden snapshot CI tests.)*
- Coordinates: `glam::DVec3` for global positions; `Vec3` for local chunk-space.
- Input: Bevy Gamepad + custom action system (PS4/PS5 optimized with deadzones;
  keyboard/mouse fallback REQUIRED).
- UI: Bevy UI + `bevy_egui` for vehicle menus and maps.
- Serialization: `serde` + `ron` (all vehicles, biomes, and planet config in RON).
- Asset pipeline: Bevy asset system + custom voxel chunk streaming.
- Profiling: `bevy_diagnostic` + Tracy/puffin.
- `unsafe` is FORBIDDEN except in documented math kernels; each unsafe block MUST
  carry a `// SAFETY:` comment explaining the invariant.

*Rationale*: A locked stack eliminates dependency churn and ensures every contributor
works from the same known-good foundation.

### IV. Mandatory Core Plugins (Must Exist From Day One)

All of the following Bevy plugins MUST be present and operational before any feature
work begins:

- `CorePlugin` — App setup, schedules, logging, diagnostics.
- `WindowPlugin` — Borderless fullscreen on primary monitor by default;
  `PresentMode::AutoNoVsync`.
- `InputPlugin` — PS4/PS5 controller auto-detect + keyboard/mouse fallback.
- `FloatingOriginPlugin` — DVec3 global positions + automatic `Transform` sync.
- `VoxelWorldPlugin` — Dual marching cubes, chunk streaming, LOD, scaled-space switch.
- `TraversalPlugin` — Player locomotion, ground vehicles, watercraft, flying vehicles.
- `PhysicsPlugin` — Avian3d integration for all traversal types.

*Rationale*: Attempting to build features before the foundation plugins exist produces
rework. This gate enforces build-order discipline.

### V. Code Quality Standards

- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` MUST pass on every commit.
  CI MUST reject commits that fail either check.
- Every public item MUST have `///` documentation. Undocumented public APIs are a
  blocking review failure.
- `unwrap()` calls are FORBIDDEN outside `main.rs` and asset-loading bootstrapping.
  `expect()` with a descriptive message is acceptable where a panic is truly invariant.
- Every `plan.md` MUST begin with a Constitution Compliance Checklist referencing each
  applicable principle from this document.

*Rationale*: Consistent quality gates prevent entropy. Enforcing at commit time is
cheaper than fixing at review time.

### VI. Testing & Quality

- Property-based tests MUST cover voxel determinism, floating-origin stability, and
  vehicle physics edge cases.
- Minimum 85% line coverage is REQUIRED on core world and traversal systems.
- A determinism regression test MUST verify that the same world seed always produces
  identical terrain geometry and vehicle behavior across builds.

*Rationale*: Procedural and physics systems are highly sensitive to regressions.
Automated property tests catch drift that unit tests miss.

### VII. User Experience (Foundation)

- The game MUST launch directly into borderless fullscreen with no intermediate menu.
- PS4/PS5 controller MUST be auto-detected; a console message confirming detection
  MUST be printed on startup.
- The player MUST spawn on foot in a varied starting biome — no blank or debug terrain.
- All four traversal categories (foot, ground vehicle, watercraft, aircraft) MUST be
  accessible to the player within the first session.
- The world MUST feel massive and continuous — uninterrupted traversal for hours in
  any direction MUST be possible without repetition, seams, or loading pauses.

*Rationale*: First-impression quality is non-negotiable. A broken or bare launch
experience undermines confidence in the entire project.

### VIII. Modding & Extensibility

- All vehicle definitions, voxel layer configurations, and traversal rules MUST be
  exposed via RON files and Bevy reflection — no hardcoding permitted.
- Mods MUST be able to register new vehicle types, new voxel layers, or new locomotion
  modes entirely through Bevy plugins without modifying engine source.

*Rationale*: Moddability is a day-one commitment. Retrofitting it is exponentially
more expensive than designing for it from the start.

## Code Quality & Workflow Enforcement

- Every feature MUST begin with `/speckit.specify`, producing a player-focused,
  tech-agnostic specification before any implementation work starts.
- `/speckit.plan` output MUST include a Constitution Compliance Checklist as its
  first substantive section, verifying alignment with all eight core principles above.
- Any plan or PR that cannot demonstrate compliance with this constitution MUST be
  blocked from merge until compliance is established or a formal amendment is approved.
- Tooling commands (e.g., `/speckit.*`) MUST be written to be agent-agnostic —
  no command file may reference a specific AI agent by name as a hard requirement.

## Governance

- This constitution is the supreme governing document of Neon Expanse. No contributor,
  AI assistant, or automated tool may override it without following the amendment process.
- **Amendment process**: A proposed change MUST be accompanied by a dedicated spec,
  a plan with explicit justification, and a version bump following semver rules.
- **Versioning policy**:
  - MAJOR — backward-incompatible governance change: removal or redefinition of a
    principle that changes what work is permitted.
  - MINOR — new principle or section added, or materially expanded guidance.
  - PATCH — clarifications, wording fixes, or non-semantic refinements.
- **Compliance review**: Every PR MUST include a checklist confirming no principle
  from this constitution is violated. Violations are a hard merge block.
- All amendments MUST update `Last Amended` to the date of the change and increment
  `Version` appropriately.
- The Sync Impact Report (HTML comment at the top of this file) MUST be updated on
  every amendment to record what changed and which dependent templates were affected.

**Signed: The Neon Expanse Constitution — 2026-02-19**
