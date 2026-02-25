//! Tests for input configuration loading and RON round-trip.

use neon_expanse::plugins::input::config::{InputConfig, load_input_config_from};

/// Default InputConfig must have the canonical set of bindings.
///
/// Seven axis entries (including vehicle axes T013), four button entries
/// (including EnterExitVehicle T013), eight keyboard entries (including
/// vehicle keyboard fallbacks T013), and a dead-zone of 0.15.
#[test]
fn test_input_config_default_is_valid() {
    let config = InputConfig::default();

    assert_eq!(config.axis_map.len(), 7, "expected 7 axis entries");
    assert!(config.axis_map.contains_key("MoveForward"));
    assert!(config.axis_map.contains_key("MoveRight"));
    assert!(config.axis_map.contains_key("LookVertical"));
    assert!(config.axis_map.contains_key("LookHorizontal"));
    // T013 vehicle axes
    assert!(config.axis_map.contains_key("ThrottleForward"));
    assert!(config.axis_map.contains_key("Brake"));
    assert!(config.axis_map.contains_key("SteerRight"));

    assert_eq!(config.button_map.len(), 4, "expected 4 button entries");
    assert!(config.button_map.contains_key("PrimaryAction"));
    assert!(config.button_map.contains_key("Jump"));
    assert!(config.button_map.contains_key("Sprint"));
    // T013 vehicle button
    assert!(config.button_map.contains_key("EnterExitVehicle"));

    assert_eq!(config.keyboard_map.len(), 8, "expected 8 keyboard entries");
    assert!(config.keyboard_map.contains_key("MoveForward"));
    assert!(config.keyboard_map.contains_key("MoveRight"));
    assert!(config.keyboard_map.contains_key("Jump"));
    assert!(config.keyboard_map.contains_key("Sprint"));
    assert!(config.keyboard_map.contains_key("PrimaryAction"));
    // T013 vehicle keyboard fallbacks
    assert!(config.keyboard_map.contains_key("ThrottleForward"));
    assert!(config.keyboard_map.contains_key("Brake"));
    assert!(config.keyboard_map.contains_key("EnterExitVehicle"));

    assert!(
        (config.deadzone_radius - 0.15).abs() < f32::EPSILON,
        "expected deadzone 0.15"
    );
}

/// Serializing the default config to RON and deserializing it must
/// reproduce an identical `deadzone_radius`.
#[test]
fn test_ron_round_trip() {
    let original = InputConfig::default();
    let serialized = ron::to_string(&original).expect("serialization failed");
    let deserialized: InputConfig = ron::from_str(&serialized).expect("deserialization failed");

    assert!(
        (deserialized.deadzone_radius - original.deadzone_radius).abs() < f32::EPSILON,
        "deadzone_radius changed after RON round-trip"
    );
    assert_eq!(
        deserialized.axis_map.len(),
        original.axis_map.len(),
        "axis_map length changed after RON round-trip"
    );
    assert_eq!(
        deserialized.keyboard_map.len(),
        original.keyboard_map.len(),
        "keyboard_map length changed after RON round-trip"
    );
}

/// When the config file does not exist, `load_input_config_from` must
/// return the default bindings without panicking.
#[test]
fn test_missing_file_returns_default() {
    let config = load_input_config_from("/tmp/neon_expanse_nonexistent_config_XXXXXXXX.ron");

    // Must fall back to defaults — same shape as InputConfig::default()
    let expected = InputConfig::default();
    assert_eq!(
        config.axis_map.len(),
        expected.axis_map.len(),
        "fallback axis_map has wrong length"
    );
    assert!(
        (config.deadzone_radius - expected.deadzone_radius).abs() < f32::EPSILON,
        "fallback deadzone_radius differs from default"
    );
}
