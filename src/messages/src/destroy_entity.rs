use bevy_ecs::prelude::{Entity, Message};

#[derive(Message)]
pub struct DestroyEntity(pub Entity);
