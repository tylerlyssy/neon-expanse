//! LOD (Level of Detail) components.

use bevy::prelude::*;

/// The currently assigned LOD level for a world entity.
///
/// Updated every `PostUpdate` frame by the LOD system based on
/// distance from the floating origin (the active camera / viewer).
///
/// | Level | Distance range       | Intended rendering |
/// |-------|---------------------|--------------------|
/// | `0`   | < 1,000 m           | High detail        |
/// | `1`   | 1,000 – 10,000 m    | Medium detail      |
/// | `2`   | 10,000 – 100,000 m  | Low detail         |
/// | `3`   | > 100,000 m         | Impostor           |
///
/// During Core Foundation no LOD transitions are observable because
/// no terrain geometry exists. The component is present on the test
/// sphere so future rendering systems can query it without modification.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct LodLevel(pub u8);
