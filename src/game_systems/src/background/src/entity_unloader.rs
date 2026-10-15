use bevy_ecs::prelude::{MessageWriter, Query, Res};
use std::collections::HashSet;
use temper_components::player::chunk_receiver::ChunkReceiver;
use temper_core::pos::FullChunkPos;
use temper_state::GlobalStateResource;

pub fn handle(
    state: Res<GlobalStateResource>,
    query: Query<&ChunkReceiver>,
    mut save_entity_writer: MessageWriter<temper_messages::save_chunk_entities::SaveChunkEntities>,
) {
    let mut all_chunks: HashSet<FullChunkPos> = HashSet::new();
    let mut visible_chunks = HashSet::new();
    'chunk_iter: for chunk_candidate in state.0.world.get_cache() {
        let (k, _v) = chunk_candidate.pair();
        all_chunks.insert(*k);
        for chunk_receiver in query.iter() {
            if chunk_receiver.loaded.contains(k) {
                visible_chunks.insert(*k);
                continue 'chunk_iter;
            }
        }
    }
    for chunk_pos in all_chunks.difference(&visible_chunks) {
        save_entity_writer.write(temper_messages::save_chunk_entities::SaveChunkEntities(
            *chunk_pos,
        ));
    }
}
