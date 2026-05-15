use bevy_ecs::prelude::*;
use mobs::spawn::handle_spawn_mob_bundle;
use temper_components::entity_identity::Identity;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::player::position::Position;
use temper_core::dimension::Dimension;
use temper_entities::entity_types::EntityTypeEnum;
use temper_entities::markers::entity_types::{Axolotl, Bat, Cow, Fox, Pig};
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_entities::MobBundle;
use temper_entities::{AxolotlBundle, BatBundle, CowBundle, FoxBundle, MobKind, PigBundle};
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
    writer.write(SpawnMobBundle {
        bundle: MobBundle::Cow(CowBundle::new(Position::new(11.5, 64.0, 9.5))),
        persist: true,
    });
    writer.write(SpawnMobBundle {
        bundle: MobBundle::Bat(BatBundle::new(Position::new(14.5, 70.0, 9.5))),
        persist: true,
    });
    writer.write(SpawnMobBundle {
        bundle: MobBundle::Axolotl(AxolotlBundle::new(Position::new(17.5, 62.0, 9.5))),
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
        &MobKind,
        Has<Pig>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let pigs: Vec<_> = pig_query
        .iter(&world)
        .filter(|(_, _, _, _, is_pig, _, _, _)| *is_pig)
        .map(
            |(identity, position, last_chunk, mob_kind, is_pig, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    *mob_kind,
                    is_pig,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(pigs.len(), 1, "one pig should be spawned");
    let (
        pig_uuid,
        pig_position,
        pig_last_chunk,
        pig_kind,
        is_pig,
        pig_gravity,
        pig_collisions,
        pig_drag,
    ) = pigs[0];
    assert!(is_pig);
    assert_eq!(pig_kind, MobKind(EntityTypeEnum::Pig));
    assert!(pig_gravity);
    assert!(pig_collisions);
    assert!(pig_drag);
    assert_eq!(pig_last_chunk.0, pig_position.chunk());

    let mut fox_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        &MobKind,
        Has<Fox>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let foxes: Vec<_> = fox_query
        .iter(&world)
        .filter(|(_, _, _, _, is_fox, _, _, _)| *is_fox)
        .map(
            |(identity, position, last_chunk, mob_kind, is_fox, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    *mob_kind,
                    is_fox,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(foxes.len(), 1, "one fox should be spawned");
    let (
        fox_uuid,
        fox_position,
        fox_last_chunk,
        fox_kind,
        is_fox,
        fox_gravity,
        fox_collisions,
        fox_drag,
    ) = foxes[0];
    assert!(is_fox);
    assert_eq!(fox_kind, MobKind(EntityTypeEnum::Fox));
    assert!(fox_gravity);
    assert!(fox_collisions);
    assert!(fox_drag);
    assert_eq!(fox_last_chunk.0, fox_position.chunk());

    let mut cow_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        &MobKind,
        Has<Cow>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let cows: Vec<_> = cow_query
        .iter(&world)
        .filter(|(_, _, _, _, is_cow, _, _, _)| *is_cow)
        .map(
            |(identity, position, last_chunk, mob_kind, is_cow, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    *mob_kind,
                    is_cow,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(cows.len(), 1, "one cow should be spawned");
    let (
        cow_uuid,
        cow_position,
        cow_last_chunk,
        cow_kind,
        is_cow,
        cow_gravity,
        cow_collisions,
        cow_drag,
    ) = cows[0];
    assert!(is_cow);
    assert_eq!(cow_kind, MobKind(EntityTypeEnum::Cow));
    assert!(cow_gravity);
    assert!(cow_collisions);
    assert!(cow_drag);
    assert_eq!(cow_last_chunk.0, cow_position.chunk());

    let mut bat_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        &MobKind,
        Has<Bat>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let bats: Vec<_> = bat_query
        .iter(&world)
        .filter(|(_, _, _, _, is_bat, _, _, _)| *is_bat)
        .map(
            |(identity, position, last_chunk, mob_kind, is_bat, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    *mob_kind,
                    is_bat,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(bats.len(), 1, "one bat should be spawned");
    let (
        bat_uuid,
        bat_position,
        bat_last_chunk,
        bat_kind,
        is_bat,
        bat_gravity,
        bat_collisions,
        bat_drag,
    ) = bats[0];
    assert!(is_bat);
    assert_eq!(bat_kind, MobKind(EntityTypeEnum::Bat));
    assert!(!bat_gravity);
    assert!(bat_collisions);
    assert!(!bat_drag);
    assert_eq!(bat_last_chunk.0, bat_position.chunk());

    let mut axolotl_query = world.query::<(
        &Identity,
        &Position,
        &LastChunkPos,
        &MobKind,
        Has<Axolotl>,
        Has<HasGravity>,
        Has<HasCollisions>,
        Has<HasWaterDrag>,
    )>();
    let axolotls: Vec<_> = axolotl_query
        .iter(&world)
        .filter(|(_, _, _, _, is_axolotl, _, _, _)| *is_axolotl)
        .map(
            |(identity, position, last_chunk, mob_kind, is_axolotl, gravity, collisions, drag)| {
                (
                    identity.uuid,
                    *position,
                    *last_chunk,
                    *mob_kind,
                    is_axolotl,
                    gravity,
                    collisions,
                    drag,
                )
            },
        )
        .collect();
    assert_eq!(axolotls.len(), 1, "one axolotl should be spawned");
    let (
        axolotl_uuid,
        axolotl_position,
        axolotl_last_chunk,
        axolotl_kind,
        is_axolotl,
        axolotl_gravity,
        axolotl_collisions,
        axolotl_drag,
    ) = axolotls[0];
    assert!(is_axolotl);
    assert_eq!(axolotl_kind, MobKind(EntityTypeEnum::Axolotl));
    assert!(axolotl_gravity);
    assert!(axolotl_collisions);
    assert!(!axolotl_drag);
    assert_eq!(axolotl_last_chunk.0, axolotl_position.chunk());

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

    let cow_chunk = state
        .0
        .world
        .get_chunk(cow_position.chunk(), Dimension::Overworld)
        .expect("cow chunk should be present");
    let persisted_cow = cow_chunk
        .entities
        .get(&cow_uuid)
        .expect("cow should be persisted");
    assert_eq!(persisted_cow.value().0, EntityTypeEnum::Cow);

    let bat_chunk = state
        .0
        .world
        .get_chunk(bat_position.chunk(), Dimension::Overworld)
        .expect("bat chunk should be present");
    let persisted_bat = bat_chunk
        .entities
        .get(&bat_uuid)
        .expect("bat should be persisted");
    assert_eq!(persisted_bat.value().0, EntityTypeEnum::Bat);

    let axolotl_chunk = state
        .0
        .world
        .get_chunk(axolotl_position.chunk(), Dimension::Overworld)
        .expect("axolotl chunk should be present");
    let persisted_axolotl = axolotl_chunk
        .entities
        .get(&axolotl_uuid)
        .expect("axolotl should be persisted");
    assert_eq!(persisted_axolotl.value().0, EntityTypeEnum::Axolotl);
}
