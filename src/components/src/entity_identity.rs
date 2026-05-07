use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI32, Ordering};

/// Global entity ID counter for non-player entities.
///
/// Each spawned entity (pig, cow, arrow, etc.) needs a unique network ID
/// to be identified in packets. This counter generates sequential IDs
/// starting from a high number to avoid collisions with player short_uuid.
static ENTITY_ID_COUNTER: AtomicI32 = AtomicI32::new(1_000_000);

/// Identity component for entities in the game world, including players and non-player entities (mobs, items, etc.).
///
/// # Examples
///
/// ```ignore
/// use temper_core::identity::Identity;
///
/// let pig_identity = Identity::new(Some("Pig".to_string()));
/// assert!(pig_identity.entity_id >= 1_000_000);
/// ```
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
    /// The entity_id is generated from an atomic counter to ensure uniqueness.
    /// The UUID is randomly generated.
    pub fn new(name: Option<String>) -> Self {
        Self {
            entity_id: ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            uuid: uuid::Uuid::new_v4(),
            name,
        }
    }

    /// Creates an entity identity with a specific UUID (for loading from disk).
    pub fn with_uuid(uuid: uuid::Uuid, name: Option<String>) -> Self {
        Self {
            entity_id: ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
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
