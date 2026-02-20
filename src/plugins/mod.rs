//! Neon Expanse plugin registry.
//!
//! All eight mandatory core plugins are declared here and re-exported
//! for use in `main.rs`. This module must never gain new entries after
//! the core-foundation milestone without a corresponding spec.

pub mod core_plugin;
pub mod floating_origin;
pub mod input;
pub mod lod;
pub mod physics_plugin;
pub mod traversal_plugin;
pub mod voxel_world_plugin;
pub mod window_plugin;
