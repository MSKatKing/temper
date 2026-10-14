use bevy_ecs::prelude::{Entity, Message};
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;

#[derive(Clone, Debug, Message)]
pub struct DimensionChange {
    pub entity: Entity,
    pub new_dimension: Dimension,
    pub spawn_position: Option<Position>,
}