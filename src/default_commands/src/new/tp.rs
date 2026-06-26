use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{MessageWriter, Query};
use temper_command_infra::CommandSource::*;
use temper_command_infra::args::{EntityArg, PositionArg};
use temper_command_infra::{CommandHandler, CommandSource};
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_core::mq;
use temper_macros::Command;
use temper_messages::teleport_entity::TeleportEntity;
use temper_text::TextComponent;
use tracing::info;

#[derive(Command)]
#[command("tp")]
#[allow(dead_code)]
enum TpCommand {
    TpToPos {
        location: PositionArg,
    },
    TpToEntity {
        destination: EntityArg,
    },
    TpEntityToPos {
        target: EntityArg,
        location: PositionArg,
    },
    TpEntityToEntity {
        target: EntityArg,
        destination: EntityArg,
    },
}

impl CommandHandler for TpCommand {
    type SystemParam<'w, 's> = (
        Query<'w, 's, (&'static Rotation, &'static Position)>,
        Query<'w, 's, (Entity, &'static Identity, Option<&'static PlayerMarker>)>,
        MessageWriter<'w, TeleportEntity>,
    );

    fn handle(self, source: CommandSource, params: &mut Self::SystemParam<'_, '_>) {
        let (positions, identities, teleports) = params;
        execute_tp(source, self, positions, identities, teleports);
    }
}

fn execute_tp(
    source: CommandSource,
    command: TpCommand,
    positions: &Query<(&Rotation, &Position)>,
    identities: &Query<(Entity, &Identity, Option<&PlayerMarker>)>,
    teleports: &mut MessageWriter<TeleportEntity>,
) {
    match command {
        TpCommand::TpToPos { location } => {
            let Player(player) = source else {
                send_message(source, "This command can only be used by players.".into());
                return;
            };

            let Ok((rotation, base_position)) = positions.get(player) else {
                send_message(source, "Could not find your player entity.".into());
                return;
            };

            let destination = location.resolve(base_position);
            teleport_entity(player, *rotation, destination, teleports);
            send_message(source, format!("Teleported to ({}).", destination).into());
        }
        TpCommand::TpToEntity { destination } => {
            let Player(player) = source else {
                send_message(source, "This command can only be used by players.".into());
                return;
            };

            let targets = destination.resolve(identities.iter());
            if targets.len() != 1 {
                send_message(
                    source,
                    "You must specify exactly one target to teleport to.".into(),
                );
                return;
            }

            let Ok((rotation, _)) = positions.get(player) else {
                send_message(source, "Could not find your player entity.".into());
                return;
            };

            let Ok((_, target_position)) = positions.get(targets[0]) else {
                send_message(source, "Could not find target entity position.".into());
                return;
            };

            teleport_entity(player, *rotation, *target_position, teleports);
            send_message(
                source,
                format!("Teleported to the entity at {}.", target_position).into(),
            );
        }
        TpCommand::TpEntityToPos { target, location } => {
            let base_position = if let Player(entity) = source
                && let Ok((_, position)) = positions.get(entity)
            {
                *position
            } else {
                Position::new(0.0, 0.0, 0.0)
            };
            let destination = location.resolve(&base_position);
            let targets = target.resolve(identities.iter());

            if targets.is_empty() {
                send_message(source, "No entities matched the target.".into());
                return;
            }

            for entity in targets {
                let Ok((rotation, _)) = positions.get(entity) else {
                    continue;
                };
                teleport_entity(entity, *rotation, destination, teleports);
            }

            send_message(
                source,
                format!("Teleported entities to ({}).", destination).into(),
            );
        }
        TpCommand::TpEntityToEntity {
            target,
            destination,
        } => {
            let targets = target.resolve(identities.iter());
            let destinations = destination.resolve(identities.iter());

            if targets.is_empty() {
                send_message(source, "No entities matched the target.".into());
                return;
            }

            if destinations.len() != 1 {
                send_message(
                    source,
                    "You must specify exactly one destination entity.".into(),
                );
                return;
            }

            let Ok((_, destination_position)) = positions.get(destinations[0]) else {
                send_message(source, "Could not find destination entity position.".into());
                return;
            };

            for entity in targets {
                let Ok((rotation, _)) = positions.get(entity) else {
                    continue;
                };
                teleport_entity(entity, *rotation, *destination_position, teleports);
            }

            send_message(
                source,
                format!("Teleported entities to {}.", destination_position).into(),
            );
        }
    }
}

fn teleport_entity(
    entity: Entity,
    rotation: Rotation,
    destination: Position,
    teleports: &mut MessageWriter<TeleportEntity>,
) {
    teleports.write(TeleportEntity::new(entity, destination, rotation));
}

fn send_message(source: CommandSource, message: TextComponent) {
    match source {
        Player(entity) => mq::queue(message, false, entity),
        Server => info!("{}", message.to_plain_text()),
    }
}
