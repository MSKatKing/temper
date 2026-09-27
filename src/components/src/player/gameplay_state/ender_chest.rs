use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use temper_inventories::inventory::Inventory;
use type_hash::TypeHash;

/// The player's 27-slot personal Ender Chest.
#[derive(Component, Clone, Debug, Serialize, Deserialize, TypeHash)]
pub struct EnderChest(pub Inventory);

impl EnderChest {
    pub const ENDERCHEST_SIZE: usize = 27;
}

impl Default for EnderChest {
    fn default() -> Self {
        Self(Inventory::new(EnderChest::ENDERCHEST_SIZE))
    }
}
