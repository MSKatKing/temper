use background::{chunk_unloader, entity_unloader};
use bevy_ecs::prelude::*;
use mobs::ground::{load_fox, save_fox};
use player::chunk_calculator;
use temper_components::entity_identity::Identity;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::last_synced_position::LastSyncedPosition;
use temper_components::player::chunk_receiver::ChunkReceiver;
use temper_components::player::client_information::ClientInformationComponent;
use temper_components::player::player_marker::PlayerMarker;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_core::pos::ChunkPos;
use temper_entities::markers::entity_types::Fox;
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::FoxBundle;
use temper_messages::chunk_calc::ChunkCalc;
use temper_messages::load_chunk_entities::LoadChunkEntities;
use temper_state::create_test_state;

fn emit_chunk_calc_for(entity: Entity) -> impl FnMut(MessageWriter<ChunkCalc>) {
    move |mut writer: MessageWriter<ChunkCalc>| {
        writer.write(ChunkCalc(entity));
    }
}

fn emit_load_messages_for_known_chunks(
    state: Res<temper_state::GlobalStateResource>,
    mut query: Query<&mut ChunkReceiver>,
    mut writer: MessageWriter<LoadChunkEntities>,
) {
    for mut receiver in query.iter_mut() {
        while let Some((x, z)) = receiver.loading.pop_front() {
            let chunk = ChunkPos::new(x, z);
            receiver.loaded.insert((x, z));

            if state
                .0
                .world
                .chunk_exists(chunk, Dimension::Overworld)
                .expect("chunk existence check should succeed")
            {
                writer.write(LoadChunkEntities(chunk));
            }
        }
    }
}

#[test]
fn player_can_unload_entities_by_moving_away_and_reload_them_after_returning() {
    let mut world = World::new();
    temper_messages::register_messages(&mut world);

    let (state, _temp_dir) = create_test_state();
    world.insert_resource(state.clone());

    let fox_position = Position::new(8.0, 64.0, 8.0);
    let fox_chunk = fox_position.chunk();
    let fox_bundle = FoxBundle::new(fox_position);
    let expected_identity = fox_bundle.identity.clone();
    let expected_last_synced = fox_bundle.last_synced_position;

    let mut receiver = ChunkReceiver::default();
    receiver.loaded.insert((fox_chunk.x(), fox_chunk.z()));

    let player_entity = world
        .spawn((
            Position::new(8.0, 64.0, 8.0),
            receiver,
            ClientInformationComponent {
                view_distance: 2,
                ..Default::default()
            },
            PlayerMarker,
        ))
        .id();

    world.spawn((
        fox_bundle,
        Fox,
        HasGravity,
        HasCollisions,
        HasWaterDrag,
        LastChunkPos::new(fox_chunk),
    ));

    {
        let chunk = state
            .0
            .world
            .get_or_generate_chunk(fox_chunk, Dimension::Overworld)
            .expect("fox chunk should be cached before unload");
        chunk.entities.clear();
        chunk.mark_dirty();
    }

    {
        let mut player_pos = world
            .get_mut::<Position>(player_entity)
            .expect("player should exist");
        *player_pos = Position::new(2048.0, 64.0, 2048.0);
    }

    let mut chunk_calc_schedule = Schedule::default();
    chunk_calc_schedule
        .add_systems((emit_chunk_calc_for(player_entity), chunk_calculator::handle).chain());
    chunk_calc_schedule.run(&mut world);

    let mut unload_schedule = Schedule::default();
    unload_schedule
        .add_systems((entity_unloader::handle, save_fox, chunk_unloader::handle).chain());
    unload_schedule.run(&mut world);

    let mut fox_query = world.query::<EntityRef<'_>>();
    let live_foxes = fox_query
        .iter(&world)
        .filter(|entity| entity.contains::<Fox>())
        .count();
    assert_eq!(
        live_foxes, 0,
        "fox should despawn when its chunk is unloaded"
    );
    assert!(
        !state
            .0
            .world
            .get_cache()
            .contains_key(&(fox_chunk, Dimension::Overworld)),
        "fox chunk should be removed from cache after unload"
    );
    {
        let saved_chunk = state
            .0
            .world
            .get_chunk(fox_chunk, Dimension::Overworld)
            .expect("saved fox chunk should be readable from storage");
        assert_eq!(
            saved_chunk.entities.len(),
            1,
            "only the test fox should be persisted in the chunk"
        );
    }
    state.0.world.get_cache().clear();

    {
        let mut receiver = world
            .get_mut::<ChunkReceiver>(player_entity)
            .expect("player chunk receiver should exist");
        receiver.loading.clear();
        receiver.unloading.clear();
        receiver.dirty.clear();
        receiver.loaded.clear();
    }
    {
        let mut player_pos = world
            .get_mut::<Position>(player_entity)
            .expect("player should still exist");
        *player_pos = fox_position;
    }

    chunk_calc_schedule.run(&mut world);

    {
        let receiver = world
            .get::<ChunkReceiver>(player_entity)
            .expect("player chunk receiver should exist after returning");
        let queued_fox_chunks = receiver
            .loading
            .iter()
            .filter(|(x, z)| (*x, *z) == (fox_chunk.x(), fox_chunk.z()))
            .count();
        assert_eq!(
            queued_fox_chunks, 1,
            "player return should queue the fox chunk exactly once"
        );
    }

    let mut load_schedule = Schedule::default();
    load_schedule.add_systems((emit_load_messages_for_known_chunks, load_fox).chain());
    load_schedule.run(&mut world);

    let mut fox_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        &LastSyncedPosition,
        Has<Fox>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let loaded_foxes: Vec<_> = fox_query
        .iter(&world)
        .filter(|(_, _, _, _, is_fox, _, _, _)| *is_fox)
        .collect();

    assert_eq!(
        loaded_foxes.len(),
        1,
        "fox should reload when the player returns"
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
    ) = loaded_foxes[0];

    assert!(is_fox, "reloaded entity should have the Fox marker");
    assert!(has_gravity, "reloaded fox should regain HasGravity");
    assert!(has_collisions, "reloaded fox should regain HasCollisions");
    assert!(has_water_drag, "reloaded fox should regain HasWaterDrag");
    assert_eq!(identity.uuid, expected_identity.uuid);
    assert_eq!(identity.entity_id, expected_identity.entity_id);
    assert_eq!(loaded_position.coords, fox_position.coords);
    assert_eq!(last_chunk.0, fox_chunk);
    assert_eq!(last_synced.0, expected_last_synced.0);
}
