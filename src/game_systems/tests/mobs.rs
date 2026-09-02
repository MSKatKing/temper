#[path = "mobs/chunk_visibility_lifecycle.rs"]
mod chunk_visibility_lifecycle;
#[path = "mobs/cross_chunk_persistence.rs"]
mod cross_chunk_persistence;
#[path = "mobs/entity_persistence.rs"]
mod entity_persistence;
#[path = "mobs/player_distance_reload.rs"]
mod player_distance_reload;
#[path = "mobs/spawn_mob_bundle.rs"]
mod spawn_mob_bundle;

use temper_core::pos::ChunkPos;
use temper_state::GlobalStateResource;
use temper_world::{Dimension, chunks::load_chunk_internal};
use temper_world_format::Chunk;

/// `chunk_unloader` dispatches storage writes to the thread pool, so a chunk
/// evicted from the cache is not immediately readable from storage. Poll the
/// storage backend directly — going through `get_chunk` would hit the cache
/// and could pass without the write having landed.
pub fn wait_for_saved_chunk(state: &GlobalStateResource, pos: ChunkPos) -> Chunk {
    for _ in 0..200 {
        if let Ok(chunk) = load_chunk_internal(
            &state.0.world.chunks.storage_backend,
            pos,
            Dimension::Overworld,
            false,
        ) {
            return chunk;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("chunk {pos:?} never reached storage after unload");
}
