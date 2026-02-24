# neon-expanse Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-20

## Active Technologies
- Rust stable 2024 edition, minimum 1.85+ + Bevy 0.18, avian3d 0.6.0-rc.1, fastnoise-lite 1.1, serde + ron 0.8 (002-voxel-planet-engine)
- RON assets + in-memory `ChunkPool` (capped by `PlanetConfig::memory_budget_mb`) (002-voxel-planet-engine)

- Rust stable 2024 edition, minimum 1.85+ + `bevy = "0.18"`, `avian3d = "0.5"`, `glam` (re-exported by Bevy), `serde = "1"`, `ron = "0.8"`, `proptest = "1"` (dev) (001-core-foundation)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust stable 2024 edition, minimum 1.85+: Follow standard conventions

## Recent Changes
- 002-voxel-planet-engine: Added Rust stable 2024 edition, minimum 1.85+ + Bevy 0.18, avian3d 0.6.0-rc.1, fastnoise-lite 1.1, serde + ron 0.8

- 001-core-foundation: Added Rust stable 2024 edition, minimum 1.85+ + `bevy = "0.18"`, `avian3d = "0.5"`, `glam` (re-exported by Bevy), `serde = "1"`, `ron = "0.8"`, `proptest = "1"` (dev)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
