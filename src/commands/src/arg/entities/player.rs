use bevy_ecs::prelude::Entity;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;

pub(crate) fn resolve_player_name<'a>(
    name: String,
    iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
) -> Option<Entity> {
    for (entity, id, player_marker) in iter {
        if player_marker.is_some()
            && id
                .name
                .as_ref()
                .map(|n| n.eq_ignore_ascii_case(&name))
                .unwrap_or(false)
        {
            return Some(entity);
        }
    }
    None
}
