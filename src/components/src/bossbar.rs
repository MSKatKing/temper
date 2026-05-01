use bevy_ecs::component::Component;
use bitcode_derive::{Decode, Encode};
use type_hash::TypeHash;

#[derive(Component, Debug, Clone, Copy, Decode, Encode, TypeHash)]
pub struct BossbarOwner(u128);
