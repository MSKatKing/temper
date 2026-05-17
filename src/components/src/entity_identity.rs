use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// Identity component for entities in the game world, including players and non-player entities (mobs, items, etc.).
#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Network entity ID used in packets.
    /// Must be unique across all entities in the server.
    /// For players, this is generally the first 4 bytes of the player's UUID, unless multiple
    /// players have the same UUID (eg. offline mode) in which case it will be random.
    pub entity_id: i32,

    /// Unique identifier for this entity.
    /// Generated randomly for each spawned entity.
    /// For players, this is the full UUID from Mojang's authentication system.
    pub uuid: uuid::Uuid,

    /// Optional name for the entity
    /// For players, this is the username. For other entities, it can be None or a custom name.
    pub name: Option<String>,
}

impl Identity {
    /// Creates a new entity identity with a unique ID and UUID.
    ///
    /// The entity_id is generated randomly to avoid collisions with player ids.
    /// The UUID is randomly generated.
    pub fn new(name: Option<String>) -> Self {
        Self {
            entity_id: rand::random(),
            uuid: uuid::Uuid::new_v4(),
            name,
        }
    }

    /// Creates an entity identity with a specific UUID (for loading from disk).
    pub fn with_uuid(uuid: uuid::Uuid, name: Option<String>) -> Self {
        Self {
            entity_id: rand::random(),
            uuid,
            name,
        }
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new(None)
    }
}
