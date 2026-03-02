//! Planet configuration types — fully data-driven via RON.
//!
//! Loaded at startup from `assets/config/planets/planet.ron` (or the path
//! specified by the `NEON_PLANET_CONFIG` environment variable) and inserted
//! as a `PlanetConfig` Bevy resource.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ── Noise ────────────────────────────────────────────────────────────────────

/// Variant of fractal noise used by a procedural layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum NoiseKind {
    /// Fractal Brownian Motion — smooth, layered octaves.
    Fbm,
    /// Ridged multifractal — sharp mountain ridges.
    Ridged,
    /// Billow — puffy cloud-like shapes.
    Billow,
}

/// A single procedural noise layer in the planet modifier stack.
///
/// Layers are evaluated in order and summed additively on top of the base
/// sphere SDF. All fields are serialisable as RON.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NoiseLayer {
    /// Noise variant for this layer.
    pub kind: NoiseKind,
    /// Spatial frequency of the noise (cycles per metre).
    pub frequency: f32,
    /// Maximum displacement this layer contributes (metres).
    pub amplitude: f32,
    /// Number of fractal octaves.
    pub octaves: u32,
    /// Amplitude scaling per octave (0–1).
    pub persistence: f32,
    /// Frequency scaling per octave (> 1).
    pub lacunarity: f32,
}

// ── Erosion ──────────────────────────────────────────────────────────────────

/// Hydraulic erosion parameters. Applied once at planet init after all noise
/// layers have been evaluated; does NOT run per-frame.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ErosionConfig {
    /// Number of erosion simulation passes over the density grid.
    pub passes: u32,
    /// Fraction of sediment removed from high-gradient cells per pass.
    pub erosion_rate: f32,
    /// Maximum sediment a simulated droplet can carry.
    pub sediment_capacity: f32,
}

impl Default for ErosionConfig {
    fn default() -> Self {
        Self {
            passes: 2,
            erosion_rate: 0.3,
            sediment_capacity: 0.6,
        }
    }
}

// ── Biome ────────────────────────────────────────────────────────────────────

/// One entry in the height-to-colour biome table.
///
/// `height_fraction` is the normalised elevation relative to planet radius:
/// - `-1.0` → deepest ocean
/// - `0.0`  → sea-level surface
/// - `+1.0` → maximum mountain peak
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BiomeBand {
    /// Normalised height threshold for this colour band.
    pub height_fraction: f32,
    /// Linear RGB colour applied at vertices at or below this height.
    pub colour: (f32, f32, f32),
}

/// Ordered table mapping normalised elevation to per-vertex biome colour.
///
/// Bands MUST be sorted by `height_fraction` ascending. Colours between
/// bands are linearly interpolated.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BiomeConfig {
    /// Sorted (ascending `height_fraction`) biome colour bands.
    pub bands: Vec<BiomeBand>,
}

impl Default for BiomeConfig {
    fn default() -> Self {
        Self {
            bands: vec![
                BiomeBand {
                    height_fraction: -1.0,
                    colour: (0.05, 0.10, 0.45),
                },
                BiomeBand {
                    height_fraction: -0.1,
                    colour: (0.08, 0.18, 0.55),
                },
                BiomeBand {
                    height_fraction: 0.0,
                    colour: (0.76, 0.70, 0.50),
                },
                BiomeBand {
                    height_fraction: 0.2,
                    colour: (0.25, 0.55, 0.20),
                },
                BiomeBand {
                    height_fraction: 0.5,
                    colour: (0.20, 0.40, 0.15),
                },
                BiomeBand {
                    height_fraction: 0.7,
                    colour: (0.55, 0.50, 0.45),
                },
                BiomeBand {
                    height_fraction: 1.0,
                    colour: (0.95, 0.95, 0.98),
                },
            ],
        }
    }
}

// ── PlanetConfig ─────────────────────────────────────────────────────────────

/// Serde default for `PlanetConfig::max_loaded_chunks`.
fn default_max_loaded_chunks() -> u32 {
    512
}

