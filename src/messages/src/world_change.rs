use bevy_ecs::prelude::Message;
use temper_core::pos::FullChunkPos;

#[derive(Message)]
pub struct WorldChange {
    pub chunk: Option<FullChunkPos>,
}
