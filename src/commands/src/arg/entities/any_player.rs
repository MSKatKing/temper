use bevy_ecs::entity::Entity;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;

pub(crate) fn resolve_any_player<'a>(
    iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
) -> Vec<Entity> {
    let mut players = Vec::new();
    for (entity, _, player_marker) in iter {
        if player_marker.is_none() {
            continue;
        }
        players.push(entity);
    }
    players
}
