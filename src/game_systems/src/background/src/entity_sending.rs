use bevy_ecs::prelude::{Entity, Has, Query};
use temper_components::entity_identity::Identity;
use temper_components::player::client_information::ClientInformationComponent;
use temper_components::player::entity_tracker::EntityTracker;
use temper_components::player::player_marker::PlayerMarker;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_config::server_config::get_global_config;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::remove_entities::RemoveEntitiesPacket;
use temper_protocol::outgoing::spawn_entity::SpawnEntityPacket;
use tracing::debug;

/// Protocol entity type ID for player entities in the current target version.
const PLAYER_TYPE_ID: i32 = 149;

pub fn send_untracked_entities(
    mut player_query: Query<(&StreamWriter, &mut EntityTracker)>,
    identity_query: Query<&Identity>,
) {
    for (conn, entity_tracker) in player_query.iter_mut() {
        while let Some(entity) = entity_tracker.to_untrack.pop() {
            let Ok(identity) = identity_query.get(entity) else {
                continue;
            };

            let packet = RemoveEntitiesPacket::from_entities(std::iter::once(identity.clone()));
            conn.send_packet(packet)
                .expect("Failed to send remove entities packet");
        }
    }
}

pub fn send_new_entities(
    mut player_query: Query<(
        &StreamWriter,
        &mut EntityTracker,
        &Position,
        &ClientInformationComponent,
    )>,
    entity_query: Query<(Entity, &Identity, &Position, &Rotation, Has<PlayerMarker>)>,
) {
    for (conn, mut entity_tracker, player_pos, client_info) in player_query.iter_mut() {
        let mut unresolved = Vec::new();

        while let Some((uuid, entity_type_id)) = entity_tracker.to_track.pop() {
            if let Some((entity, identity, entity_pos, rot, is_player)) = entity_query
                .iter()
                .find_map(|(entity, identity, pos, rot, is_player)| {
                    if identity.uuid == uuid {
                        Some((entity, identity, pos, rot, is_player))
                    } else {
                        None
                    }
                })
            {
                if entity_tracker.tracking.contains(&entity) {
                    continue;
                }

                let render_distance = client_info
                    .view_distance
                    .min(get_global_config().chunk_render_distance as u8);
                if player_pos.distance(**entity_pos) > (f64::from(render_distance) * 16.0) {
                    continue; // Skip entities outside of render distance
                }

                let entity_type_id = if is_player {
                    PLAYER_TYPE_ID
                } else {
                    i32::from(entity_type_id)
                };

                let packet = SpawnEntityPacket::new(
                    identity.entity_id,
                    identity.uuid.as_u128(),
                    entity_type_id,
                    entity_pos,
                    rot,
                );
                conn.send_packet(packet)
                    .expect("Failed to send spawn entity packet");
                debug!(
                    "Sent spawn packet for entity {} with UUID {} to player at position {:?}",
                    identity.entity_id,
                    identity.uuid,
                    player_pos.xyz()
                );
                entity_tracker.tracking.insert(entity);
            } else {
                // Retry unresolved entities on a later tick instead of reinserting
                // into the actively-drained queue and looping forever.
                unresolved.push((uuid, entity_type_id));
            }
        }

        for item in unresolved {
            entity_tracker.to_track.push(item);
        }
    }
}
