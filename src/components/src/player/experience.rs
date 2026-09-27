use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use type_hash::TypeHash;

#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize, TypeHash)]
pub struct Experience {
    /// 0.0-1.0 progress to next level
    pub progress: f32,
    /// The player's level
    pub level: u32,
    /// The total XP the player has ever collected
    pub total_xp: u32,
}
