use bevy_ecs::prelude::{Commands, Entity, Has, MessageWriter, Query, Res};
use std::collections::{HashMap, HashSet};
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
    // If there are no connected players, unload all cached chunks
    if query.count() == 0 {
        let mut removed = 0;
        let chunk_mapped_entities: HashMap<ChunkPos, Vec<Entity>> = entity_query
            .iter()
            .filter(|(_, _, is_player, _)| !is_player)
            .map(|(entity, last_chunk, _, _)| (last_chunk.0, entity))
            .fold(HashMap::new(), |mut acc, (chunk_pos, entity)| {
                acc.entry(chunk_pos).or_insert_with(Vec::new).push(entity);
                acc
            });
        for (pos, entities) in chunk_mapped_entities {
            let chunk = state
                .0
                .world
                .get_cache()
                .get(&(pos, Overworld))
                .expect("Chunk position from entity last chunk pos should exist in cache.");
            removed += 1;
            for (entity, _last_chunk, _is_player, is_mob) in entities.iter().map(|entity| {
                entity_query
                    .get(*entity)
                    .expect("Entity from chunk mapped entities should exist in entity query.")
            }) {
                trace!(
                    "Unloading live entity {:?} from chunk {:?} as no players are connected.",
                    entity,
                    pos
                );
                if is_mob {
                    despawn_mobs.write(DespawnMob {
                        entity,
                        remove_from_chunk: false,
                    });
                } else {
                    cmd.entity(entity).despawn();
                }
            }

            // Write chunks back to the world storage
            if chunk.is_dirty() {
                if let Err(err) = state.0.world.insert_chunk(pos, Overworld, chunk.clone()) {
                    error!(
                        "Failed to write chunk at position {:?} back to world storage: {:?}",
                        pos, err
                    );
                }
                continue;
            }
        }
        // Clear the entire cache
        state.0.world.get_cache().clear();
        state.0.world.chunk_generator.forget_all();
        // Log how many chunks were removed
        if removed > 0 {
            trace!(
                "Unloaded {} chunks from cache as there are no connected players.",
                removed
            );
        }
        return;
    }
    let mut all_chunks: HashSet<ChunkPos> = HashSet::new();
    let mut visible_chunks = HashSet::new();
    'chunk_iter: for chunk_candidate in state.0.world.get_cache() {
        let (k, _v) = chunk_candidate.pair();
        // Track all chunk positions seen in the cache
        all_chunks.insert(k.0);
        // Track chunks that are visible to any connected player
        for chunk_receiver in query.iter() {
            if chunk_receiver.loaded.contains(&(k.0.x(), k.0.z())) {
                visible_chunks.insert(k.0);
                continue 'chunk_iter;
            }
        }
    }
    let mut unloaded_entries = 0;
    let mut written_chunks = 0;
    // The difference is the set of chunks that are in the cache but not visible to any player
    for chunk_pos in all_chunks.difference(&visible_chunks) {
        let removed_chunk = state.0.world.get_cache().remove(&(*chunk_pos, Overworld));
        match removed_chunk {
            Some(((pos, dim), chunk)) => {
                state.0.world.chunk_generator.forget_chunk(dim, pos);
                for (entity, last_chunk, is_player, is_mob) in entity_query.iter() {
                    if is_player || last_chunk.0 != *chunk_pos {
                        continue;
                    }

                    trace!(
                        "Unloading live entity {:?} from chunk {:?} as it is no longer visible to any player.",
                        entity, chunk_pos
                    );
                    if is_mob {
                        despawn_mobs.write(DespawnMob {
                            entity,
                            remove_from_chunk: false,
                        });
                    } else {
                        cmd.entity(entity).despawn();
                    }
                }
                let dirty = chunk.is_dirty();
                if dirty {
                    state
                        .0
                        .world
                        .insert_chunk(pos, dim, chunk)
                        .expect("Failed to re-insert chunk after unloading from cache.");
                    written_chunks += 1;
                }
                unloaded_entries += 1;
            }
            None => {
                error!(
                    "Chunk at position {:?} could not be removed because it does not exist in the cache.",
                    chunk_pos
                );
            }
        }
    }
    let remaining_chunks = state.0.world.get_cache().len();
    trace!(
        "Unloaded {} chunks from cache ({} written to world). {} chunks remain in cache.",
        unloaded_entries,
        written_chunks,
        remaining_chunks
    );
}
