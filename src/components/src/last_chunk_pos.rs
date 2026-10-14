use bevy_ecs::prelude::Component;
use temper_core::pos::{ChunkPos, FullChunkPos};

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct LastChunkPos(pub FullChunkPos);

impl LastChunkPos {
    pub fn new(chunk: FullChunkPos) -> Self {
        Self(chunk)
    }
}
