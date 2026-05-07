use bevy_ecs::prelude::{Entity, Has, MessageReader, MessageWriter, Query, Res};
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_core::dimension::Dimension::Overworld;
use temper_messages::chunk_calc::ChunkCalc;
use temper_messages::cross_chunk_boundary_event::ChunkBoundaryCrossed;
use temper_state::GlobalStateResource;
use temper_world::WorldError;
use tracing::error;

pub fn cross_chunk_border(
    mut chunk_cross_events: MessageReader<ChunkBoundaryCrossed>,
    query: Query<(Entity, &Identity, Has<PlayerMarker>)>,
    mut chunk_calc_messages: MessageWriter<ChunkCalc>,
    state: Res<GlobalStateResource>,
) {
    'ev_loop: for event in chunk_cross_events.read() {
        let (entity, identity, is_player) = query
            .get(event.entity)
            .expect("Entity in ChunkBoundaryCrossed event does not exist");
        // If it's a player, send the chunk calc message
        if is_player {
            chunk_calc_messages.write(ChunkCalc(entity));
        } else {
            // For mobs, we update the chunk they are saved in
            let old_chunk_cords = event.old_chunk;
            let new_chunk_cords = event.new_chunk;
            // Pull out the entity data from the old chunk. We have to do it this way cos holding locks on multiple chunks at once can easily deadlock
            let Some(extracted_old_data) = ({
                let chunk = state.0.world.get_chunk(old_chunk_cords, Overworld);
                match chunk {
                    Ok(chunk) => {
                        let data = chunk.entities.remove(&identity.uuid);
                        chunk.mark_dirty();
                        data
                    }
                    Err(WorldError::ChunkNotFound) => {
                        error!(
                            "Invalid old chunk coordinates in ChunkBoundaryCrossed event for entity {}: {:?}",
                            entity, old_chunk_cords
                        );
                        continue 'ev_loop;
                    }
                    Err(e) => {
                        error!(
                            "Error accessing old chunk in ChunkBoundaryCrossed event for entity {}: {:?}",
                            entity, e
                        );
                        continue 'ev_loop;
                    }
                }
            }) else {
                error!(
                    "Entity {} not found in old chunk during ChunkBoundaryCrossed event",
                    entity
                );
                continue 'ev_loop;
            };
            // If the server crashes here, bye bye entity
            {
                let chunk = state
                    .0
                    .world
                    .get_or_generate_chunk(new_chunk_cords, Overworld)
                    .expect("Failed to get or generate new chunk in ChunkBoundaryCrossed event");
                chunk.entities.insert(identity.uuid, extracted_old_data.1);
                chunk.mark_dirty();
            }
        }
    }
}
