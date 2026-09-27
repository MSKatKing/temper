use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use type_hash::TypeHash;

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, TypeHash)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
        }
    }
}
