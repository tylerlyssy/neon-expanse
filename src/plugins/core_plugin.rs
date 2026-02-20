//! Core application plugin for Neon Expanse.
//!
//! Configures the deterministic world seed, the 64 Hz fixed-timestep
//! schedule shared by game logic and physics, and all diagnostic
//! logging plugins. This plugin must be registered before all other
//! game plugins.

use std::time::Duration;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

/// The shared world seed used by all procedural generation systems.
///
/// Changing this constant changes the entire planet. Never random.
/// ASCII representation: `"NEON2026"`.
pub const WORLD_SEED: u64 = 0x4E45_4F4E_3230_3236; // ASCII: "NEON2026"

/// Bevy plugin that configures schedules, diagnostics, and deterministic seed.
///
/// Must be registered immediately after `DefaultPlugins`.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // ── Fixed-timestep: 64 Hz ─────────────────────────────────────────────
        // Physics (FixedPostUpdate / avian3d) and game logic (FixedUpdate) both
        // tick at this rate. This is the single source of truth for simulation
        // frequency across the entire engine.
        app.insert_resource(Time::<Fixed>::from_hz(64.0));

        // ── World seed resource ───────────────────────────────────────────────
        app.insert_resource(WorldSeed(WORLD_SEED));

        // ── Diagnostics ───────────────────────────────────────────────────────
        // LogDiagnosticsPlugin emits FPS, frame time, and entity count
        // to the console at 1 Hz (once per second).
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin {
                wait_duration: Duration::from_secs(1),
                filter: None,
                ..default()
            },
        ));
    }
}

/// Deterministic world seed resource.
///
/// Every procedural generation system MUST consume this resource rather
/// than seeding its own RNG independently. Identical seeds produce
/// identical planets, guaranteed.
#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);
