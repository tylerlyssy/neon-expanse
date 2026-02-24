//! Scaled-space switcher system.
//!
//! Toggles between the cheap scaled-space UV sphere (visible from orbit) and
//! the full voxel mesh (visible at close range) based on viewer distance from
//! the planet centre.
//!
//! Transition policy (FR-012 / H-AMB-1 resolution):
//! A 1-frame `Visibility` hysteresis prevents flicker at the threshold.
//! Continuous alpha-blend fading is deferred to a future sprint.

use bevy::prelude::*;

use crate::plugins::{
    floating_origin::resources::OriginFrame,
    voxel::{components::ScaledSpaceMarker, config::PlanetConfig, resources::ScaledSpaceState},
};

/// The distance multiplier above which the scaled-space sphere is shown.
///
/// `sphere_visible` when `viewer_dist > SWITCH_MULTIPLIER * planet_radius`.
const SWITCH_MULTIPLIER: f64 = 10.0;

/// `Update` system: toggle planet sphere / voxel-mesh visibility by altitude.
///
/// Reads `OriginFrame::position` as the viewer's world-space location. The
/// sphere is shown when the viewer is more than 10× the planet radius from
/// the planet centre (`DVec3::ZERO`).
///
/// # 1-frame hysteresis
/// A ±1% dead-band around the threshold prevents the sphere flickering on/off
/// when the viewer hovers exactly at the boundary.
pub fn scaled_space_switcher(
    origin: Res<OriginFrame>,
    planet_config: Res<PlanetConfig>,
    mut state: ResMut<ScaledSpaceState>,
    mut sphere_q: Query<&mut Visibility, With<ScaledSpaceMarker>>,
) {
    let planet_r_m = planet_config.radius_km * 1_000.0;
    let switch_dist = planet_r_m * SWITCH_MULTIPLIER;
    // 1% dead-band: hysteresis window to prevent per-frame flicker.
    let hysteresis = switch_dist * 0.01;

    let viewer_dist = origin.position.length();

    let should_show_sphere = if state.sphere_visible {
        // Currently showing sphere: keep it until viewer moves well inside threshold.
        viewer_dist > (switch_dist - hysteresis)
    } else {
        // Currently showing voxels: switch to sphere once viewer is well outside.
        viewer_dist > (switch_dist + hysteresis)
    };

    if should_show_sphere != state.sphere_visible {
        let new_vis = if should_show_sphere {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for mut vis in &mut sphere_q {
            *vis = new_vis;
        }

        state.sphere_visible = should_show_sphere;

        debug!(
            "ScaledSpace switched: sphere={} at viewer_dist={:.1} km (threshold={:.1} km)",
            should_show_sphere,
            viewer_dist / 1_000.0,
            switch_dist / 1_000.0,
        );
    }
}
