use temper_components::entity_identity::Identity;
use temper_components::player::position::Position;

use crate::bundles::{FoxBundle, PigBundle};
use crate::entity_types::EntityTypeEnum;

pub enum MobBundle {
    Pig(PigBundle),
    Fox(FoxBundle),
}

impl Clone for MobBundle {
    fn clone(&self) -> Self {
        match self {
            Self::Pig(bundle) => Self::Pig(PigBundle {
                identity: bundle.identity.clone(),
                metadata: bundle.metadata,
                combat: bundle.combat,
                spawn: bundle.spawn.clone(),
                position: bundle.position,
                rotation: bundle.rotation,
                velocity: bundle.velocity,
                on_ground: bundle.on_ground,
                last_synced_position: bundle.last_synced_position,
            }),
            Self::Fox(bundle) => Self::Fox(FoxBundle {
                identity: bundle.identity.clone(),
                metadata: bundle.metadata,
                combat: bundle.combat,
                spawn: bundle.spawn.clone(),
                position: bundle.position,
                rotation: bundle.rotation,
                velocity: bundle.velocity,
                on_ground: bundle.on_ground,
                last_synced_position: bundle.last_synced_position,
            }),
        }
    }
}

impl MobBundle {
    pub fn kind(&self) -> EntityTypeEnum {
        match self {
            Self::Pig(_) => EntityTypeEnum::Pig,
            Self::Fox(_) => EntityTypeEnum::Fox,
        }
    }

    pub fn identity(&self) -> &Identity {
        match self {
            Self::Pig(bundle) => &bundle.identity,
            Self::Fox(bundle) => &bundle.identity,
        }
    }

    pub fn position(&self) -> Position {
        match self {
            Self::Pig(bundle) => bundle.position,
            Self::Fox(bundle) => bundle.position,
        }
    }
}
