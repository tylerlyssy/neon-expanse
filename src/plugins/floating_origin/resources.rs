//! Resources for the floating-origin coordinate system.

use bevy::{math::DVec3, prelude::*};

/// The current floating-origin reference position in world space.
///
/// Updated every `PreUpdate` frame to equal the [`GlobalPosition`]
/// of the entity carrying [`FloatingOrigin`]. All [`Transform`] sync
/// computations in `PostUpdate` read this resource.
///
/// # Invariant
/// Always equals the [`GlobalPosition`] of the [`FloatingOrigin`] entity
/// at the start of the current frame (before any `Update` systems run).
#[derive(Resource, Debug, Clone, Copy)]
pub struct OriginFrame {
    /// Current reference position in metres (f64 precision).
    pub position: DVec3,
}

impl Default for OriginFrame {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
        }
    }
}
