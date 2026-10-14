use bevy_ecs::prelude::{Commands, MessageReader, MessageWriter, Query};
use temper_codec::net_types::var_int::VarInt;
use temper_components::player::dimension::CurrentDimension;
use temper_components::player::gamemode::GameModeComponent;
use temper_components::player::old_position::OldPosition;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_components::player::teleport_tracker::TeleportTracker;
use temper_components::player::velocity::Velocity;
use temper_messages::chunk_calc::ChunkCalc;
use temper_messages::dimension_change::DimensionChange;
use temper_messages::teleport_entity::TeleportEntity;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::game_event::GameEventPacket;
use temper_protocol::outgoing::respawn::RespawnPacket;
use tracing::warn;

pub fn handle_dimension_change(
    mut messages: MessageReader<DimensionChange>,
    mut entities: Query<(&mut Position, &mut CurrentDimension)>,
    mut writers: Query<(&StreamWriter, &GameModeComponent, &Rotation, &mut Velocity)>,
    mut chunk_calc_writer: MessageWriter<ChunkCalc>,
    mut teleport_writer: MessageWriter<TeleportEntity>,
    mut commands: Commands,
) {
    for change_request in messages.read() {
        let entity = change_request.entity;

        let Ok((mut position, mut dimension)) = entities.get_mut(entity) else {
            warn!(
                "Could not change dimension of {} because it did not have a Position or CurrentDimension component",
                entity
            );
            continue;
        };

        if dimension.0 == change_request.new_dimension {
            warn!(
                "Can't change dimension of entity {}; it's already in the requested dimension",
                entity
            );
            continue;
        }

        *dimension = change_request.new_dimension.into();

        if let Some(spawn_position) = change_request.spawn_position {
            *position = spawn_position;
        }

        if let Ok((stream, game_mode, rot, mut vel)) = writers.get_mut(entity) {
            stream
                .send_packet(RespawnPacket {
                    dimension_type: change_request.new_dimension.as_var_int(),
                    dimension_name: change_request.new_dimension.as_identifier(),
                    seed_hash: 0,
                    gamemode: game_mode.0,
                    previous_gamemode: -1,
                    is_debug: false,
                    is_flat: false,
                    has_death_location: false,
                    death_dimension_name: None,
                    death_location: None,
                    portal_cooldown: VarInt::new(20),
                    sea_level: VarInt::new(0),
                    data_kept: 3, // keep attributes and metadata
                })
                .unwrap_or_else(|err| warn!("failed to send packet to player: {err}"));

            stream
                .send_packet(GameEventPacket::start_waiting_for_level_chunks())
                .unwrap_or_else(|err| warn!("failed to send packet to player: {err}"));

            chunk_calc_writer.write(ChunkCalc(entity));
            *vel = Velocity::new(0.0, 0.0, 0.0);

            commands.entity(entity).insert((
                TeleportTracker {
                    waiting_for_confirm: false,
                },
                OldPosition::new(position.coords.x, position.coords.y, position.coords.z),
            ));

            teleport_writer.write(TeleportEntity {
                position: *position,
                rotation: *rot,
                velocity: *vel,
                entity,
            });
        }
    }
}
