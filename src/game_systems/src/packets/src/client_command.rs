use bevy_ecs::prelude::{MessageWriter, Query, Res};
use temper_codec::net_types::network_position::NetworkPosition;
use temper_components::player::chunk_receiver::ChunkReceiver;
use temper_components::player::gamemode::GameModeComponent;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_messages::dimension_change::DimensionChange;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::ClientCommandReceiver;
use temper_protocol::incoming::client_command::ClientCommandAction;
use temper_protocol::outgoing::game_event::GameEventPacket;
use temper_protocol::outgoing::respawn::RespawnPacket;
use temper_state::GlobalStateResource;
use tracing::error;
use temper_components::health::Health;

pub fn handle_client_command(
    events: Res<ClientCommandReceiver>,
    mut query: Query<(
        &StreamWriter,
        &GameModeComponent,
        &Position,
        &mut ChunkReceiver,
        &mut Health
    )>,
    mut dim_change_writer: MessageWriter<DimensionChange>,
    state: Res<GlobalStateResource>,
) {
    for (message, sender) in events.0.try_iter() {
        match message.action {
            ClientCommandAction::PerformRespawn => {
                let (conn, gamemode, pos, _chunk_recv, mut health) = query
                    .get_mut(sender)
                    .expect("No StreamWriter or GameModeComponent for sender");

                let packet = RespawnPacket {
                    dimension_type: Dimension::Overworld.as_var_int(),
                    dimension_name: Dimension::Overworld.as_identifier(),
                    seed_hash: 0,
                    gamemode: gamemode.0,
                    previous_gamemode: -1,
                    is_debug: false,
                    is_flat: false,
                    data_kept: 0,
                    has_death_location: true,
                    death_dimension_name: Some(pos.dimension.as_identifier()),
                    death_location: Some(NetworkPosition::from(*pos).as_u64()),
                    portal_cooldown: Default::default(),
                    sea_level: Default::default(),
                };
                if let Err(err) = conn.send_packet(packet) {
                    error!("Failed to send respawn packet: {:?}", err);
                }
                let game_event = GameEventPacket::start_waiting_for_level_chunks();
                if let Err(err) = conn.send_packet(game_event) {
                    error!("Failed to send game event packet: {:?}", err);
                }

                dim_change_writer.write(DimensionChange {
                    entity: sender,
                    is_respawn: true,
                    new_dimension: Dimension::Overworld,
                    spawn_position: Some(
                        state
                            .0
                            .spawn_positions
                            .pop()
                            .expect("No spawn position available")
                            .into(),
                    ),
                    spawn_rotation: Some(Default::default()),
                });
                health.current = health.max;
            }
            ClientCommandAction::GameRules => {}
            ClientCommandAction::RequestStats => {}
        }
    }
}
