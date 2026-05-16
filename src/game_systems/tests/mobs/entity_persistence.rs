use bevy_ecs::prelude::*;
use mobs::spawn::{
    handle_spawn_mob_bundle, load_mob_bundles, queue_live_mob_chunk_saves, save_mob_bundles,
};
use shutdown::send_save_message::send_save_message;
use std::time::Instant;
use temper_components::entity_identity::Identity;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::last_synced_position::LastSyncedPosition;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_entities::entity_types::EntityTypeEnum;
use temper_entities::markers::entity_types::{Cow, Fox, Pig};
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::{CowBundle, FoxBundle, MobBundle, MobKind, PigBundle};
use temper_messages::load_chunk_entities::LoadChunkEntities;
use temper_messages::save_chunk_entities::SaveChunkEntities;
use temper_resources::world_sync_tracker::WorldSyncTracker;
use temper_scheduler::Scheduler;
use temper_state::create_test_state;

fn emit_save_for(
    chunk: temper_core::pos::ChunkPos,
) -> impl FnMut(MessageWriter<SaveChunkEntities>) {
    move |mut writer: MessageWriter<SaveChunkEntities>| {
        writer.write(SaveChunkEntities(chunk));
    }
}

fn emit_load_for(
    chunk: temper_core::pos::ChunkPos,
) -> impl FnMut(MessageWriter<LoadChunkEntities>) {
    move |mut writer: MessageWriter<LoadChunkEntities>| {
        writer.write(LoadChunkEntities(chunk));
    }
}

#[test]
fn pig_round_trips_through_chunk_save_and_load() {
    let mut world = World::new();
    temper_messages::register_messages(&mut world);

    let (state, _temp_dir) = create_test_state();
    world.insert_resource(state);

    let position = Position::new(5.5, 64.0, 7.5);
    let chunk = position.chunk();
    let bundle = PigBundle::new(position);
    let expected_identity = bundle.identity.clone();
    let expected_last_synced = bundle.last_synced_position;

    let original_entity = world
        .spawn((
            bundle,
            Pig,
            MobKind(EntityTypeEnum::Pig),
            HasGravity,
            HasCollisions,
            HasWaterDrag,
        ))
        .id();

    let mut save_schedule = Schedule::default();
    save_schedule.add_systems((emit_save_for(chunk), save_mob_bundles).chain());
    save_schedule.run(&mut world);

    {
        let state = world.resource::<temper_state::GlobalStateResource>();
        let saved_chunk = state
            .0
            .world
            .get_chunk(chunk, Dimension::Overworld)
            .expect("chunk should exist after save");
        let saved_entity = saved_chunk
            .entities
            .get(&expected_identity.uuid)
            .expect("saved pig should be present in chunk storage");

        assert_eq!(saved_entity.value().0, EntityTypeEnum::Pig);
    }

    world.despawn(original_entity);

    let mut load_schedule = Schedule::default();
    load_schedule.add_systems(
        (
            emit_load_for(chunk),
            load_mob_bundles,
            handle_spawn_mob_bundle,
        )
            .chain(),
    );
    load_schedule.run(&mut world);

    let mut query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        &LastSyncedPosition,
        Has<Pig>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();

    let loaded: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        loaded.len(),
        1,
        "exactly one pig should be loaded back into ECS"
    );

    let (
        identity,
        loaded_position,
        last_chunk,
        last_synced,
        is_pig,
        has_gravity,
        has_collisions,
        has_water_drag,
    ) = &loaded[0];

    assert!(is_pig, "loaded entity should have the Pig marker");
    assert!(has_gravity, "loaded pig should regain HasGravity");
    assert!(has_collisions, "loaded pig should regain HasCollisions");
    assert!(has_water_drag, "loaded pig should regain HasWaterDrag");
    assert_eq!(identity.uuid, expected_identity.uuid);
    assert_eq!(identity.entity_id, expected_identity.entity_id);
    assert_eq!(loaded_position.coords, position.coords);
    assert_eq!(last_chunk.0, chunk);
    assert_eq!(last_synced.0, expected_last_synced.0);
}

