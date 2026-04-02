use bevy_ecs::prelude::Entity;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;

pub(crate) fn resolve_any_entity<'a>(
    iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
) -> Vec<Entity> {
    iter.map(|(entity, _, _)| entity).collect()
}
