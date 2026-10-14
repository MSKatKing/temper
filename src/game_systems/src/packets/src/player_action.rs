use bevy_ecs::prelude::{Entity, MessageWriter, Query, Res};
use temper_components::player::abilities::PlayerAbilities;
use temper_messages::BlockBrokenEvent;
use temper_messages::player_digging::*;

use temper_blocks::BlockDispatch;
use temper_codec::net_types::network_position::NetworkPosition;
use temper_codec::net_types::var_int::VarInt;
use temper_components::player::dimension::CurrentDimension;
use temper_core::block_state_id::BlockStateId;
use temper_core::pos::BlockPos;
use temper_macros::block;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::PlayerActionReceiver;
use temper_protocol::outgoing::block_change_ack::BlockChangeAck;
use temper_protocol::outgoing::block_update::BlockUpdate;
use temper_state::GlobalStateResource;
use tracing::{error, warn};

pub fn handle(
    receiver: Res<PlayerActionReceiver>,
    state: Res<GlobalStateResource>,
    broadcast_query: Query<(Entity, &StreamWriter, &CurrentDimension)>,
    player_query: Query<(&PlayerAbilities, &CurrentDimension)>,
    (
        mut start_dig_events,
        mut cancel_dig_events,
        mut finish_dig_events,
        mut block_break_events,
        mut world_change,
    ): (
        MessageWriter<PlayerStartedDigging>,
        MessageWriter<PlayerCancelledDigging>,
        MessageWriter<PlayerFinishedDigging>,
        MessageWriter<BlockBrokenEvent>,
        MessageWriter<temper_messages::world_change::WorldChange>,
    ),
) {
    // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol?oldid=2773393#Player_Action
    for (event, trigger_eid) in receiver.0.try_iter() {
        // Get the player's abilities to check their gamemode
        let Ok((abilities, dim)) = player_query.get(trigger_eid) else {
            warn!(
                "PlayerAction: Player {:?} has no PlayerAbilities component",
                trigger_eid
            );
            continue;
        };

        let pos: BlockPos = event.location.clone().into();
        let pos = pos.full(dim.0);
        if abilities.creative_mode {
            // --- CREATIVE MODE LOGIC ---
            // Only instabreak (status 0) is relevant in creative.
            if event.status.0 == 0 {
                world_change.write(temper_messages::world_change::WorldChange {
                    chunk: Some(pos.full_chunk()),
                });

                let Ok(id) = state.0.world.get_block(pos) else {
                    error!("Couldn't get block at pos {}", pos);
                    continue;
                };

                let mut broken_positions = id.try_break(&state.0.world, pos);
                broken_positions.blocks.push(*pos);

                for pos in &broken_positions.blocks {
                    let pos = pos.full(broken_positions.dimension);
                    block_break_events.write(BlockBrokenEvent { position: *pos });

                    if let Err(world_error) = state.0.world.set_block(pos, block!("air")) {
                        error!("Failed to break block at {}: {}", pos, world_error)
                    }
                }

                // Broadcast the change
                for (eid, conn, dim) in &broadcast_query {
                    if !state.0.players.is_connected(eid) {
                        continue;
                    }

                    if broken_positions.dimension != dim.0 {
                        continue;
                    }

                    for broken_pos in &broken_positions.blocks {
                        // TODO: check for in range

                        let update = BlockUpdate {
                            location: NetworkPosition {
                                x: broken_pos.pos.x,
                                y: broken_pos.pos.y as i16,
                                z: broken_pos.pos.z,
                            },
                            block_state_id: VarInt::from(BlockStateId::default()),
                        };

                        if let Err(e) = conn.send_packet_ref(&update) {
                            error!(
                                "Failed to send block update to {:?} for broken block: {:?}",
                                eid, e
                            );
                        }
                    }

                    if eid == trigger_eid {
                        // Send ACK to the creative player
                        let ack_packet = BlockChangeAck {
                            sequence: event.sequence,
                        };
                        if let Err(e) = conn.send_packet_ref(&ack_packet) {
                            error!("Failed to send block change ACK: {:?}", e);
                        }
                    }
                }
            };
        } else {
            // --- SURVIVAL MODE LOGIC ---
            // This handler's only job is to fire messages.
            match event.status.0 {
                0 => {
                    // Started digging
                    start_dig_events.write(PlayerStartedDigging {
                        player: trigger_eid,
                        position: event.location,
                        sequence: event.sequence,
                    });
                }
                1 => {
                    // Cancelled digging
                    cancel_dig_events.write(PlayerCancelledDigging {
                        player: trigger_eid,
                        sequence: event.sequence,
                    });
                }
                2 => {
                    // Finished digging
                    finish_dig_events.write(PlayerFinishedDigging {
                        player: trigger_eid,
                        position: event.location,
                        sequence: event.sequence,
                        dimension: dim.0,
                    });
                }
                _ => {} // Other statuses (drop item, etc.) are handled by different packets
            }
        }
    }
}