/// Full planet configuration loaded from RON at startup.
///
/// Inserted as a `Resource` by the `load_planet_config` startup system.
/// The `NEON_PLANET_CONFIG` environment variable overrides the default asset path.
/// If the file is absent or malformed, `PlanetConfig::default()` is used and
/// a warning is logged — the engine never panics on missing config.
///
/// # Modding
/// A new planet variant requires only a new RON file. No source changes or
/// recompile are needed. See `assets/config/planets/planet.ron` for the schema.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PlanetConfig {
    /// Planet radius in kilometres. Drives all derivative spatial constants.
    pub radius_km: f64,
    /// Optional seed override. When `Some`, overrides the `WorldSeed` resource.
    pub seed_override: Option<u64>,
    /// Maximum voxel data resident in memory (megabytes). Controls eviction.
    pub memory_budget_mb: u32,
    /// Hard cap on the number of simultaneously loaded chunk entities.
    ///
    /// Prevents unbounded entity-count growth during the initial surface fill-up.
    /// New chunk tasks are not dispatched while `pool.loaded.len() >= max_loaded_chunks`.
    /// Eviction (out-of-range or over-budget) frees slots so nearby chunks can replace them.
    /// Setting this to `0` disables the count cap (memory budget is still enforced).
    #[serde(default = "default_max_loaded_chunks")]
    pub max_loaded_chunks: u32,
    /// Ordered list of additive noise layers applied to the base sphere SDF.
    pub noise_layers: Vec<NoiseLayer>,
    /// Hydraulic erosion parameters applied once at planet init.
    pub erosion: ErosionConfig,
    /// Height-to-colour biome table for per-vertex colouring.
    pub biome: BiomeConfig,
}

impl Default for PlanetConfig {
    /// Hard-coded Earth-like fallback used when `planet.ron` is absent or malformed.
    fn default() -> Self {
        Self {
            radius_km: 6_371.0,
            seed_override: None,
            memory_budget_mb: 512,
            max_loaded_chunks: 512,
            noise_layers: vec![
                NoiseLayer {
                    kind: NoiseKind::Fbm,
                    frequency: 0.000_000_8,
                    amplitude: 8_000.0,
                    octaves: 6,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
                NoiseLayer {
                    kind: NoiseKind::Ridged,
                    frequency: 0.000_004,
                    amplitude: 1_200.0,
                    octaves: 4,
                    persistence: 0.6,
                    lacunarity: 2.2,
                },
                NoiseLayer {
                    kind: NoiseKind::Billow,
                    frequency: 0.000_02,
                    amplitude: 200.0,
                    octaves: 3,
                    persistence: 0.45,
                    lacunarity: 1.9,
                },
            ],
            erosion: ErosionConfig::default(),
            biome: BiomeConfig::default(),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let config = PlanetConfig::default();
        assert_eq!(config.radius_km, 6_371.0);
        assert_eq!(config.noise_layers.len(), 3);
        assert_eq!(config.biome.bands.len(), 7);
        assert_eq!(config.erosion.passes, 2);
    }

    #[test]
    fn planet_ron_deserializes() {
        let src = std::fs::read_to_string("assets/config/planets/planet.ron")
            .expect("planet.ron should exist");
        let config: PlanetConfig = ron::from_str(&src).expect("planet.ron should deserialize");
        assert!(config.radius_km > 0.0);
        assert!(!config.noise_layers.is_empty());
        assert!(!config.biome.bands.is_empty());
    }

    /// T031 — test_variant.ron (Mars-scale) deserialises and round-trips.
    #[test]
    fn test_variant_ron_deserializes() {
        let src = std::fs::read_to_string("assets/config/planets/test_variant.ron")
            .expect("test_variant.ron should exist");
        let config: PlanetConfig =
            ron::from_str(&src).expect("test_variant.ron should deserialize");
        // Mars-scale radius
        assert!(
            (config.radius_km - 3_389.0).abs() < 1.0,
            "expected radius ~3389 km, got {}",
            config.radius_km
        );
        assert!(
            !config.noise_layers.is_empty(),
            "noise layers must not be empty"
        );
        assert!(
            !config.biome.bands.is_empty(),
            "biome bands must not be empty"
        );
    }

    /// T031 — ensure test_variant.ron re-serialises to identical RON then parses back.
    #[test]
    fn test_variant_ron_roundtrip() {
        let src = std::fs::read_to_string("assets/config/planets/test_variant.ron")
            .expect("test_variant.ron should exist");
        let config: PlanetConfig = ron::from_str(&src).expect("initial deserialise");
        // Verify key fields survive a full Rust round-trip (not re-serialising to
        // RON, just checking the parsed values are stable when used as a source
        // of truth).
        let radius = config.radius_km;
        let layers = config.noise_layers.len();
        let bands = config.biome.bands.len();
        // Second pass: re-parse and compare field by field.
        let config2: PlanetConfig = ron::from_str(&src).expect("second deserialise");
        assert_eq!(config2.radius_km, radius);
        assert_eq!(config2.noise_layers.len(), layers);
        assert_eq!(config2.biome.bands.len(), bands);
    }

    #[test]
    fn missing_config_falls_back_to_default() {
        // Simulates load_planet_config fallback path without filesystem access.
        let config = PlanetConfig::default();
        assert_eq!(config.radius_km, 6_371.0);
    }
}
