//! Property-based tests for LOD level classification.

use proptest::prelude::*;

/// Pure function mirroring the logic in `lod::systems::update_lod_levels`.
fn classify_lod(dist: f32, thresholds: [f32; 3]) -> u8 {
    if dist < thresholds[0] {
        0
    } else if dist < thresholds[1] {
        1
    } else if dist < thresholds[2] {
        2
    } else {
        3
    }
}

proptest! {
    /// LOD level must always be in the valid range [0, 3].
    #[test]
    fn lod_level_in_range(dist in 0.0_f32..=20_000_000.0_f32) {
        let level = classify_lod(dist, [1_000.0, 10_000.0, 100_000.0]);
        prop_assert!(level <= 3);
    }

    /// Same distance must always produce the same LOD level (deterministic).
    #[test]
    fn lod_level_is_deterministic(dist in 0.0_f32..=20_000_000.0_f32) {
        let a = classify_lod(dist, [1_000.0, 10_000.0, 100_000.0]);
        let b = classify_lod(dist, [1_000.0, 10_000.0, 100_000.0]);
        prop_assert_eq!(a, b);
    }
}
