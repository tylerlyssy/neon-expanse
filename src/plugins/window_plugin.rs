//! Borderless fullscreen window configuration for Neon Expanse.
//!
//! Returns a configured [`DefaultPlugins`] group with the correct
//! window mode, present mode, and title. This function is called
//! exactly once from `main.rs` and must never be modified after the
//! core-foundation milestone.

use bevy::{
    prelude::*,
    window::{MonitorSelection, PresentMode, Window, WindowMode, WindowPlugin},
};

/// Returns Bevy's [`DefaultPlugins`] configured for Neon Expanse:
/// borderless fullscreen on the primary monitor, no vsync, with
/// the canonical window title.
///
/// # Plugin group contents
/// All standard Bevy plugins are included. The only customisation is
/// the [`WindowPlugin`] configuration — all other plugins use defaults.
pub fn neon_window_plugins() -> impl PluginGroup {
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Neon Expanse v0.0.1-dev".into(),
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    })
}
