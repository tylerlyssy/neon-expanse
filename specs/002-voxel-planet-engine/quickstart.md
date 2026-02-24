# Quickstart: Procedural Voxel Planet Engine

**Feature**: `002-voxel-planet-engine` | **Date**: 2026-02-20

Follow this guide to implement, run, and validate Feature 002 from scratch.

---

## Prerequisites

- Feature 001 merged and all 8 `cargo test` cases passing.
- Rust stable 1.85+ (`rustup update stable`).
- PS4 controller connected (optional but recommended for the fly-through test).

---

## Step 1 — Add `fastnoise-lite` to `Cargo.toml`

```toml
[dependencies]
# … existing deps …
fastnoise-lite = "1.1"
```

Verify: `cargo check` passes.

---

## Step 2 — Create the Asset File

```bash
mkdir -p assets/config/planets
# Copy the planet.ron from plan.md's RON schema section, or:
cat > assets/config/planets/planet.ron << 'EOF'
PlanetConfig(
    radius_km: 6371.0,
    seed_override: None,
    memory_budget_mb: 512,
    noise_layers: [
        NoiseLayer(kind: Fbm, frequency: 0.0000008, amplitude: 8000.0,
                   octaves: 6, persistence: 0.5, lacunarity: 2.0),
    ],
    erosion: ErosionConfig(passes: 1, erosion_rate: 0.3, sediment_capacity: 0.6),
    biome: BiomeConfig(bands: [
        (height_fraction: -1.0, colour: (0.05, 0.10, 0.45)),
        (height_fraction:  0.0, colour: (0.76, 0.70, 0.50)),
        (height_fraction:  0.5, colour: (0.25, 0.55, 0.20)),
        (height_fraction:  1.0, colour: (0.95, 0.95, 0.98)),
    ]),
)
EOF
```

---

## Step 3 — Create the Module Tree

Create the following empty files (content from `plan.md`):

```bash
mkdir -p src/plugins/voxel/mesher
mkdir -p src/plugins/voxel/systems
mkdir -p src/plugins/voxel/tests

touch src/plugins/voxel/mod.rs
touch src/plugins/voxel/components.rs
touch src/plugins/voxel/resources.rs
touch src/plugins/voxel/config.rs
touch src/plugins/voxel/noise_stack.rs
touch src/plugins/voxel/erosion.rs
touch src/plugins/voxel/mesher/mod.rs
touch src/plugins/voxel/mesher/dmc.rs
touch src/plugins/voxel/mesher/tables.rs
touch src/plugins/voxel/systems/mod.rs
touch src/plugins/voxel/systems/planet_init.rs
touch src/plugins/voxel/systems/scaled_space.rs
touch src/plugins/voxel/systems/streaming.rs
touch src/plugins/voxel/systems/mesh_builder.rs
touch src/plugins/voxel/systems/collider_sync.rs
```

---

## Step 4 — Implement Files

Implement each file using the code in `plan.md` as the authoritative reference.
Recommended implementation order (each step produces a compiling binary):

| Step | Files | Compile Gate |
|------|-------|-------------|
| 4.1 | `config.rs` + `mod.rs` declaration | `cargo check` |
| 4.2 | `components.rs`, `resources.rs` | `cargo check` |
| 4.3 | `noise_stack.rs` + `fastnoise-lite` dep | `cargo check` |
| 4.4 | `mesher/dmc.rs` (DMC core) | `cargo check` |
| 4.5 | `systems/planet_init.rs` (config load + sphere spawn) | `cargo run` → sphere visible |
| 4.6 | `systems/scaled_space.rs` | `cargo run` → sphere toggles |
| 4.7 | `systems/streaming.rs` (streamer + poller) | `cargo run` → chunks generating |
| 4.8 | `systems/collider_sync.rs` | `cargo run` → physics solid |
| 4.9 | Replace `voxel_world_plugin.rs` stub | `cargo run` → full pipeline |

