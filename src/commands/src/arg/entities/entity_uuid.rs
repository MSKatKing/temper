use bevy_ecs::entity::Entity;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use uuid::Uuid;

pub(crate) fn resolve_uuid<'a>(
    uuid: Uuid,
    iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
) -> Option<Entity> {
    for (entity, entity_id_opt, _) in iter {
        if entity_id_opt.uuid == uuid {
            return Some(entity);
        }
    }
    None
}
