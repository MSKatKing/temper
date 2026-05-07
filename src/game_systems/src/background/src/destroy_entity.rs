use bevy_ecs::prelude::{Commands, Entity, Has, MessageReader, Query, Res};
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension::Overworld;
use temper_messages::destroy_entity::DestroyEntity;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::remove_entities::RemoveEntitiesPacket;
use temper_protocol::outgoing::system_message::SystemMessagePacket;
use temper_state::GlobalStateResource;
use temper_text::{Color, NamedColor, TextComponentBuilder};
use tracing::trace;

pub fn destroy_entity_system(
    mut commands: Commands,
    mut destroy_entity_events: MessageReader<DestroyEntity>,
    query: Query<(
        Entity,
        &Position,
        &Identity,
        Has<PlayerMarker>,
        Option<&StreamWriter>,
    )>,
    state: Res<GlobalStateResource>,
) {
    let mut destroyed_entities = Vec::new();
    let killed_message = SystemMessagePacket {
        message: temper_nbt::NBT::new(
            TextComponentBuilder::new("You have been killed. How sad :(")
                .bold()
                .color(Color::Named(NamedColor::Red))
                .build(),
        ),
        overlay: false,
    };

    for event in destroy_entity_events.read() {
        if let Ok((_, position, identity, has_player_marker, conn_opt)) = query.get(event.0) {
            if !has_player_marker {
                destroyed_entities.push(identity.entity_id.into());
                commands.entity(event.0).despawn();
                let Ok(chunk) = state.0.world.get_chunk(position.chunk(), Overworld) else {
                    continue;
                };
                if chunk.entities.remove(&identity.uuid).is_some() {
                    trace!(
                        "Entity {:?} destroyed and removed from chunk",
                        identity.entity_id
                    );
                    chunk.mark_dirty();
                }
                destroyed_entities.push(identity.entity_id.into());
            } else if let Some(conn) = conn_opt
                && let Err(err) = conn.send_packet_ref(&killed_message)
            {
                trace!("Failed to send killed message: {}", err);
            }
        }
    }

    let packet = RemoveEntitiesPacket {
        entity_ids: LengthPrefixedVec::new(destroyed_entities),
    };

    for (_, _, _, has_player_marker, conn_opt) in query.iter() {
        if has_player_marker
            && let Some(conn) = conn_opt
            && let Err(err) = conn.send_packet_ref(&packet)
        {
            trace!("Failed to send RemoveEntitiesPacket: {}", err);
        }
    }
}
