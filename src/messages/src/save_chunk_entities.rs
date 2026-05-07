use bevy_ecs::prelude::Message;
use temper_core::pos::ChunkPos;

#[derive(Message)]
pub struct SaveChunkEntities(pub ChunkPos);
