use bevy_ecs::entity::Entity;
use rand::prelude::IteratorRandom;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;

pub(crate) fn resolve_random_player<'a>(
    iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
) -> Option<Entity> {
    let mut rng = rand::rng();
    iter.filter_map(|(entity, _, player_id)| {
        if player_id.is_some() {
            Some(entity)
        } else {
            None
        }
    })
    .choose(&mut rng)
}
