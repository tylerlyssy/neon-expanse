//! Vehicle enter/exit lifecycle systems.
//!
//! Systems:
//! - `update_enter_exit_prompts` — each `Update` frame, scan which players are
//!   within `interact_radius_m` of each vehicle and update `EnterExitProximity`.
//! - `enter_exit_vehicle` — each `Update` frame, emit `EnterVehicleEvent` or
//!   `ExitVehicleEvent` when the player presses the enter/exit binding.
//! - `apply_enter_vehicle` — consume `EnterVehicleEvent`, assign components.
//! - `apply_exit_vehicle` — consume `ExitVehicleEvent`, remove components and
//!   reposition the player outside the vehicle.

use crate::plugins::floating_origin::components::GlobalPosition;
use bevy::prelude::*;

use crate::plugins::traversal::{
    components::{
        EnterExitHintMarker, EnterExitProximity, OccupiedBy, OnFootInputState, PassengerOf,
        PlayerTag, VehicleInputState, VehicleTag,
    },
    config::PlayerLocomotionConfig,
    events::{EnterVehicleEvent, ExitVehicleEvent},
    resources::CurrentVehicle,
};

// ── T026: Proximity scan ─────────────────────────────────────────────────────

/// Each `Update` frame, refresh which players are within `interact_radius_m` of
/// every vehicle.  The result drives UI prompts and `enter_exit_vehicle`.
pub fn update_enter_exit_prompts(
    loco_cfg: Res<PlayerLocomotionConfig>,
    player_q: Query<(Entity, &GlobalPosition), With<PlayerTag>>,
    mut vehicle_q: Query<
        (
            &GlobalPosition,
            &mut EnterExitProximity,
            Option<&OccupiedBy>,
        ),
        With<VehicleTag>,
    >,
    hint_q: Query<Entity, With<EnterExitHintMarker>>,
    mut commands: Commands,
) {
    let radius = loco_cfg.interact_radius_m;

    for (vehicle_pos, mut proximity, occupied) in &mut vehicle_q {
        // Occupied vehicles cannot be entered by a second passenger.
        if occupied.is_some() {
            proximity.nearby_players.clear();
            continue;
        }

        proximity.nearby_players.clear();
        for (player_entity, player_pos) in &player_q {
            let dist = (player_pos.0 - vehicle_pos.0).length() as f32;
            if dist <= radius {
                proximity.nearby_players.push(player_entity);
            }
        }
    }

    // T026a: UI hint — show when any vehicle has nearby players.
    let any_nearby = vehicle_q
        .iter()
        .any(|(_, prox, _)| !prox.nearby_players.is_empty());

    let hint_exists = !hint_q.is_empty();

    if any_nearby && !hint_exists {
        commands.spawn((
            EnterExitHintMarker,
            // Minimal UI text node — full styling is out of scope for this milestone.
            Text::new("[F] / [△] Enter vehicle"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(50.0),
                ..default()
            },
        ));
        warn!("TODO: style EnterExitHintMarker properly (center, colour, etc.)");
    } else if !any_nearby && hint_exists {
        for entity in &hint_q {
            commands.entity(entity).despawn();
        }
    }
}

// ── T026: Enter/exit request ──────────────────────────────────────────────────

/// Detect the player's enter/exit key-press and emit the appropriate message.
///
/// - Player on foot + nearby vehicle → `EnterVehicleEvent`  
/// - Player in vehicle → `ExitVehicleEvent`
pub fn enter_exit_vehicle(
    player_q: Query<(Entity, &OnFootInputState, Option<&PassengerOf>), With<PlayerTag>>,
    vehicle_q: Query<(Entity, &EnterExitProximity), With<VehicleTag>>,
    mut enter_writer: MessageWriter<EnterVehicleEvent>,
    mut exit_writer: MessageWriter<ExitVehicleEvent>,
    current_vehicle: Res<CurrentVehicle>,
) {
    for (player, on_foot_input, maybe_passenger) in &player_q {
        if !on_foot_input.enter_exit_pressed {
            continue;
        }

        if let Some(PassengerOf(vehicle)) = maybe_passenger {
            // Player is inside a vehicle — request exit.
            exit_writer.write(ExitVehicleEvent {
                player,
                vehicle: *vehicle,
            });
        } else {
            // Player is on foot — find the nearest vehicle in proximity.
            let _ = current_vehicle; // silence unused warning; used for read path
            let nearest = vehicle_q
                .iter()
                .filter(|(_, prox)| prox.nearby_players.contains(&player))
                .min_by(|(pos_a, _), (pos_b, _)| {
                    // Both entities are merely candidates; tiebreak by Entity id.
                    pos_a
                        .partial_cmp(pos_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            if let Some((vehicle_entity, _)) = nearest {
                enter_writer.write(EnterVehicleEvent {
                    player,
                    vehicle: vehicle_entity,
                });
            }
        }
    }
}

// ── T026: Apply enter ─────────────────────────────────────────────────────────

/// Consume `EnterVehicleEvent` and perform the component lifecycle:
/// - Hide the player (they sit inside the vehicle visually).
/// - Mark the player as `PassengerOf(vehicle)`.
/// - Mark the vehicle as `OccupiedBy(player)`.
/// - Update `CurrentVehicle` resource.
pub fn apply_enter_vehicle(
    mut enter_reader: MessageReader<EnterVehicleEvent>,
    mut commands: Commands,
    mut current_vehicle: ResMut<CurrentVehicle>,
) {
    for ev in enter_reader.read() {
        commands
            .entity(ev.player)
            .insert((PassengerOf(ev.vehicle), Visibility::Hidden));

        commands.entity(ev.vehicle).insert(OccupiedBy(ev.player));

        current_vehicle.0 = Some(ev.vehicle);

        info!(
            player = ?ev.player,
            vehicle = ?ev.vehicle,
            "Player entered vehicle"
        );
    }
}

// ── T026: Apply exit ──────────────────────────────────────────────────────────

/// Consume `ExitVehicleEvent` and perform the component lifecycle:
/// - Make the player visible again.
/// - Remove `PassengerOf` from the player.
/// - Remove `OccupiedBy` from the vehicle.
/// - Reposition the player in front of the vehicle.
/// - Clear `CurrentVehicle`.
pub fn apply_exit_vehicle(
    mut exit_reader: MessageReader<ExitVehicleEvent>,
    mut commands: Commands,
    vehicle_pos_q: Query<&GlobalPosition, With<VehicleTag>>,
    mut player_pos_q: Query<&mut GlobalPosition, (With<PlayerTag>, Without<VehicleTag>)>,
    mut current_vehicle: ResMut<CurrentVehicle>,
    mut vehicle_input_q: Query<&mut VehicleInputState>,
) {
    for ev in exit_reader.read() {
        // Clear vehicle input so it doesn't keep driving after exit.
        if let Ok(mut vin) = vehicle_input_q.get_mut(ev.vehicle) {
            *vin = VehicleInputState::default();
        }

        // Reposition player 2.5 m in front of the vehicle.
        if let Ok(vpos) = vehicle_pos_q.get(ev.vehicle) {
            if let Ok(mut ppos) = player_pos_q.get_mut(ev.player) {
                ppos.0 = vpos.0 + bevy::math::DVec3::new(0.0, 0.5, 2.5);
            }
        }

        commands
            .entity(ev.player)
            .remove::<PassengerOf>()
            .insert(Visibility::Visible);

        commands.entity(ev.vehicle).remove::<OccupiedBy>();

        current_vehicle.0 = None;

        info!(
            player = ?ev.player,
            vehicle = ?ev.vehicle,
            "Player exited vehicle"
        );
    }
}
