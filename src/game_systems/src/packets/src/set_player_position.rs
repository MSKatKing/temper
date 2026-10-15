use bevy_ecs::prelude::{Commands, Entity, MessageWriter, Query, Res};
use temper_components::entity_identity::Identity;
use temper_components::player::grounded::OnGround;
use temper_components::player::old_position::OldPosition;
use temper_components::player::position::Position;
use temper_components::player::teleport_tracker::TeleportTracker;
use temper_messages::cross_chunk_boundary_event::ChunkBoundaryCrossed;
use temper_messages::packet_messages::Movement;
use temper_protocol::SetPlayerPositionPacketReceiver;
use tracing::trace;

pub fn handle(
    receiver: Res<SetPlayerPositionPacketReceiver>,
    mut query: Query<(
        Entity,
        &mut Position,
        &mut OnGround,
        &TeleportTracker,
        &Identity,
        Option<&mut OldPosition>,
    )>,
    mut movement_messages: MessageWriter<Movement>,
    mut cross_chunk_border_msg: MessageWriter<ChunkBoundaryCrossed>,
    mut cmd: Commands,
) {
    for (event, eid) in receiver.0.try_iter() {
        if let Ok((entity, mut pos, mut ground, tracker, identity, old_pos)) = query.get_mut(eid) {
            if tracker.waiting_for_confirm {
                // Ignore position updates while waiting for teleport confirmation
                continue;
            }
            let new_pos = Position::new(event.x, event.feet_y, event.z, pos.dimension);

            // Check if chunk changed
            let old_chunk = pos.chunk();
            let new_chunk = new_pos.chunk();
            if old_chunk != new_chunk {
                cross_chunk_border_msg.write(ChunkBoundaryCrossed {
                    entity,
                    old_chunk,
                    new_chunk,
                });
            }

            // Build movement message with delta BEFORE updating component
            let movement = Movement::new(eid)
                .position_delta_from(&pos, &new_pos)
                .on_ground(event.on_ground);

            // Update components
            if pos.coords != new_pos.coords {
                if let Some(mut old_pos) = old_pos {
                    *old_pos = OldPosition::from(*pos);
                } else {
                    cmd.entity(entity).insert(OldPosition::from(*pos));
                }
                *pos = new_pos;
            }
            ground.set_grounded(event.on_ground);

            // Send movement message for broadcasting
            movement_messages.write(movement);

            trace!(
                "Updated position for player {}: ({:.2}, {:.2}, {:.2})",
                identity.name.as_ref().unwrap(),
                event.x,
                event.feet_y,
                event.z
            );
        }
    }
}
