use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use type_hash::TypeHash;

// --- Placeholders ---
// TODO: fill this out in some way
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TypeHash)]
pub enum EffectType {
    Speed,
    Poison,
    Regeneration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypeHash)]
pub struct EffectState {
    pub amplifier: u8,
    /// Duration in server ticks
    pub duration_ticks: u32,
}

/// Tracks all active potion effects on the player.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, TypeHash)]
pub struct ActiveEffects {
    pub effects: HashMap<EffectType, EffectState>,
}
