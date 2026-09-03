use bevy_ecs::prelude::{Entity, MessageWriter, Query, Res};
use bevy_math::{IVec2, IVec3};
use std::cmp::max;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use temper_codec::encode::NetEncodeOpts;
use temper_components::player::chunk_receiver::{ChunkReceiver, PreparedChunk};
use temper_components::player::client_information::ClientInformationComponent;
use temper_components::player::entity_tracker::EntityTracker;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_core::pos::ChunkPos;
use temper_net_runtime::compression::compress_packet;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::chunk_and_light_data::ChunkAndLightData;
use temper_protocol::outgoing::chunk_batch_finish::ChunkBatchFinish;
use temper_protocol::outgoing::chunk_batch_start::ChunkBatchStart;
use temper_protocol::outgoing::set_center_chunk::SetCenterChunk;
use temper_state::GlobalStateResource;
use tracing::error;

/// How many times a chunk may fail to prepare before we stop retrying it.
const MAX_CHUNK_RETRIES: u8 = 3;

pub fn handle(
    mut query: Query<(
        Entity,
        &StreamWriter,
        &mut ChunkReceiver,
        &Position,
        &ClientInformationComponent,
        &EntityTracker,
    )>,
    state: Res<GlobalStateResource>,
    mut mob_load_writer: MessageWriter<temper_messages::load_chunk_entities::LoadChunkEntities>,
) {
    for (eid, conn, mut chunk_receiver, pos, client_info, entity_tracker) in query.iter_mut() {
        if !state.0.players.is_connected(eid) {
            continue;
        }

        let chunk_receiver = &mut *chunk_receiver;

        // ==========================================
        // PHASE 1: HARVEST & SEND COMPLETED CHUNKS
        // ==========================================
        let mut harvested = Vec::new();

        // Pull everything the pool has finished with — successes and failures
        while let Ok(prepared) = chunk_receiver.ready_rx.try_recv() {
            harvested.push(prepared);
        }

        // Anything harvested is no longer in flight, whatever its outcome.
        // This is what stops a failure from stranding the coordinate forever.
        for prepared in &harvested {
            let pos = prepared.pos();
            chunk_receiver.in_flight.remove(&(pos.x(), pos.z()));
        }

        let mut ready_to_send = Vec::new();
        for prepared in harvested {
            match prepared {
                PreparedChunk::Ready { pos, .. } => {
                    // Success wipes the failure history for this chunk, so a
                    // chunk that fails twice then succeeds starts clean.
                    chunk_receiver.retry_counts.remove(&(pos.x(), pos.z()));
                    ready_to_send.push(prepared);
                }
                PreparedChunk::Failed { pos } => {
                    let key = (pos.x(), pos.z());
                    let attempts = chunk_receiver.retry_counts.entry(key).or_insert(0);
                    *attempts += 1;

                    if *attempts >= MAX_CHUNK_RETRIES {
                        error!(
                            "Chunk {:?} failed to prepare {} times; giving up for this session",
                            pos, attempts
                        );
                        chunk_receiver.retry_counts.remove(&key);
                    } else {
                        // Put it back in the queue for another attempt next tick.
                        chunk_receiver.loading.push_back(key);
                    }
                }
            }
        }

        if !ready_to_send.is_empty() {
            conn.send_packet(ChunkBatchStart {})
                .expect("Failed to send ChunkBatchStart");

            let center_chunk: IVec3 = pos.coords.floor().as_ivec3() >> 4;
            conn.send_packet(SetCenterChunk {
                x: center_chunk.x.into(),
                z: center_chunk.z.into(),
            })
            .expect("Failed to send SetCenterChunk");

            let packets_len = ready_to_send.len();

            for chunk in ready_to_send {
                let PreparedChunk::Ready {
                    pos: chunk_pos,
                    packet_data,
                    entities,
                    is_new_load,
                } = chunk
                else {
                    unreachable!();
                };

                chunk_receiver.loaded.insert((chunk_pos.x(), chunk_pos.z()));

                if is_new_load {
                    mob_load_writer.write(temper_messages::load_chunk_entities::LoadChunkEntities(
                        chunk_pos,
                    ));
                }

                if let Err(err) = conn.send_raw_packet(packet_data) {
                    error!("Failed to send chunk packet: {:?}", err);
                }

                for entity_tuple in entities {
                    entity_tracker.to_track.push(entity_tuple);
                }
            }

            if let Err(err) = conn.send_packet(ChunkBatchFinish {
                batch_size: packets_len.into(),
            }) {
                error!("Failed to send ChunkBatchFinish packet: {:?}", err);
            }
        }

        // tell the client to unload chunks that are no longer needed
        while let Some(coords) = chunk_receiver.unloading.pop_front() {
            let packet = temper_protocol::outgoing::unload_chunk::UnloadChunk {
                x: coords.0,
                z: coords.1,
            };
            if let Err(err) = conn.send_packet(packet) {
                error!("Failed to send UnloadChunk packet: {:?}", err);
            }
        }

        // ==========================================
        // PHASE 2: DISPATCH NEW CHUNKS (NON-BLOCKING)
        // ==========================================
        let chunk_per_tick = match state.0.config.performance.chunks_per_tick {
            0 => max(
                chunk_receiver.loading.len() / 3,
                state.0.config.performance.chunks_per_tick_min as usize,
            ),
            -1 => usize::MAX,
            hard_limit => hard_limit as usize,
        };

        if chunk_receiver.dirty.is_empty() && chunk_receiver.loading.is_empty() {
            continue;
        }

        let mut dirty_chunks = Vec::new();
        let mut sent_chunks = 0;

        while let Some(coords) = chunk_receiver.dirty.pop_front() {
            dirty_chunks.push(coords);
            sent_chunks += 1;
            if sent_chunks >= chunk_per_tick {
                break;
            }
        }

        let mut needed_chunks: Vec<(i32, i32)> = Vec::new();

        if sent_chunks < chunk_receiver.chunks_per_tick as usize {
            while let Some(coords) = chunk_receiver.loading.pop_front() {
                needed_chunks.push(coords);
                sent_chunks += 1;
                if sent_chunks >= chunk_per_tick {
                    break;
                };
            }
        }

        let loading_chunks: HashSet<_> = needed_chunks.iter().copied().collect();
        needed_chunks.extend(dirty_chunks);

        if needed_chunks.is_empty() {
            continue;
        }

        // dispatch to the thread pool
        for coordinates in needed_chunks
            .into_iter()
            .filter(|coord| {
                let chunk_pos = IVec2::new(coord.0, coord.1);
                let player_chunk_pos = IVec2::new(
                    pos.coords.x.floor() as i32 >> 4,
                    pos.coords.z.floor() as i32 >> 4,
                );
                let distance = chunk_pos.distance_squared(player_chunk_pos);
                let view_distance = max(
                    u32::from(client_info.view_distance),
                    state.0.config.chunk_render_distance,
                );
                distance <= (view_distance * view_distance) as i32
            })
            .map(|c| ChunkPos::new(c.0, c.1))
        {
            let is_new_load = loading_chunks.contains(&(coordinates.x(), coordinates.z()));
            let is_compressed = conn.compress.load(Ordering::Relaxed);

            let state_clone = state.clone();
            let tx = chunk_receiver.ready_tx.clone();

            chunk_receiver
                .in_flight
                .insert((coordinates.x(), coordinates.z())); // Mark this chunk as in-flight to prevent duplicate generation

            state.0.thread_pool.oneshot(move || {
                // Inner closure so we can use `?` on the three fallible steps
                // instead of unwinding the whole pool thread on any of them.
                let prepare = || -> Result<PreparedChunk, String> {
                    let chunk_data = {
                        let chunk_ref = state_clone
                            .0
                            .world
                            .get_or_generate_chunk(coordinates, Dimension::Overworld)
                            .map_err(|err| format!("load or generate failed: {err:?}"))?;

                        (*chunk_ref).clone_without_transient_noise()
                    };

                    let mut entities = Vec::new();
                    for kv in chunk_data.entities.iter() {
                        entities.push((*kv.key(), kv.value().0.to_entity_type().id));
                    }

                    let packet = ChunkAndLightData::from_chunk(coordinates, &chunk_data)
                        .map_err(|err| format!("building ChunkAndLightData failed: {err:?}"))?;

                    let packet_data = compress_packet(
                        &packet,
                        is_compressed,
                        &NetEncodeOpts::WithLength,
                        state_clone.0.config.network_compression_threshold as usize,
                    )
                    .map_err(|err| format!("compressing chunk packet failed: {err:?}"))?;

                    Ok(PreparedChunk::Ready {
                        pos: coordinates,
                        packet_data,
                        entities,
                        is_new_load,
                    })
                };

                let message = prepare().unwrap_or_else(|err| {
                    error!("Chunk {:?} failed to prepare: {}", coordinates, err);
                    PreparedChunk::Failed { pos: coordinates }
                });

                let _ = tx.send(message);
            });
        }
    }
}
