use bevy_ecs::prelude::*;
use temper_components::last_chunk_pos::LastChunkPos;
use temper_components::player::entity_tracker::EntityTracker;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_entities::bundles::*;
use temper_entities::entity_types::EntityTypeEnum;
use temper_entities::markers::entity_types::*;
use temper_entities::markers::{HasCollisions, HasGravity, HasWaterDrag};
use temper_messages::{SpawnEntityCommand, SpawnEntityEvent};
use temper_state::GlobalStateResource;
use tracing::warn;

/// Macro for spawning ground entities (gravity + collisions + water drag)
macro_rules! spawn_ground_entity {
    ($commands:expr, $position:expr, $Bundle:ident, $Marker:ident, $State:ident, $EType:path, $Query:ident) => {{
        let bundle = $Bundle::new($position);
        let uuid = bundle.identity.uuid;
        let last_chunk = LastChunkPos::new(bundle.position.chunk());
        let chunk = $State
            .world
            .get_or_generate_chunk(
                $position.chunk(),
                temper_core::dimension::Dimension::Overworld,
            )
            .expect("Failed to get or generate chunk");
        chunk.entities.insert(
            uuid,
            (
                $EType,
                bitcode::serialize(&bundle).expect("Failed to serialize entity bundle"),
            ),
        );
        chunk.mark_dirty();
        $commands.spawn((
            bundle,
            $Marker,
            HasGravity,
            HasCollisions,
            HasWaterDrag,
            last_chunk,
        ));
        $Query.iter().for_each(|tracker| {
            tracker.to_track.push((uuid, $EType.to_entity_type().id));
        });
    }};
}

/// Macro for spawning flying/swimming entities (collisions only)
macro_rules! spawn_flying_entity {
    ($commands:expr, $position:expr, $Bundle:ident, $Marker:ident, $State:ident, $EType:path, $Query:ident) => {{
        let bundle = $Bundle::new($position);
        let uuid = bundle.identity.uuid;
        let last_chunk = LastChunkPos::new(bundle.position.chunk());
        let chunk = $State
            .world
            .get_or_generate_chunk(
                $position.chunk(),
                temper_core::dimension::Dimension::Overworld,
            )
            .expect("Failed to get or generate chunk");
        chunk.entities.insert(
            uuid,
            (
                $EType,
                bitcode::serialize(&bundle).expect("Failed to serialize entity bundle"),
            ),
        );
        chunk.mark_dirty();
        $commands.spawn((bundle, $Marker, HasCollisions, last_chunk));
        $Query.iter().for_each(|tracker| {
            tracker.to_track.push((uuid, $EType.to_entity_type().id));
        });
    }};
}

/// Macro for spawning entities with gravity but no water drag (lava/amphibian creatures)
macro_rules! spawn_gravity_entity {
    ($commands:expr, $position:expr, $Bundle:ident, $Marker:ident, $State:ident, $EType:path, $Query:ident) => {{
        let bundle = $Bundle::new($position);
        let uuid = bundle.identity.uuid;
        let last_chunk = LastChunkPos::new(bundle.position.chunk());
        let chunk = $State
            .world
            .get_or_generate_chunk(
                $position.chunk(),
                temper_core::dimension::Dimension::Overworld,
            )
            .expect("Failed to get or generate chunk");
        chunk.entities.insert(
            uuid,
            (
                $EType,
                bitcode::serialize(&bundle).expect("Failed to serialize entity bundle"),
            ),
        );
        chunk.mark_dirty();
        $commands.spawn((bundle, $Marker, HasGravity, HasCollisions, last_chunk));
        $Query.iter().for_each(|tracker| {
            tracker.to_track.push((uuid, $EType.to_entity_type().id));
        });
    }};
}

/// System that processes spawn commands from messages
pub fn spawn_command_processor(
    mut spawn_commands: MessageReader<SpawnEntityCommand>,
    query: Query<(&Position, &Rotation)>,
    mut spawn_events: MessageWriter<SpawnEntityEvent>,
) {
    // Process all spawn command messages
    for command in spawn_commands.read() {
        // Get player position and rotation
        if let Ok((pos, rot)) = query.get(command.player_entity) {
            // Calculate spawn position 2 blocks in front of the player
            let spawn_pos = pos.offset_forward(rot, 2.0);

            spawn_events.write(SpawnEntityEvent {
                entity_type: command.entity_type,
                position: spawn_pos,
            });
        } else {
            warn!(
                "Failed to get position for entity {:?}",
                command.player_entity
            );
        }
    }
}

