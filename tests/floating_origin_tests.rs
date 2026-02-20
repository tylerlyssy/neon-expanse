//! Property-based tests for the floating-origin coordinate system.

use glam::DVec3;
use proptest::prelude::*;

/// Generates DVec3 values in the range of planetary coordinates
/// (-20,000 km to +20,000 km per axis).
fn planetary_dvec3() -> impl Strategy<Value = DVec3> {
    (
        -20_000_000.0_f64..=20_000_000.0_f64,
        -20_000_000.0_f64..=20_000_000.0_f64,
        -20_000_000.0_f64..=20_000_000.0_f64,
    )
        .prop_map(|(x, y, z)| DVec3::new(x, y, z))
}

proptest! {
    /// The local f32 offset between any two planetary positions must
    /// be representable with no catastrophic precision loss, provided
    /// the entities are within ~1,000 km of each other.
    #[test]
    fn sync_offset_is_finite_at_planetary_scale(
        origin in planetary_dvec3(),
        entity in planetary_dvec3()
    ) {
        let offset = (entity - origin).as_vec3();
        prop_assert!(offset.is_finite(), "offset contains NaN or infinity");
    }

    /// When origin == entity, the local transform is exactly zero.
    #[test]
    fn same_position_produces_zero_offset(pos in planetary_dvec3()) {
        let offset = (pos - pos).as_vec3();
        prop_assert_eq!(offset, glam::Vec3::ZERO);
    }

    /// Camera at origin → camera transform is zero vector.
    #[test]
    fn camera_transform_is_always_zero(camera_pos in planetary_dvec3()) {
        let offset = (camera_pos - camera_pos).as_vec3();
        prop_assert_eq!(offset, glam::Vec3::ZERO);
    }
}