#[test]
fn shutdown_save_message_saves_spawned_pig_from_ecs() {
    let mut world = World::new();
    temper_messages::register_messages(&mut world);

    let (state, _temp_dir) = create_test_state();
    world.insert_resource(state);

    let position = Position::new(6.5, 64.0, 7.5);
    let chunk = position.chunk();
    let bundle = PigBundle::new(position);
    let expected_identity = bundle.identity.clone();

    {
        let state = world.resource::<temper_state::GlobalStateResource>();
        state
            .0
            .world
            .get_or_generate_chunk(chunk, Dimension::Overworld)
            .expect("chunk should be cached before shutdown save");
    }

    world.spawn((
        bundle,
        Pig,
        MobKind(EntityTypeEnum::Pig),
        HasGravity,
        HasCollisions,
        HasWaterDrag,
    ));

    let mut save_schedule = Schedule::default();
    save_schedule.add_systems((send_save_message, save_mob_bundles).chain());
    save_schedule.run(&mut world);

    let state = world.resource::<temper_state::GlobalStateResource>();
    let saved_chunk = state
        .0
        .world
        .get_chunk(chunk, Dimension::Overworld)
        .expect("chunk should exist after shutdown save");
    let saved_entity = saved_chunk
        .entities
        .get(&expected_identity.uuid)
        .expect("spawned pig should be saved by shutdown save messages");

    assert_eq!(saved_entity.value().0, EntityTypeEnum::Pig);
}

#[test]
fn live_mob_chunk_save_saves_pig_when_chunk_is_not_cached() {
    let mut world = World::new();
    temper_messages::register_messages(&mut world);

    let (state, _temp_dir) = create_test_state();
    world.insert_resource(state);

    let position = Position::new(40.5, 64.0, 40.5);
    let chunk = position.chunk();
    let bundle = PigBundle::new(position);
    let expected_identity = bundle.identity.clone();

    world.spawn((
        bundle,
        Pig,
        MobKind(EntityTypeEnum::Pig),
        HasGravity,
        HasCollisions,
        HasWaterDrag,
        LastChunkPos::new(chunk),
    ));

    let mut save_schedule = Schedule::default();
    save_schedule.add_systems((queue_live_mob_chunk_saves, save_mob_bundles).chain());
    save_schedule.run(&mut world);

    let state = world.resource::<temper_state::GlobalStateResource>();
    let saved_chunk = state
        .0
        .world
        .get_chunk(chunk, Dimension::Overworld)
        .expect("live mob chunk save should generate the pig chunk");
    let saved_entity = saved_chunk
        .entities
        .get(&expected_identity.uuid)
        .expect("pig should be saved even when its chunk was not already cached");

    assert_eq!(saved_entity.value().0, EntityTypeEnum::Pig);
}

#[test]
fn live_mob_chunk_save_flushes_uncached_pig_to_storage() {
    let (state, _temp_dir) = create_test_state();

    let position = Position::new(40.5, 64.0, 40.5);
    let chunk = position.chunk();
    let bundle = PigBundle::new(position);
    let expected_identity = bundle.identity.clone();

    {
        let mut first_world = World::new();
        temper_messages::register_messages(&mut first_world);
        first_world.insert_resource(state.clone());
        first_world.insert_resource(WorldSyncTracker {
            last_synced: Instant::now(),
        });
        first_world.spawn((
            bundle,
            Pig,
            MobKind(EntityTypeEnum::Pig),
            HasGravity,
            HasCollisions,
            HasWaterDrag,
            LastChunkPos::new(chunk),
        ));

        let mut save_schedule = Schedule::default();
        save_schedule.add_systems((queue_live_mob_chunk_saves, save_mob_bundles).chain());
        save_schedule.run(&mut first_world);
    }

    state
        .0
        .world
        .sync()
        .expect("saved pig should be flushed to storage before restart-style load");
    state.0.world.get_cache().clear();

    let loaded = {
        let mut second_world = World::new();
        temper_messages::register_messages(&mut second_world);
        second_world.insert_resource(state.clone());
        let mut load_schedule = Schedule::default();
        load_schedule.add_systems(
            (
                emit_load_for(chunk),
                load_mob_bundles,
                handle_spawn_mob_bundle,
            )
                .chain(),
        );
        load_schedule.run(&mut second_world);

        let mut query = second_world.query::<(&Identity, &Position, Has<Pig>)>();
        query
            .iter(&second_world)
            .filter(|(_, _, is_pig)| *is_pig)
            .map(|(identity, position, _)| (identity.clone(), *position))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        loaded.len(),
        1,
        "exactly one pig should reload after flushing storage"
    );

    let (identity, loaded_position) = &loaded[0];
    assert_eq!(identity.uuid, expected_identity.uuid);
    assert_eq!(loaded_position.coords, position.coords);
}