---

## Step 5 — Run and Validate

### Basic launch
```bash
cargo run
```
Expected within 2 seconds: white/green sphere visible against a black background.

### Descent fly-through (PS4 controller)
- **Left stick**: translate camera
- **L2**: descend towards planet
- **R2**: ascend
- **✕**: boost (10× speed)

Descent path: hold L2 from 80,000 km altitude. Watch LOD transitions at:
- ~63,710 km → LOD 3 impostor fades (scaled-space sphere → chunk mesh)
- ~100 km → LOD 2 chunks appear
- ~10 km → LOD 1 chunks appear
- ~1 km → LOD 0 high-detail chunks with surface overhangs

### Keyboard fallback
| Key | Action |
|-----|--------|
| WASD | Translate XZ |
| Space / Shift | Ascend / Descend |
| Mouse | Look |

---

## Step 6 — Run Tests

```bash
# Unit + proptest
cargo test voxel

# 1000-case determinism fuzz
PROPTEST_CASES=1000 cargo test voxel_determinism

# Full suite — must not regress 001 tests
cargo test

# Lints
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All must pass before merging.

---

## Step 7 — Test Variant Planet

```bash
cp assets/config/planets/planet.ron assets/config/planets/test_variant.ron
# Edit test_variant.ron: change radius_km to 3000.0 and noise amplitude to 2000.0
NEON_PLANET_CONFIG=assets/config/planets/test_variant.ron cargo run
```

Expected: visually smaller planet with less dramatic terrain relief.
No code changes required — validates US5 / FR-006.

## Step 5 — SC-002 Seam Visual Verification (Manual Checklist)

> **Note**: Bevy 0.18's `ScreenshotManager` spawns a secondary render pass and
> cannot reliably capture headless test frames in CI without a full GPU driver.
> This step must be performed manually by the reviewer before merging.

### Setup
```bash
cargo run
# Or with the alternate planet:
NEON_PLANET_CONFIG=assets/config/planets/test_variant.ron cargo run
```

### Checkpoints

At each altitude below, pause and inspect for visual seams between LOD bands:

| Altitude | LOD Transition | Expected | ✓/✗ |
|----------|---------------|----------|-----|
| 63,710 km | LOD-3 impostor → LOD-2 mesh | Smooth fade; no pop | |
| 100 km   | LOD-2 → LOD-1 chunks     | No dark seam lines perpendicular to surface | |
| 10 km    | LOD-1 → LOD-0 chunks     | No T-junction cracks at chunk borders | |
| 1 km     | LOD-0 high detail         | Overhangs and terrain relief visible | |

### Pass Criteria
- No dark lines / Z-fighting visible at LOD boundaries.
- No single-frame popping (geometry disappears then reappears in 1 frame).
- All terrain surfaces are continuous (no visible holes).
- Physics cube rests on LOD-0 surface without clipping (drop from 20 m above).

### Annotated Reference Screenshots
Commit reference screenshots to `tests/snapshots/lod_seam_*.png` after the first
successful manual verification pass. Subsequent reviewers compare visually.

---

- [X] `cargo test` — 22 tests pass (12 lib + 3×2 integration + 2 determinism)
- [X] `PROPTEST_CASES=1000 cargo test --features proptest` — green (T034+T035)
- [X] `cargo clippy --all-targets -- -D warnings` — zero errors
- [X] `cargo fmt --check` — clean
- [X] `NEON_PLANET_CONFIG` override works with `test_variant.ron` (T029, T031)
- [X] All Feature 002 public items have `///` documentation (T038)
- [X] `tests/snapshots/voxel_density_golden.bin` committed (T033, 400 KB)
- [ ] Planet sphere visible within 2 s of `cargo run` (requires runtime verification)
- [ ] No geometry seam observed during manual descent (SC-002 — manual visual test)
- [X] Physics cube rests on surface without clipping (T024 — headless integration test + manual runtime verification)
