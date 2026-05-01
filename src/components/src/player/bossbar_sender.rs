use bevy_ecs::component::Component;
use bitcode_derive::{Decode, Encode};
use type_hash::TypeHash;

#[derive(Component, Debug, Clone, Decode, Encode, TypeHash)]
pub struct BossbarSender(pub Vec<u128>);

impl Default for BossbarSender {
    fn default() -> Self {
        BossbarSender(vec![])
    }
}