use std::collections::HashMap;
use bevy_ecs::component::Component;
use bitcode_derive::{Decode, Encode};
use type_hash::TypeHash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Decode, Encode, TypeHash, Default, Eq, PartialEq)]
pub enum BossbarSenderState {
    Additive,
    Subtractive,
    Update,

    #[default]
    Informed,
}

#[derive(Component, Debug, Clone, Decode, Encode, TypeHash, Default)]
pub struct BossbarSender(pub HashMap<u128, BossbarSenderState>);

impl BossbarSender {
    pub fn add(&mut self, uuid: Uuid) {
        self.0.insert(uuid.as_u128(), BossbarSenderState::Additive);
    }

    pub fn update(&mut self, uuid: Uuid) {
        self.0.insert(uuid.as_u128(), BossbarSenderState::Update);
    }

    pub fn remove(&mut self, uuid: Uuid) {
        self.0.insert(uuid.as_u128(), BossbarSenderState::Subtractive);
    }

    pub fn informed(&mut self, uuid: Uuid) {
        let id = uuid.as_u128();

        match self.0.get(&id) {
            Some(BossbarSenderState::Subtractive) => {
                self.0.remove(&id);
            }
            Some(_) => {
                self.0.insert(id, BossbarSenderState::Informed);
            }
            None => {}
        }
    }

    pub fn get_state(&self, uuid: Uuid) -> Option<BossbarSenderState> {
        self.0.get(&uuid.as_u128()).cloned()
    }
}
