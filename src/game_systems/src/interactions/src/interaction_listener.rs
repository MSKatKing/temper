use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use temper_codec::net_types::network_position::NetworkPosition;
use temper_codec::net_types::var_int::VarInt;
use temper_components::interaction::InteractionCooldown;
use temper_components::player::position::Position;
use temper_core::pos::BlockPos;
use temper_messages::{BlockInteractMessage, BlockToggledEvent, DoorToggledEvent};
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::block_change_ack::BlockChangeAck;
use temper_protocol::outgoing::block_update::BlockUpdate;
use temper_state::GlobalStateResource;
use temper_world::Dimension;
use tracing::{debug, error};
use temper_blocks::BlockDispatch;
use crate::door_interaction::is_open;

pub fn handle_block_interact(
    mut events: MessageReader<BlockInteractMessage>,
    state: Res<GlobalStateResource>,
    query: Query<(Entity, &StreamWriter, &Position)>,
    mut toggled_writer: MessageWriter<BlockToggledEvent>,
    mut door_toggled_writer: MessageWriter<DoorToggledEvent>,
    mut cooldowns: Local<HashMap<BlockPos, Instant>>,
) {
    let cooldown_duration = Duration::from_millis(InteractionCooldown::default().cooldown_ms);

    for event in events.read() {
        let pos = event.position;

        // Ignore rapid repeated interactions on the same block
        if cooldowns
            .get(&pos)
            .is_some_and(|t| t.elapsed() < cooldown_duration)
        {
            if let Ok((_, conn, _)) = query.get(event.player) {
                let ack = BlockChangeAck {
                    sequence: event.sequence,
                };
                if let Err(e) = conn.send_packet_ref(&ack) {
                    error!("Failed to send BlockChangeAck (cooldown): {:?}", e);
                }
            }
            continue;
        }
        cooldowns.insert(pos, Instant::now());

        // Load the chunk and get current block state
        let mut block_state = state.0.world.get_chunk(pos.chunk(), Dimension::Overworld).map(|chunk| chunk.get_block(pos.chunk_block_pos())).unwrap();
        let original = block_state.clone();

        let (updates, is_active) = {
            let updates = block_state.interact(&state.0.world, pos);
            let mut updates = updates.blocks;
            updates.insert(pos, block_state);

            debug!(
                "Block interact: toggled ({}, {}, {}) from {} to {}",
                pos.pos.x,
                pos.pos.y,
                pos.pos.z,
                original.raw(),
                block_state.raw()
            );

            for (pos, block_state) in &updates {
                if !state.0.world
                    .get_chunk_mut(pos.chunk(), Dimension::Overworld)
                    .map(|mut chunk| chunk.set_block(pos.chunk_block_pos(), *block_state))
                    .is_ok() {
                    error!("Attempted to update block at {} to {} but failed. (interaction failure)", pos, block_state.raw());
                }
            }

            let updates = updates
                .into_iter()
                .map(|(pos, state)| BlockUpdate {
                    location: NetworkPosition {
                        x: pos.pos.x,
                        y: pos.pos.y as i16,
                        z: pos.pos.z,
                    },
                    block_state_id: VarInt::from(state),
                })
                .collect::<Vec<_>>();

            let is_active = is_open(block_state).unwrap_or(false);
            (updates, is_active)
        }; // chunk lock released here

        // Emit BlockToggledEvent for other systems to react
        toggled_writer.write(BlockToggledEvent {
            player: event.player,
            position: pos,
            is_active,
        });

        // Send BlockChangeAck to the player
        if let Ok((_, conn, _)) = query.get(event.player) {
            let ack = BlockChangeAck {
                sequence: event.sequence,
            };
            if let Err(e) = conn.send_packet_ref(&ack) {
                error!("Failed to send BlockChangeAck: {:?}", e);
            }
        }

        // Broadcast BlockUpdate to all players within render distance
        let block_chunk = pos.chunk();
        let (block_cx, block_cz) = (block_chunk.x(), block_chunk.z());
        let render_distance = state.0.config.chunk_render_distance as i32;

        for (_, conn, player_pos) in query.iter() {
            let pchunk = player_pos.chunk();
            let (pcx, pcz) = (pchunk.x(), pchunk.z());

            if (block_cx - pcx).abs() <= render_distance
                && (block_cz - pcz).abs() <= render_distance
            {
                for update in &updates {
                    if let Err(e) = conn.send_packet_ref(update) {
                        error!("Failed to send block update: {:?}", e);
                    }
                }
            }
        }
    }
}