#[test]
fn registered_shutdown_schedule_saves_pig_into_replacement_ecs_world() {
    let (state, _temp_dir) = create_test_state();

    let position = Position::new(45.5, 64.0, 45.5);
    let chunk = position.chunk();
    let bundle = PigBundle::new(position);
    let expected_identity = bundle.identity.clone();

    {
        let mut first_world = World::new();
        temper_messages::register_messages(&mut first_world);
        first_world.insert_resource(state.clone());
        first_world.insert_resource(WorldSyncTracker {
            last_synced: Instant::now(),
        });
        first_world.spawn((
            bundle,
            Pig,
            MobKind(EntityTypeEnum::Pig),
            HasGravity,
            HasCollisions,
            HasWaterDrag,
            LastChunkPos::new(chunk),
        ));

        let mut timed = Scheduler::new();
        let mut shutdown_schedule = Schedule::default();
        temper_game_systems::register_schedules(&mut timed, &mut shutdown_schedule);
        shutdown_schedule.run(&mut first_world);
    }

    state.0.world.get_cache().clear();

    let loaded = {
        let mut second_world = World::new();
        temper_messages::register_messages(&mut second_world);
        second_world.insert_resource(state.clone());
        let mut load_schedule = Schedule::default();
        load_schedule.add_systems(
            (
                emit_load_for(chunk),
                load_mob_bundles,
                handle_spawn_mob_bundle,
            )
                .chain(),
        );
        load_schedule.run(&mut second_world);

        let mut query = second_world.query::<(&Identity, &Position, Has<Pig>)>();
        query
            .iter(&second_world)
            .filter(|(_, _, is_pig)| *is_pig)
            .map(|(identity, position, _)| (identity.clone(), *position))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        loaded.len(),
        1,
        "registered shutdown schedule should persist one pig for replacement ECS load"
    );

    let (identity, loaded_position) = &loaded[0];
    assert_eq!(identity.uuid, expected_identity.uuid);
    assert_eq!(loaded_position.coords, position.coords);
}

#[test]
fn mob_save_refreshes_existing_chunk_entry_with_live_position() {
    let mut world = World::new();
    temper_messages::register_messages(&mut world);

    let (state, _temp_dir) = create_test_state();
    world.insert_resource(state.clone());

    let spawn_position = Position::new(6.5, 64.0, 7.5);
    let moved_position = Position::new(8.5, 64.0, 7.5);
    let chunk = spawn_position.chunk();
    let cow_bundle = CowBundle::new(spawn_position);
    let expected_identity = cow_bundle.identity.clone();

    {
        let chunk = state
            .0
            .world
            .get_or_generate_chunk(chunk, Dimension::Overworld)
            .expect("chunk should be cached before save");
        chunk.entities.insert(
            expected_identity.uuid,
            (
                EntityTypeEnum::Cow,
                MobBundle::Cow(CowBundle::new(spawn_position)).serialize_for_chunk(),
            ),
        );
        chunk.mark_dirty();
    }

    let cow_entity = world
        .spawn((
            cow_bundle,
            Cow,
            MobKind(EntityTypeEnum::Cow),
            HasGravity,
            HasCollisions,
            HasWaterDrag,
            LastChunkPos::new(chunk),
        ))
        .id();

    {
        let mut position = world
            .get_mut::<Position>(cow_entity)
            .expect("cow should still be alive");
        *position = moved_position;
    }

    let mut save_schedule = Schedule::default();
    save_schedule.add_systems((emit_save_for(chunk), save_mob_bundles).chain());
    save_schedule.run(&mut world);

    let state = world.resource::<temper_state::GlobalStateResource>();
    let saved_chunk = state
        .0
        .world
        .get_chunk(chunk, Dimension::Overworld)
        .expect("chunk should exist after save");
    let saved_entity = saved_chunk
        .entities
        .get(&expected_identity.uuid)
        .expect("moved cow should still be stored");
    let saved_bundle = MobBundle::deserialize(saved_entity.value().0, &saved_entity.value().1)
        .expect("saved cow bundle should deserialize");

    assert_eq!(saved_bundle.position().coords, moved_position.coords);
}

