# Quickstart: Basic Traversal System (Feature 003)

**Branch**: `003-basic-traversal` | **Target**: Neon Expanse local dev

---

## Prerequisites

- Rust 1.85+ stable (`rustup update stable`)
- A PS4 / DualShock 4 controller connected via USB or Bluetooth (optional — keyboard also works)
- `cargo build` completing without errors on branch `003-basic-traversal`

---

## Build & Run

```bash
git checkout 003-basic-traversal
cargo run --features dev_tools
```

Expected: window opens, camera floating above an empty world, no panics.

---

## Step 1 — Player Spawns

**What to check:**
1. After startup a white capsule appears at world position (0, 2, 0).
2. The capsule has a `GlobalPosition` component visible in the Bevy `inspector` panel (if `dev_tools` enabled).
3. No "missing resource `PlayerLocomotionConfig`" panic.

**Troubleshooting:**
- No capsule → check console for `[WARN] PlayerLocomotionConfig asset not found` — ensure `assets/player/locomotion.ron` exists.

---

## Step 2 — On-Foot Keyboard Movement

| Key | Expected result |
|---|---|
| `W` | Capsule moves forward (+Z) at ~5 m/s |
| `S` | Capsule moves backward |
| `A` / `D` | Strafe left / right |
| `Space` | Single jump, ~1 m height; cannot double-jump |
| `Shift + W` | Sprint at ~10 m/s |

**Pass criteria**: Capsule slides along flat ground; does NOT clip through floor mesh.

---

## Step 3 — Slope Handling

Find or temporarily add a sloped box mesh with > 45° pitch.

| Scenario | Expected |
|---|---|
| Walk at a 30° ramp | Slides up ramp |
| Walk into a 50° wall | Blocked, slides along the face |
| Jump off ramp | Falls and lands without tunnelling |

---

## Step 4 — PS4 Controller (On Foot)

Connect DualShock 4. The engine auto-detects the first gamepad as `Gamepad(0)`.

| Input | Expected |
|---|---|
| Left stick | Move (deadzone 0.15 applied) |
| Right stick | Camera rotation |
| `Cross` (×) | Jump |
| `L3` (left stick click) | Sprint |

**Pass criteria**: No stick drift at rest (deadzone filtering confirmed).

---

## Step 5 — Vehicle Spawns

Ensure `assets/vehicles/world_spawns.ron` has at least one entry, e.g.:
```ron
WorldSpawnList(
    entries: [
        VehicleSpawnEntry( config: "car.ron", position: (10.0, 0.5, 0.0), yaw_deg: Some(0.0) ),
    ]
)
```

**What to check:**
1. After `PostStartup`, a vehicle mesh appears at (10, 0.5, 0) world.
2. No panic for missing or malformed RON.
3. Bad config filename → `[WARN] Vehicle config 'missing.ron' not found, skipping` logged; other vehicles still spawn.

---

## Step 6 — Enter / Exit Vehicle

Walk the player capsule within 3 m of the car.

| Input | Expected |
|---|---|
| `F` (keyboard) or `Triangle` (PS4) | Player mesh disappears; camera relocates to inside vehicle view |
| `F` again | Player re-appears 2.5 m in front of vehicle; can walk again |

**Pass criteria:**
- `PassengerOf` component on player entity while inside.
- `OccupiedBy` component on vehicle entity while inside.
- Player `Visibility::Hidden` while passenger; `Visibility::Inherited` after exit.
- Cannot enter a vehicle already occupied by another player (solo testing: N/A, but the component blocks it).

---

## Step 7 — Ground Vehicle Controls (Car)

After entering the car:

| Input | Expected |
|---|---|
| `W` / R2 throttle | Car accelerates forward |
| `S` / L2 brake | Car decelerates / brakes |
| `A` / `D` / Left stick X | Steers ± max_steer_angle_deg |
| Drive off ledge | Car falls under gravity, suspension compresses on landing |
| Hit a wall at high speed | `ImpactEvent` emitted; velocity clamped (check log: `[DEBUG] ImpactEvent speed=xx`) |

**Visual check:** Wheels visually depress / extend during suspension compression (if wheel meshes are bound to wheel offsets).

---

## Step 8 — Watercraft

Spawn a motorboat near sea level (y ≈ 0 for MVP flat ocean):
```ron
VehicleSpawnEntry( config: "motorboat.ron", position: (0.0, 0.0, 50.0), yaw_deg: None )
```

| Scenario | Expected |
|---|---|
| Boat at sea level | Floats (net buoyancy ≈ mg) |
| Submerge boat (editor) | Buoyancy force rises; boat pops back up |
| Throttle W / R2 | Boat moves forward; wake (audio placeholder is OK) |
| Steer A / D | Yaws left/right; turning radius roughly matches `turn_torque_n_m` |

---

## Step 9 — Helicopter

| Scenario | Expected |
|---|---|
| Throttle up (R2 / W) to > hover_throttle | Helicopter rises |
| Throttle at hover_throttle | Holds altitude (within ±0.5 m) |
| Left stick forward | Nose pitches forward, moves forward |
| Left stick X | Rolls / banks laterally |
| Right stick X | Yaws (pedals) |
| Cut throttle | Descends; does not pass through ground |

---

## Step 10 — Fixed-Wing Airplane

| Scenario | Expected |
|---|---|
| Full throttle on ground (~runway) | Accelerates; lift builds with speed |
| Below stall speed with no throttle | No lift; descends |
| Above stall speed | Pitch up (right stick / stick back) → climbs |
| Banking right | Roll + rudder → coordinated turn |
| Land (low speed + flare) | Contacts terrain; `ImpactEvent` if speed > 20 m/s |

---

## Step 11 — Floating-Origin Precision Sanity

Move player ≥ 1 000 m from origin using console or by editing `GlobalPosition` in the inspector.

| Check | Expected |
|---|---|
| `Transform.translation` of player | ≤ 1 000 m magnitude (clamped to local frame) |
| Physics collisions still work at 50 000 m | No jitter; capsule walks normally |

---

## Step 12 — Performance Gate

With player + 5 vehicles spawned, run 60 seconds.

```bash
# Optional: log frame times
RUST_LOG=bevy_diagnostics=debug cargo run
```

**Pass**: Average frame time ≤ 16.6 ms (≥ 60 FPS) on a discrete GPU machine.  
**Fail trigger**: Any traversal FixedUpdate system exceeding 4 ms individual budget.

---

## Known Limitations (MVP)

- Sea level is a flat plane at `y = 0`; waves not simulated.
- Fixed-wing landing gear / runway not modelled (vehicle spawns floating above ground).
- Air density is constant (no altitude variation).
- No sound or VFX on any traversal event.
- Camera follow is a simple offset; no obstacle avoidance.
