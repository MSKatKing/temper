use bevy_ecs::prelude::*;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::player::entity_tracker::EntityTracker;
use temper_core::dimension::Dimension;
use temper_entities::markers::entity_types::{Fox, Pig};
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::{MobBundle, PigBundle};
use temper_messages::{SpawnMobBundle, load_chunk_entities::LoadChunkEntities};
use temper_state::GlobalStateResource;

trait SpawnMobBundleExt {
    fn serialize_for_chunk(&self) -> Vec<u8>;
    fn spawn(self, commands: &mut Commands) -> Entity;
}

impl SpawnMobBundleExt for MobBundle {
    fn serialize_for_chunk(&self) -> Vec<u8> {
        match self {
            Self::Pig(bundle) => {
                bitcode::serialize(bundle).expect("Failed to serialize pig bundle")
            }
            Self::Fox(bundle) => {
                bitcode::serialize(bundle).expect("Failed to serialize fox bundle")
            }
        }
    }

    fn spawn(self, commands: &mut Commands) -> Entity {
        match self {
            Self::Pig(bundle) => spawn_pig(commands, bundle),
            Self::Fox(bundle) => {
                let last_chunk = LastChunkPos::new(bundle.position.chunk());
                commands
                    .spawn((
                        bundle,
                        Fox,
                        HasGravity,
                        HasCollisions,
                        HasWaterDrag,
                        last_chunk,
                    ))
                    .id()
            }
        }
    }
}

fn spawn_pig(commands: &mut Commands, bundle: PigBundle) -> Entity {
    let last_chunk = LastChunkPos::new(bundle.position.chunk());
    commands
        .spawn((
            bundle,
            Pig,
            HasGravity,
            HasCollisions,
            HasWaterDrag,
            crate::pig::PigAI::default(),
            pathfinding::Pathfinder::default(),
            pathfinding::PathfinderSearch::default(),
            last_chunk,
        ))
        .id()
}

pub fn handle_spawn_mob_bundle(
    mut events: MessageReader<SpawnMobBundle>,
    mut commands: Commands,
    state: Res<GlobalStateResource>,
    query: Query<&EntityTracker>,
) {
    for event in events.read() {
        let kind = event.bundle.kind();
        let uuid = event.bundle.identity().uuid;
        let position = event.bundle.position();

        if event.persist {
            let chunk = state
                .0
                .world
                .get_or_generate_chunk(position.chunk(), Dimension::Overworld)
                .expect("Failed to get or generate chunk");
            chunk
                .entities
                .insert(uuid, (kind, event.bundle.serialize_for_chunk()));
            chunk.mark_dirty();
        }

        event.bundle.clone().spawn(&mut commands);

        query.iter().for_each(|tracker| {
            tracker.to_track.push((uuid, kind.to_entity_type().id));
        });
    }
}

pub fn load_mob_bundles(
    state: Res<GlobalStateResource>,
    mut load_events: MessageReader<LoadChunkEntities>,
    mut spawn_events: MessageWriter<SpawnMobBundle>,
) {
    for event in load_events.read() {
        let Ok(chunk) = state.0.world.get_chunk(event.0, Dimension::Overworld) else {
            tracing::error!("Failed to load chunk {} for entity loading", event.0);
            continue;
        };

        for kv in chunk.entities.iter() {
            let (kind, data) = kv.value();
            let Some(bundle) = MobBundle::deserialize(*kind, data) else {
                continue;
            };

            spawn_events.write(SpawnMobBundle {
                bundle,
                persist: false,
            });
        }
    }
}
