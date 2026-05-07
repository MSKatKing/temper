//! How this binfire works:
//! - `define_entity_save_load` generates a pair of save/load systems for a specific entity bundle.
//!   The generated save system listens for `SaveChunkEntities` and writes matching entities into
//!   the chunk entity map in world state.
//! - The generated load system listens for `LoadChunkEntities`, deserializes matching saved
//!   entities from the chunk, and respawns them with the required runtime-only marker/components.
//! - `define_standard_mob_save_load` is a wrapper that supplies the standard persisted
//!   fields used by most mobs. You don't have to use this if you have a weird mob with different
//!   persisted fields, but it should cover most cases and saves a lot of boilerplate.
//!
//! The macro invocations themselves live in the category modules:
//! - `ground` for mobs with gravity, collisions, and water drag
//! - `collision_only` for mobs that only need collisions
//! - `gravity_no_drag` for mobs with gravity/collisions but no water drag
//!
//! Those module-level macro invocations generate the concrete `save_*` and `load_*` systems.
//! The `register_load_systems` and `register_save_systems` functions in this file do not generate
//! systems; they just ask each category module to add its already-generated systems to the
//! appropriate Bevy system set.

use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule, SystemSet};

pub mod collision_only;
pub mod gravity_no_drag;
pub mod ground;
pub mod pig;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MobLoadSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MobSaveSystems;

pub fn register_tick_systems(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            pig::init_pig,
            bevy_ecs::prelude::ApplyDeferred,
            pathfinding::tick_pathfinder,
            pig::tick_pig,
            pig::tick_pig_particles,
        )
            .chain(),
    );
}

pub fn register_load_systems(schedule: &mut Schedule) {
    ground::register_load_systems(schedule);
    collision_only::register_load_systems(schedule);
    gravity_no_drag::register_load_systems(schedule);
}

pub fn register_save_systems(schedule: &mut Schedule) {
    ground::register_save_systems(schedule);
    collision_only::register_save_systems(schedule);
    gravity_no_drag::register_save_systems(schedule);
}

#[macro_export]
macro_rules! add_systems_to_set {
    ($schedule:expr, $set:path, [ $( $system:path ),+ $(,)? ]) => {
        $(
            $schedule.add_systems($system.in_set($set));
        )+
    };
}

#[macro_export]
macro_rules! define_standard_mob_save_load {
    (
        $name:ident,
        marker = $marker:path,
        bundle = $bundle:path,
        entity_type = $entity_type:ident,
        runtime_components = ( $( $runtime_component:path ),* $(,)? )
    ) => {
        $crate::define_entity_save_load!(
            $name,
            marker = $marker,
            bundle = $bundle,
            entity_type = $entity_type,
            runtime_components = ( $( $runtime_component ),* ),
            fields = {
                identity: temper_components::entity_identity::Identity => clone,
                metadata: temper_components::metadata::EntityMetadata => copy,
                combat: temper_components::combat::CombatProperties => copy,
                spawn: temper_components::spawn::SpawnProperties => clone,
                position: temper_components::player::position::Position => copy,
                rotation: temper_components::player::rotation::Rotation => copy,
                velocity: temper_components::player::velocity::Velocity => copy,
                on_ground: temper_components::player::grounded::OnGround => copy,
                last_synced_position: temper_components::last_synced_position::LastSyncedPosition => copy,
            }
        );
    };
}

