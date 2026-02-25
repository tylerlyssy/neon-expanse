//! Traversal ECS resources.

use bevy::prelude::*;

use crate::plugins::traversal::config::VehicleConfig;

/// Tracks which vehicle the player currently occupies.
/// `None` → player is on foot; `Some(entity)` → player is inside that vehicle.
/// Read by enter/exit, input routing, and player locomotion systems.
#[derive(Resource, Debug, Default, Reflect)]
pub struct CurrentVehicle(pub Option<Entity>);

/// Registry of all vehicle configs loaded from  `assets/vehicles/*.ron` at startup.
/// Used by `startup_spawn_vehicles` to construct vehicle entities. FR-020.
#[derive(Resource, Debug, Default)]
pub struct VehicleRegistry(pub Vec<VehicleConfig>);
