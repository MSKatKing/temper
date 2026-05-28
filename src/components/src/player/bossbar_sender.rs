use bevy_ecs::component::Component;
use bitcode_derive::{Decode, Encode};
use std::collections::HashMap;
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

#[derive(Component, Debug, Clone, Default)]
pub struct BossbarSender(pub HashMap<Uuid, BossbarSenderState>);

impl BossbarSender {
    pub fn add(&mut self, uuid: Uuid) {
        self.0.insert(uuid, BossbarSenderState::Additive);
    }

    pub fn update(&mut self, uuid: Uuid) {
        self.0.insert(uuid, BossbarSenderState::Update);
    }

    pub fn remove(&mut self, uuid: Uuid) {
        self.0.insert(uuid, BossbarSenderState::Subtractive);
    }

    pub fn informed(&mut self, uuid: Uuid) {
        match self.0.get(&uuid) {
            Some(BossbarSenderState::Subtractive) => {
                self.0.remove(&uuid);
            }
            Some(_) => {
                self.0.insert(uuid, BossbarSenderState::Informed);
            }
            None => {}
        }
    }

    pub fn get_state(&self, uuid: Uuid) -> Option<BossbarSenderState> {
        self.0.get(&uuid).cloned()
    }
}
