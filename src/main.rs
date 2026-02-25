//! Neon Expanse — entry point.
//!
//! Registers all mandatory core plugins in dependency order and launches
//! the Bevy application. This file must never be modified after the
//! core-foundation milestone is complete.

use bevy::prelude::*;
use neon_expanse::plugins::{
    core_plugin::CorePlugin, debug_plugin::DebugPlugin, floating_origin::FloatingOriginPlugin,
    input::NeonInputPlugin, lod::LodPlugin, physics_plugin::PhysicsPlugin,
    traversal::TraversalPlugin, voxel_world_plugin::VoxelWorldPlugin,
    window_plugin::neon_window_plugins,
};

fn main() {
    App::new()
        // ── 1. DefaultPlugins (configured for borderless fullscreen) ──────────
        .add_plugins(neon_window_plugins())
        // ── 2. Core: schedules, diagnostics, fixed-tick ────────────────────
        .add_plugins(CorePlugin)
        // ── 3. Input: PS4/PS5 auto-detect + keyboard fallback ────────────────
        .add_plugins(NeonInputPlugin)
        // ── 4. Physics: Avian3d full initialization ───────────────────────────
        .add_plugins(PhysicsPlugin)
        // ── 5. Floating Origin: DVec3 global positions + Transform sync ───────
        .add_plugins(FloatingOriginPlugin)
        // ── 6. LOD: distance thresholds (no visible transitions yet) ─────────
        .add_plugins(LodPlugin)
        // ── 7. Voxel World: empty stub ────────────────────────────────────────
        .add_plugins(VoxelWorldPlugin)
        // ── 8. Traversal: empty stub ──────────────────────────────────────────
        .add_plugins(TraversalPlugin)
        // ── 9. Debug overlays: F1 wireframe toggle ────────────────────────────
        .add_plugins(DebugPlugin)
        .run();
}
