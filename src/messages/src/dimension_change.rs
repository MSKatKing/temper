use bevy_ecs::prelude::{Entity, Message};
use bevy_math::DVec3;
use temper_components::player::rotation::Rotation;
use temper_core::dimension::Dimension;

#[derive(Clone, Debug, Message)]
pub struct DimensionChange {
    pub entity: Entity,
    pub is_respawn: bool,
    pub new_dimension: Dimension,
    pub spawn_position: Option<DVec3>,
    pub spawn_rotation: Option<Rotation>,
}
