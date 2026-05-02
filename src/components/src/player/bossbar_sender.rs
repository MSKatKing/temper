use bevy_ecs::component::Component;
use bitcode_derive::{Decode, Encode};
use type_hash::TypeHash;
use uuid::Uuid;

#[derive(Component, Debug, Clone, Decode, Encode, TypeHash, Default)]
pub struct BossbarSender(pub Vec<u128>);

impl BossbarSender {
    pub fn add(&mut self, uuid: Uuid) {
        self.0.push(uuid.as_u128());
    }

    pub fn remove(&mut self, uuid: Uuid) {
        self.0.retain(|&u| u != uuid.as_u128());
    }
}
