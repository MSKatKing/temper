use bevy_ecs::prelude::*;
use mobs::ground::{load_fox, load_pig, save_fox, save_pig};
use temper_components::entity_identity::Identity;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::last_synced_position::LastSyncedPosition;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_entities::entity_types::EntityTypeEnum;
use temper_entities::markers::entity_types::{Fox, Pig};
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::{FoxBundle, PigBundle};
use temper_messages::load_chunk_entities::LoadChunkEntities;
use temper_messages::save_chunk_entities::SaveChunkEntities;
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
        .spawn((bundle, Pig, HasGravity, HasCollisions, HasWaterDrag))
        .id();

    let mut save_schedule = Schedule::default();
    save_schedule.add_systems((emit_save_for(chunk), save_pig).chain());
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
    load_schedule.add_systems((emit_load_for(chunk), load_pig).chain());
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
        first_world.spawn((bundle, Fox, HasGravity, HasCollisions, HasWaterDrag));

        let mut save_schedule = Schedule::default();
        save_schedule.add_systems((emit_save_for(chunk), save_fox).chain());
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
        load_schedule.add_systems((emit_load_for(chunk), load_fox).chain());
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