/// Generates chunk save/load systems for an entity bundle.
///
/// The macro expects the persisted field list to include:
/// - `identity`, used as the saved entity key
/// - `position`, used to determine the entity's chunk
///
/// Field modes:
/// - `clone` expands to `field.clone()`
/// - `copy` expands to `*field`
///
/// Example:
/// ```ignore
/// define_entity_save_load!(
///     pig,
///     marker = Pig,
///     bundle = PigBundle,
///     entity_type = Pig,
///     runtime_components = (HasGravity, HasCollisions, HasWaterDrag),
///     fields = {
///         identity: Identity => clone,
///         metadata: EntityMetadata => copy,
///         combat: CombatProperties => copy,
///         spawn: SpawnProperties => clone,
///         position: Position => copy,
///         rotation: Rotation => copy,
///         velocity: Velocity => copy,
///         on_ground: OnGround => copy,
///         last_synced_position: LastSyncedPosition => copy,
///     }
/// );
/// ```
#[macro_export]
macro_rules! define_entity_save_load {
    (@field_value $field:ident, clone) => {
        $field.clone()
    };
    (@field_value $field:ident, copy) => {
        *$field
    };
    (
        $name:ident,
        marker = $marker:path,
        bundle = $bundle:path,
        entity_type = $entity_type:ident,
        runtime_components = ( $( $runtime_component:path ),* $(,)? ),
        fields = {
            $(
                $field:ident : $field_ty:path => $mode:ident
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            pub fn [<save_ $name>](
                state: bevy_ecs::prelude::Res<temper_state::GlobalStateResource>,
                query: bevy_ecs::prelude::Query<
                    (
                        $( &$field_ty, )*
                    ),
                    bevy_ecs::prelude::With<$marker>,
                >,
                mut reader: bevy_ecs::prelude::MessageReader<
                    temper_messages::save_chunk_entities::SaveChunkEntities,
                >,
            ) {
                for message in reader.read() {
                    for ($($field,)*) in query.iter() {
                        let bundle = $bundle {
                            $(
                                $field: $crate::define_entity_save_load!(@field_value $field, $mode),
                            )*
                        };

                        if bundle.position.chunk() != message.0 {
                            continue;
                        }

                        tracing::debug!(
                            "Saving {} with UUID {} at chunk {}",
                            stringify!($name),
                            bundle.identity.uuid,
                            message.0
                        );

                        let chunk = state
                            .0
                            .world
                            .get_or_generate_chunk(
                                message.0,
                                temper_core::dimension::Dimension::Overworld,
                            )
                            .expect("Failed to get or generate chunk");

                        chunk.entities.insert(
                            bundle.identity.uuid,
                            (
                                temper_entities::entity_types::EntityTypeEnum::$entity_type,
                                bitcode::serialize(&bundle)
                                    .expect("Failed to serialize entity bundle"),
                            ),
                        );
                        chunk.mark_dirty();
                    }
                }
            }

            #[expect(unused_variables)]
            pub fn [<load_ $name>](
                state: bevy_ecs::prelude::Res<temper_state::GlobalStateResource>,
                mut cmd: bevy_ecs::prelude::Commands,
                mut reader: bevy_ecs::prelude::MessageReader<
                    temper_messages::load_chunk_entities::LoadChunkEntities,
                >,
                players: bevy_ecs::prelude::Query<
                    (
                        &temper_net_runtime::connection::StreamWriter,
                        &temper_components::player::position::Position,
                        &temper_components::player::client_information::ClientInformationComponent,
                    ),
                    bevy_ecs::prelude::With<
                        temper_components::player::player_marker::PlayerMarker,
                    >,
                >,
            ) {
                for message in reader.read() {
                    let Ok(chunk) = state
                        .0
                        .world
                        .get_chunk(
                            message.0,
                            temper_core::dimension::Dimension::Overworld,
                        )
                        else {
                            tracing::error!("Failed to load chunk {} for entity loading", message.0);
                            continue;
                        };

                    for kv in chunk.entities.iter() {
                        let (entity_type, data) = kv.value();
                        if *entity_type
                            == temper_entities::entity_types::EntityTypeEnum::$entity_type
                        {
                            tracing::debug!(
                            "Loading entity of type {:?} from chunk {}",
                            entity_type,
                            message.0
                        );
                            let bundle: $bundle = bitcode::deserialize(data)
                                .expect("Failed to deserialize entity bundle");
                            let last_chunk = temper_components::last_chunk_pos::LastChunkPos::new(
                                bundle.position.chunk(),
                            );
                            cmd.spawn((
                                bundle,
                                $marker,
                                $( $runtime_component, )*
                                last_chunk,
                            ));
                        }
                    }
                }
            }
        }
    };
}
