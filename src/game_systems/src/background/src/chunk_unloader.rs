use bevy_ecs::prelude::{Commands, Entity, Has, MessageWriter, Query, Res};
use std::collections::HashSet;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::player::chunk_receiver::ChunkReceiver;
use temper_components::player::player_marker::PlayerMarker;
use temper_core::dimension::Dimension::Overworld;
use temper_core::pos::ChunkPos;
use temper_entities::MobKind;
use temper_messages::DespawnMob;
use temper_state::GlobalStateResource;
use tracing::{error, trace};

pub fn handle(
    state: Res<GlobalStateResource>,
    query: Query<&ChunkReceiver>,
    mut cmd: Commands,
    entity_query: Query<(Entity, &LastChunkPos, Has<PlayerMarker>, Has<MobKind>)>,
    mut despawn_mobs: MessageWriter<DespawnMob>,
) {
    let mut visible_chunks = HashSet::new();

    // gather chunks players can see OR are waiting to see
    for chunk_receiver in query.iter() {
        for &(x, z) in &chunk_receiver.loaded {
            visible_chunks.insert(ChunkPos::new(x, z));
        }

        // this protects chunks currently being generated/sent
        for &(x, z) in &chunk_receiver.loading {
            visible_chunks.insert(ChunkPos::new(x, z));
        }

        // this protects chunks waiting for block updates
        for &(x, z) in &chunk_receiver.dirty {
            visible_chunks.insert(ChunkPos::new(x, z));
        }
    }

    // map all chunks currently in the cache
    let mut all_chunks = HashSet::new();
    for entry in state.0.world.get_cache().iter() {
        all_chunks.insert(entry.key().0);
    }

    let mut unloaded_entries = 0;
    let mut written_chunks = 0;

    // unload anything not in the visible/pending set
    // if 0 players are online, visible_chunks is empty, so this should gracefully unload the entire server.
    for chunk_pos in all_chunks.difference(&visible_chunks) {
        if let Some(((pos, dim), chunk)) =
            state.0.world.get_cache().remove(&(*chunk_pos, Overworld))
        {
            // cancel any pending generation for this chunk
            state.0.world.chunk_generator.forget_chunk(dim, pos);

            // despawn orphaned entities
            for (entity, last_chunk, is_player, is_mob) in entity_query.iter() {
                if is_player || last_chunk.0 != pos {
                    continue;
                }

                if is_mob {
                    despawn_mobs.write(DespawnMob {
                        entity,
                        remove_from_chunk: false,
                    });
                } else {
                    cmd.entity(entity).despawn();
                }
            }

            // save state
            if chunk.is_dirty() {
                if let Err(err) = state.0.world.insert_chunk(pos, dim, chunk) {
                    error!("Failed to write chunk {:?} back to storage: {:?}", pos, err);
                } else {
                    written_chunks += 1;
                }
            }

            unloaded_entries += 1;
        }
    }

    if unloaded_entries > 0 {
        trace!(
            "Unloaded {} chunks ({} written to disk). {} chunks remain in cache.",
            unloaded_entries,
            written_chunks,
            state.0.world.get_cache().len()
        );
    }
}
