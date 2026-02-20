//! Components for the floating-origin coordinate system.
//!
//! The floating-origin system maps high-precision world-space positions
//! (DVec3, f64, metres) to Bevy's f32 render-space [`Transform`].
//! Any entity that exists at planetary scales MUST use [`GlobalPosition`]
//! as its authoritative position. [`Transform`] is managed exclusively
//! by [`FloatingOriginPlugin`] and MUST NOT be set manually.

use bevy::{math::DVec3, prelude::*};

/// High-precision world-space position for any entity participating in
/// the floating-origin coordinate system.
///
/// Values are in **metres**. The coordinate origin (0, 0, 0) corresponds
/// to the planet's geometric centre.
///
/// # Invariants
/// - Components of the inner [`DVec3`] must never be NaN or infinite.
/// - Do NOT write to [`Transform`] on entities that carry this component —
///   the sync system will overwrite it every `PostUpdate`.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct GlobalPosition(pub DVec3);

/// Marker component identifying the floating-origin **reference viewer**.
///
/// The entity carrying this component defines the current origin frame.
/// Its [`GlobalPosition`] is used as the subtraction base when computing
/// all other entities' f32 [`Transform`] offsets.
///
/// # Rules
/// - Exactly ONE entity may carry this component at any time.
/// - During `Core Foundation`: the main camera entity.
/// - In future specs: may transfer to the active player entity.
#[derive(Component, Debug, Default, Reflect)]
pub struct FloatingOrigin;

/// Marker component for the diagnostic test entity.
///
/// Placed on the single visible sphere spawned at ~6,000 km global
/// distance during startup. Exists solely to enable test queries
/// that locate the entity by type without a stored [`Entity`] handle.
#[derive(Component, Debug, Default, Reflect)]
pub struct TestOriginMarker;