#[test]
fn fox_loads_in_a_separate_ecs_world_after_save() {
    let (state, _temp_dir) = create_test_state();

    let position = Position::new(23.5, 70.0, -10.25);
    let chunk = position.chunk();
    let bundle = FoxBundle::new(position);
    let expected_identity = bundle.identity.clone();
    let expected_last_synced = bundle.last_synced_position;

    {
        let mut first_world = World::new();
        temper_messages::register_messages(&mut first_world);
        first_world.insert_resource(state.clone());
        first_world.spawn((
            bundle,
            Fox,
            MobKind(EntityTypeEnum::Fox),
            HasGravity,
            HasCollisions,
            HasWaterDrag,
        ));

        let mut save_schedule = Schedule::default();
        save_schedule.add_systems((emit_save_for(chunk), save_mob_bundles).chain());
        save_schedule.run(&mut first_world);
    }

    state
        .0
        .world
        .sync()
        .expect("saved fox should be flushed to storage before restart-style load");
    state.0.world.get_cache().clear();

    let loaded = {
        let mut second_world = World::new();
        temper_messages::register_messages(&mut second_world);
        second_world.insert_resource(state.clone());

        let mut load_schedule = Schedule::default();
        load_schedule.add_systems(
            (
                emit_load_for(chunk),
                load_mob_bundles,
                handle_spawn_mob_bundle,
            )
                .chain(),
        );
        load_schedule.run(&mut second_world);

        let mut query = second_world.query::<(
            &Identity,
            &Position,
            &LastChunkPos,
            &LastSyncedPosition,
            Has<Fox>,
            Has<HasGravity>,
            Has<HasCollisions>,
            Has<HasWaterDrag>,
        )>();

        query
            .iter(&second_world)
            .map(
                |(
                    identity,
                    loaded_position,
                    last_chunk,
                    last_synced,
                    is_fox,
                    has_gravity,
                    has_collisions,
                    has_water_drag,
                )| {
                    (
                        identity.clone(),
                        *loaded_position,
                        *last_chunk,
                        *last_synced,
                        is_fox,
                        has_gravity,
                        has_collisions,
                        has_water_drag,
                    )
                },
            )
            .collect::<Vec<_>>()
    };

    assert_eq!(
        loaded.len(),
        1,
        "exactly one fox should be loaded into the replacement ECS world"
    );

    let (
        identity,
        loaded_position,
        last_chunk,
        last_synced,
        is_fox,
        has_gravity,
        has_collisions,
        has_water_drag,
    ) = &loaded[0];

    assert!(is_fox, "loaded entity should have the Fox marker");
    assert!(has_gravity, "loaded fox should regain HasGravity");
    assert!(has_collisions, "loaded fox should regain HasCollisions");
    assert!(has_water_drag, "loaded fox should regain HasWaterDrag");
    assert_eq!(identity.uuid, expected_identity.uuid);
    assert_eq!(identity.entity_id, expected_identity.entity_id);
    assert_eq!(loaded_position.coords, position.coords);
    assert_eq!(last_chunk.0, chunk);
    assert_eq!(last_synced.0, expected_last_synced.0);
}