/// System that listens for `SpawnEntityEvent` and spawns the entity,
/// then broadcasts the spawn packet.
pub fn handle_spawn_entity(
    mut events: MessageReader<SpawnEntityEvent>,
    mut commands: Commands,
    state: Res<GlobalStateResource>,
    query: Query<&EntityTracker>,
) {
    for event in events.read() {
        let pos = event.position;
        let state = state.0.clone();
        match event.entity_type {
            // Ground entities (gravity + collisions + water drag)
            EntityTypeEnum::Pig => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    PigBundle,
                    Pig,
                    state,
                    EntityTypeEnum::Pig,
                    query
                )
            }
            EntityTypeEnum::Cow => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    CowBundle,
                    Cow,
                    state,
                    EntityTypeEnum::Cow,
                    query
                )
            }
            EntityTypeEnum::Armadillo => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    ArmadilloBundle,
                    Armadillo,
                    state,
                    EntityTypeEnum::Armadillo,
                    query
                )
            }
            EntityTypeEnum::Camel => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    CamelBundle,
                    Camel,
                    state,
                    EntityTypeEnum::Camel,
                    query
                )
            }
            EntityTypeEnum::Cat => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    CatBundle,
                    Cat,
                    state,
                    EntityTypeEnum::Cat,
                    query
                )
            }
            EntityTypeEnum::CaveSpider => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    CaveSpiderBundle,
                    CaveSpider,
                    state,
                    EntityTypeEnum::CaveSpider,
                    query
                )
            }
            EntityTypeEnum::Chicken => spawn_ground_entity!(
                commands,
                pos,
                ChickenBundle,
                Chicken,
                state,
                EntityTypeEnum::Chicken,
                query
            ),
            EntityTypeEnum::Donkey => spawn_ground_entity!(
                commands,
                pos,
                DonkeyBundle,
                Donkey,
                state,
                EntityTypeEnum::Donkey,
                query
            ),
            EntityTypeEnum::Enderman => spawn_ground_entity!(
                commands,
                pos,
                EndermanBundle,
                Enderman,
                state,
                EntityTypeEnum::Enderman,
                query
            ),
            EntityTypeEnum::Fox => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    FoxBundle,
                    Fox,
                    state,
                    EntityTypeEnum::Fox,
                    query
                )
            }
            EntityTypeEnum::Frog => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    FrogBundle,
                    Frog,
                    state,
                    EntityTypeEnum::Frog,
                    query
                )
            }
            EntityTypeEnum::Goat => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    GoatBundle,
                    Goat,
                    state,
                    EntityTypeEnum::Goat,
                    query
                )
            }
            EntityTypeEnum::Horse => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    HorseBundle,
                    Horse,
                    state,
                    EntityTypeEnum::Horse,
                    query
                )
            }
            EntityTypeEnum::IronGolem => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    IronGolemBundle,
                    IronGolem,
                    state,
                    EntityTypeEnum::IronGolem,
                    query
                )
            }
            EntityTypeEnum::Llama => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    LlamaBundle,
                    Llama,
                    state,
                    EntityTypeEnum::Llama,
                    query
                )
            }
            EntityTypeEnum::Mooshroom => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    MooshroomBundle,
                    Mooshroom,
                    state,
                    EntityTypeEnum::Mooshroom,
                    query
                )
            }
            EntityTypeEnum::Mule => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    MuleBundle,
                    Mule,
                    state,
                    EntityTypeEnum::Mule,
                    query
                )
            }
            EntityTypeEnum::Ocelot => spawn_ground_entity!(
                commands,
                pos,
                OcelotBundle,
                Ocelot,
                state,
                EntityTypeEnum::Ocelot,
                query
            ),
            EntityTypeEnum::Panda => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    PandaBundle,
                    Panda,
                    state,
                    EntityTypeEnum::Panda,
                    query
                )
            }
            EntityTypeEnum::Piglin => spawn_ground_entity!(
                commands,
                pos,
                PiglinBundle,
                Piglin,
                state,
                EntityTypeEnum::Piglin,
                query
            ),
            EntityTypeEnum::PolarBear => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    PolarBearBundle,
                    PolarBear,
                    state,
                    EntityTypeEnum::PolarBear,
                    query
                )
            }
            EntityTypeEnum::Rabbit => spawn_ground_entity!(
                commands,
                pos,
                RabbitBundle,
                Rabbit,
                state,
                EntityTypeEnum::Rabbit,
                query
            ),
            EntityTypeEnum::Sheep => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    SheepBundle,
                    Sheep,
                    state,
                    EntityTypeEnum::Sheep,
                    query
                )
            }
            EntityTypeEnum::SkeletonHorse => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    SkeletonHorseBundle,
                    SkeletonHorse,
                    state,
                    EntityTypeEnum::SkeletonHorse,
                    query
                )
            }
            EntityTypeEnum::Sniffer => spawn_ground_entity!(
                commands,
                pos,
                SnifferBundle,
                Sniffer,
                state,
                EntityTypeEnum::Sniffer,
                query
            ),
            EntityTypeEnum::Spider => spawn_ground_entity!(
                commands,
                pos,
                SpiderBundle,
                Spider,
                state,
                EntityTypeEnum::Spider,
                query
            ),
            EntityTypeEnum::SnowGolem => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    SnowGolemBundle,
                    SnowGolem,
                    state,
                    EntityTypeEnum::SnowGolem,
                    query
                )
            }
            EntityTypeEnum::TraderLlama => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    TraderLlamaBundle,
                    TraderLlama,
                    state,
                    EntityTypeEnum::TraderLlama,
                    query
                )
            }
            EntityTypeEnum::Turtle => spawn_ground_entity!(
                commands,
                pos,
                TurtleBundle,
                Turtle,
                state,
                EntityTypeEnum::Turtle,
                query
            ),
            EntityTypeEnum::Villager => spawn_ground_entity!(
                commands,
                pos,
                VillagerBundle,
                Villager,
                state,
                EntityTypeEnum::Villager,
                query
            ),
            EntityTypeEnum::WanderingTrader => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    WanderingTraderBundle,
                    WanderingTrader,
                    state,
                    EntityTypeEnum::WanderingTrader,
                    query
                )
            }
            EntityTypeEnum::Wolf => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    WolfBundle,
                    Wolf,
                    state,
                    EntityTypeEnum::Wolf,
                    query
                )
            }
            EntityTypeEnum::ZombieHorse => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    ZombieHorseBundle,
                    ZombieHorse,
                    state,
                    EntityTypeEnum::ZombieHorse,
                    query
                )
            }
            EntityTypeEnum::ZombifiedPiglin => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    ZombifiedPiglinBundle,
                    ZombifiedPiglin,
                    state,
                    EntityTypeEnum::ZombifiedPiglin,
                    query
                )
            }

            // Flying entities (collisions only)
            EntityTypeEnum::Allay => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    AllayBundle,
                    Allay,
                    state,
                    EntityTypeEnum::Allay,
                    query
                )
            }
            EntityTypeEnum::Bat => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    BatBundle,
                    Bat,
                    state,
                    EntityTypeEnum::Bat,
                    query
                )
            }
            EntityTypeEnum::Bee => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    BeeBundle,
                    Bee,
                    state,
                    EntityTypeEnum::Bee,
                    query
                )
            }
            EntityTypeEnum::Parrot => spawn_flying_entity!(
                commands,
                pos,
                ParrotBundle,
                Parrot,
                state,
                EntityTypeEnum::Parrot,
                query
            ),

            // Water creatures (collisions only, no gravity/water drag)
            EntityTypeEnum::Cod => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    CodBundle,
                    Cod,
                    state,
                    EntityTypeEnum::Cod,
                    query
                )
            }
            EntityTypeEnum::Dolphin => spawn_flying_entity!(
                commands,
                pos,
                DolphinBundle,
                Dolphin,
                state,
                EntityTypeEnum::Dolphin,
                query
            ),
            EntityTypeEnum::Drowned => spawn_flying_entity!(
                commands,
                pos,
                DrownedBundle,
                Drowned,
                state,
                EntityTypeEnum::Drowned,
                query
            ),
            EntityTypeEnum::GlowSquid => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    GlowSquidBundle,
                    GlowSquid,
                    state,
                    EntityTypeEnum::GlowSquid,
                    query
                )
            }
            EntityTypeEnum::Pufferfish => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    PufferfishBundle,
                    Pufferfish,
                    state,
                    EntityTypeEnum::Pufferfish,
                    query
                )
            }
            EntityTypeEnum::Salmon => spawn_flying_entity!(
                commands,
                pos,
                SalmonBundle,
                Salmon,
                state,
                EntityTypeEnum::Salmon,
                query
            ),
            EntityTypeEnum::Squid => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    SquidBundle,
                    Squid,
                    state,
                    EntityTypeEnum::Squid,
                    query
                )
            }
            EntityTypeEnum::Tadpole => spawn_flying_entity!(
                commands,
                pos,
                TadpoleBundle,
                Tadpole,
                state,
                EntityTypeEnum::Tadpole,
                query
            ),
            EntityTypeEnum::TropicalFish => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    TropicalFishBundle,
                    TropicalFish,
                    state,
                    EntityTypeEnum::TropicalFish,
                    query
                )
            }

            // Special: gravity but no water drag (amphibians, lava creatures)
            EntityTypeEnum::Axolotl => spawn_gravity_entity!(
                commands,
                pos,
                AxolotlBundle,
                Axolotl,
                state,
                EntityTypeEnum::Axolotl,
                query
            ),
            EntityTypeEnum::Strider => spawn_gravity_entity!(
                commands,
                pos,
                StriderBundle,
                Strider,
                state,
                EntityTypeEnum::Strider,
                query
            ),
            EntityTypeEnum::MagmaCube => {
                spawn_gravity_entity!(
                    commands,
                    pos,
                    MagmaCubeBundle,
                    MagmaCube,
                    state,
                    EntityTypeEnum::MagmaCube,
                    query
                )
            }
            EntityTypeEnum::Slime => {
                spawn_gravity_entity!(
                    commands,
                    pos,
                    SlimeBundle,
                    Slime,
                    state,
                    EntityTypeEnum::Slime,
                    query
                )
            }

            // Hostile ground entities
            EntityTypeEnum::Bogged => spawn_ground_entity!(
                commands,
                pos,
                BoggedBundle,
                Bogged,
                state,
                EntityTypeEnum::Bogged,
                query
            ),
            EntityTypeEnum::Creaking => spawn_ground_entity!(
                commands,
                pos,
                CreakingBundle,
                Creaking,
                state,
                EntityTypeEnum::Creaking,
                query
            ),
            EntityTypeEnum::Creeper => spawn_ground_entity!(
                commands,
                pos,
                CreeperBundle,
                Creeper,
                state,
                EntityTypeEnum::Creeper,
                query
            ),
            EntityTypeEnum::Endermite => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    EndermiteBundle,
                    Endermite,
                    state,
                    EntityTypeEnum::Endermite,
                    query
                )
            }
            EntityTypeEnum::Evoker => spawn_ground_entity!(
                commands,
                pos,
                EvokerBundle,
                Evoker,
                state,
                EntityTypeEnum::Evoker,
                query
            ),
            EntityTypeEnum::Hoglin => spawn_ground_entity!(
                commands,
                pos,
                HoglinBundle,
                Hoglin,
                state,
                EntityTypeEnum::Hoglin,
                query
            ),
            EntityTypeEnum::Husk => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    HuskBundle,
                    Husk,
                    state,
                    EntityTypeEnum::Husk,
                    query
                )
            }
            EntityTypeEnum::PiglinBrute => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    PiglinBruteBundle,
                    PiglinBrute,
                    state,
                    EntityTypeEnum::PiglinBrute,
                    query
                )
            }
            EntityTypeEnum::Pillager => spawn_ground_entity!(
                commands,
                pos,
                PillagerBundle,
                Pillager,
                state,
                EntityTypeEnum::Pillager,
                query
            ),
            EntityTypeEnum::Ravager => spawn_ground_entity!(
                commands,
                pos,
                RavagerBundle,
                Ravager,
                state,
                EntityTypeEnum::Ravager,
                query
            ),
            EntityTypeEnum::Silverfish => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    SilverfishBundle,
                    Silverfish,
                    state,
                    EntityTypeEnum::Silverfish,
                    query
                )
            }
            EntityTypeEnum::Skeleton => spawn_ground_entity!(
                commands,
                pos,
                SkeletonBundle,
                Skeleton,
                state,
                EntityTypeEnum::Skeleton,
                query
            ),
            EntityTypeEnum::Stray => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    StrayBundle,
                    Stray,
                    state,
                    EntityTypeEnum::Stray,
                    query
                )
            }
            EntityTypeEnum::Vindicator => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    VindicatorBundle,
                    Vindicator,
                    state,
                    EntityTypeEnum::Vindicator,
                    query
                )
            }
            EntityTypeEnum::Warden => spawn_ground_entity!(
                commands,
                pos,
                WardenBundle,
                Warden,
                state,
                EntityTypeEnum::Warden,
                query
            ),
            EntityTypeEnum::Witch => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    WitchBundle,
                    Witch,
                    state,
                    EntityTypeEnum::Witch,
                    query
                )
            }
            EntityTypeEnum::WitherSkeleton => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    WitherSkeletonBundle,
                    WitherSkeleton,
                    state,
                    EntityTypeEnum::WitherSkeleton,
                    query
                )
            }
            EntityTypeEnum::Zoglin => spawn_ground_entity!(
                commands,
                pos,
                ZoglinBundle,
                Zoglin,
                state,
                EntityTypeEnum::Zoglin,
                query
            ),
            EntityTypeEnum::Zombie => spawn_ground_entity!(
                commands,
                pos,
                ZombieBundle,
                Zombie,
                state,
                EntityTypeEnum::Zombie,
                query
            ),
            EntityTypeEnum::ZombieVillager => {
                spawn_ground_entity!(
                    commands,
                    pos,
                    ZombieVillagerBundle,
                    ZombieVillager,
                    state,
                    EntityTypeEnum::ZombieVillager,
                    query
                )
            }
            EntityTypeEnum::Shulker => spawn_ground_entity!(
                commands,
                pos,
                ShulkerBundle,
                Shulker,
                state,
                EntityTypeEnum::Shulker,
                query
            ),

            // Hostile flying entities
            EntityTypeEnum::Blaze => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    BlazeBundle,
                    Blaze,
                    state,
                    EntityTypeEnum::Blaze,
                    query
                )
            }
            EntityTypeEnum::Breeze => spawn_flying_entity!(
                commands,
                pos,
                BreezeBundle,
                Breeze,
                state,
                EntityTypeEnum::Breeze,
                query
            ),
            EntityTypeEnum::Ghast => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    GhastBundle,
                    Ghast,
                    state,
                    EntityTypeEnum::Ghast,
                    query
                )
            }
            EntityTypeEnum::Phantom => spawn_flying_entity!(
                commands,
                pos,
                PhantomBundle,
                Phantom,
                state,
                EntityTypeEnum::Phantom,
                query
            ),
            EntityTypeEnum::Vex => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    VexBundle,
                    Vex,
                    state,
                    EntityTypeEnum::Vex,
                    query
                )
            }

            // Hostile water entities
            EntityTypeEnum::ElderGuardian => {
                spawn_flying_entity!(
                    commands,
                    pos,
                    ElderGuardianBundle,
                    ElderGuardian,
                    state,
                    EntityTypeEnum::ElderGuardian,
                    query
                )
            }
            EntityTypeEnum::Guardian => spawn_flying_entity!(
                commands,
                pos,
                GuardianBundle,
                Guardian,
                state,
                EntityTypeEnum::Guardian,
                query
            ),
        }
    }
}
