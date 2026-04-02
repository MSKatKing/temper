use bevy_ecs::prelude::*;
use temper_components::entity_identity::Identity;

#[derive(Message, Clone)]
#[allow(unused)]
pub struct PlayerLeft(pub Identity);
