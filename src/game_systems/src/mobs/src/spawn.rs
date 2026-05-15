use bevy_ecs::prelude::*;
use temper_components::combat::CombatProperties;
use temper_components::entity_identity::Identity;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::last_synced_position::LastSyncedPosition;
use temper_components::metadata::EntityMetadata;
use temper_components::player::entity_tracker::EntityTracker;
use temper_components::player::grounded::OnGround;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_components::player::velocity::Velocity;
use temper_components::spawn::SpawnProperties;
use temper_core::dimension::Dimension;
use temper_entities::entity_types::EntityTypeEnum;
use temper_entities::markers::entity_types::{Axolotl, Bat, Cow, Fox, Pig};
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::mob_definition::{MobProfile, StandardMobParts};
use temper_entities::{MobBundle, MobKind, PigBundle};
use temper_messages::{
    SpawnMobBundle, load_chunk_entities::LoadChunkEntities, save_chunk_entities::SaveChunkEntities,
};
use temper_state::GlobalStateResource;

type StandardMobQuery<'a> = (
    &'a Identity,
    &'a EntityMetadata,
    &'a CombatProperties,
    &'a SpawnProperties,
    &'a Position,
    &'a Rotation,
    &'a Velocity,
    &'a OnGround,
    &'a LastSyncedPosition,
    &'a MobKind,
);

trait SpawnMobBundleExt {
    fn spawn(self, commands: &mut Commands) -> Entity;
}

impl SpawnMobBundleExt for MobBundle {
    fn spawn(self, commands: &mut Commands) -> Entity {
        match self {
            Self::Pig(bundle) => spawn_pig(commands, bundle),
            Self::Fox(bundle) => {
                let position = bundle.position;
                let profile = bundle.profile();
                spawn_profiled(
                    commands,
                    bundle,
                    Fox,
                    EntityTypeEnum::Fox,
                    profile,
                    position,
                )
            }
            Self::Cow(bundle) => {
                let position = bundle.position;
                let profile = bundle.profile();
                spawn_profiled(
                    commands,
                    bundle,
                    Cow,
                    EntityTypeEnum::Cow,
                    profile,
                    position,
                )
            }
            Self::Bat(bundle) => {
                let position = bundle.position;
                let profile = bundle.profile();
                spawn_profiled(
                    commands,
                    bundle,
                    Bat,
                    EntityTypeEnum::Bat,
                    profile,
                    position,
                )
            }
            Self::Axolotl(bundle) => {
                let position = bundle.position;
                let profile = bundle.profile();
                spawn_profiled(
                    commands,
                    bundle,
                    Axolotl,
                    EntityTypeEnum::Axolotl,
                    profile,
                    position,
                )
            }
        }
    }
}

fn spawn_pig(commands: &mut Commands, bundle: PigBundle) -> Entity {
    let position = bundle.position;
    let profile = bundle.profile();
    let entity = spawn_profiled(
        commands,
        bundle,
        Pig,
        EntityTypeEnum::Pig,
        profile,
        position,
    );
    commands.entity(entity).insert((
        crate::pig::PigAI::default(),
        pathfinding::Pathfinder::default(),
        pathfinding::PathfinderSearch::default(),
    ));
    entity
}

fn spawn_profiled<B, M>(
    commands: &mut Commands,
    bundle: B,
    marker: M,
    kind: EntityTypeEnum,
    profile: MobProfile,
    position: Position,
) -> Entity
where
    B: Bundle,
    M: Component,
{
    let last_chunk = LastChunkPos::new(position.chunk());
    let entity = commands
        .spawn((bundle, marker, MobKind(kind), last_chunk))
        .id();

    match profile {
        MobProfile::Ground => {
            commands
                .entity(entity)
                .insert((HasGravity, HasCollisions, HasWaterDrag));
        }
        MobProfile::CollisionOnly => {
            commands.entity(entity).insert(HasCollisions);
        }
        MobProfile::GravityNoDrag => {
            commands.entity(entity).insert((HasGravity, HasCollisions));
        }
    }

    entity
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

pub fn save_mob_bundles(
    state: Res<GlobalStateResource>,
    query: Query<StandardMobQuery>,
    mut save_events: MessageReader<SaveChunkEntities>,
) {
    for event in save_events.read() {
        for (
            identity,
            metadata,
            combat,
            spawn,
            position,
            rotation,
            velocity,
            on_ground,
            last_synced_position,
            mob_kind,
        ) in query.iter()
        {
            if position.chunk() != event.0 {
                continue;
            }

            let Some(bundle) = standard_mob_bundle(
                identity,
                metadata,
                combat,
                spawn,
                position,
                rotation,
                velocity,
                on_ground,
                last_synced_position,
                mob_kind,
            ) else {
                continue;
            };

            let kind = bundle.kind();
            let uuid = bundle.identity().uuid;
            let chunk = state
                .0
                .world
                .get_or_generate_chunk(event.0, Dimension::Overworld)
                .expect("Failed to get or generate chunk");
            chunk
                .entities
                .insert(uuid, (kind, bundle.serialize_for_chunk()));
            chunk.mark_dirty();
        }
    }
}

fn standard_mob_bundle(
    identity: &Identity,
    metadata: &EntityMetadata,
    combat: &CombatProperties,
    spawn: &SpawnProperties,
    position: &Position,
    rotation: &Rotation,
    velocity: &Velocity,
    on_ground: &OnGround,
    last_synced_position: &LastSyncedPosition,
    mob_kind: &MobKind,
) -> Option<MobBundle> {
    MobBundle::from_standard_parts(
        mob_kind.0,
        StandardMobParts {
            identity,
            metadata,
            combat,
            spawn,
            position,
            rotation,
            velocity,
            on_ground,
            last_synced_position,
        },
    )
}
