# Quickstart — Core Foundation

Branch: `001-core-foundation`  
Target: stable, borderless fullscreen window + cyan sphere at ~6,000 km

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.85+ (stable) | `rustup update stable` |
| Cargo | bundled with Rust | — |
| macOS | 12+ (primary dev) | — |

No external C libraries are required on macOS. On Linux you will need `libudev-dev` and `libasound2-dev` for Bevy's gamepad/audio backends.

---

## Build & Run (development)

```bash
# Clone / check out the feature branch
git checkout 001-core-foundation

# First build (downloads all crates — takes ~2–4 min on a cold cache)
cargo build

# Run the game (dynamic linking, dev profile)
cargo run
```

Expected output on the console:

```
INFO neon_expanse: Neon Expanse core initialized
INFO neon_expanse: Test entity spawned at global position 6000000 m from world origin
INFO neon_expanse::plugins::input::systems: No controller detected — keyboard/mouse fallback active
INFO bevy_diagnostic::log_diagnostics_plugin: fps: 60.xx | frame_time: xx.xx ms | entity_count: 4
```

Expected on screen:

- Borderless fullscreen window titled "Neon Expanse v0.0.1-dev"
- Black background
- Cyan/teal sphere approximately 100 m in front of the camera

---

## With Controller

Connect a PS4 or PS5 controller **before** launching, or plug it in while running.  
You should see in the console:

```
INFO neon_expanse::plugins::input::systems: PS4/PS5 controller connected: DualSense Wireless Controller
```

---

## Tracy Profiler (optional)

```bash
# Requires Tracy installed separately: https://github.com/wolfpld/tracy
cargo run --features tracy
```

---

## Release Build

```bash
cargo build --release
./target/release/neon-expanse
```

---

## Run Tests

```bash
cargo test
```

Property-based tests run with a fixed seed by default. To run more iterations:

```bash
PROPTEST_CASES=10000 cargo test
```

---

## Run Lints (CI gate)

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Both must pass with zero warnings before merging any PR.

---

## Remap Controls

Edit `assets/config/input.ron`. The file is loaded at startup; delete it to revert to built-in defaults. No recompile needed.

---

## Project Layout Reference

See [plan.md](plan.md) → **Project Structure** section for the complete file tree.
