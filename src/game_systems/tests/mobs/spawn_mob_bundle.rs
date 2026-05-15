use bevy_ecs::prelude::*;
use mobs::spawn::handle_spawn_mob_bundle;
use temper_components::entity_identity::Identity;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_entities::entity_types::EntityTypeEnum;
use temper_entities::markers::entity_types::{Fox, Pig};
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::MobBundle;
use temper_entities::{FoxBundle, PigBundle};
use temper_messages::SpawnMobBundle;
use temper_state::create_test_state;

fn emit_spawn_bundles(mut writer: MessageWriter<SpawnMobBundle>) {
    writer.write(SpawnMobBundle {
        bundle: MobBundle::Pig(PigBundle::new(Position::new(5.5, 64.0, 7.5))),
        persist: true,
    });
    writer.write(SpawnMobBundle {
        bundle: MobBundle::Fox(FoxBundle::new(Position::new(8.5, 64.0, 9.5))),
        persist: true,
    });
}

#[test]
fn spawn_mob_bundle_message_spawns_and_persists_supported_mobs() {
    let mut world = World::new();
    temper_messages::register_messages(&mut world);

    let (state, _temp_dir) = create_test_state();
    world.insert_resource(state.clone());

    let mut schedule = Schedule::default();
    schedule.add_systems((emit_spawn_bundles, handle_spawn_mob_bundle).chain());
    schedule.run(&mut world);

    let mut pig_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        Has<Pig>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let pigs: Vec<_> = pig_query
        .iter(&world)
        .filter(|(_, _, _, is_pig, _, _, _)| *is_pig)
        .map(
            |(identity, position, last_chunk, is_pig, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    is_pig,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(pigs.len(), 1, "one pig should be spawned");
    let (pig_uuid, pig_position, pig_last_chunk, is_pig, pig_gravity, pig_collisions, pig_drag) =
        pigs[0];
    assert!(is_pig);
    assert!(pig_gravity);
    assert!(pig_collisions);
    assert!(pig_drag);
    assert_eq!(pig_last_chunk.0, pig_position.chunk());

    let mut fox_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        Has<Fox>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let foxes: Vec<_> = fox_query
        .iter(&world)
        .filter(|(_, _, _, is_fox, _, _, _)| *is_fox)
        .map(
            |(identity, position, last_chunk, is_fox, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    is_fox,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(foxes.len(), 1, "one fox should be spawned");
    let (fox_uuid, fox_position, fox_last_chunk, is_fox, fox_gravity, fox_collisions, fox_drag) =
        foxes[0];
    assert!(is_fox);
    assert!(fox_gravity);
    assert!(fox_collisions);
    assert!(fox_drag);
    assert_eq!(fox_last_chunk.0, fox_position.chunk());

    let pig_chunk = state
        .0
        .world
        .get_chunk(pig_position.chunk(), Dimension::Overworld)
        .expect("pig chunk should be present");
    let persisted_pig = pig_chunk
        .entities
        .get(&pig_uuid)
        .expect("pig should be persisted");
    assert_eq!(persisted_pig.value().0, EntityTypeEnum::Pig);

    let fox_chunk = state
        .0
        .world
        .get_chunk(fox_position.chunk(), Dimension::Overworld)
        .expect("fox chunk should be present");
    let persisted_fox = fox_chunk
        .entities
        .get(&fox_uuid)
        .expect("fox should be persisted");
    assert_eq!(persisted_fox.value().0, EntityTypeEnum::Fox);
}
