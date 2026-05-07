use bevy_ecs::prelude::MessageWriter;
use bimap::BiMap;
use lazy_static::lazy_static;
use temper_commands::{
    arg::{primitive::PrimitiveArgument, utils::parser_error, CommandArgument, ParserResult},
    CommandContext, Sender, Suggestion,
};
use temper_entities::entity_types::EntityTypeEnum;
use temper_macros::command;
use temper_messages::SpawnEntityCommand;
use temper_text::TextComponent;

/// Wrapper type for EntityType that implements CommandArgument
#[derive(Debug, Clone, Copy)]
struct EntityTypeArg(EntityTypeEnum);

lazy_static! {
    static ref MAPPED_ENTITIES: BiMap<&'static str, EntityTypeEnum> = {
        let mut m = BiMap::new();

        // Add supported entities here
        m.insert("allay", EntityTypeEnum::Allay);
        m.insert("armadillo", EntityTypeEnum::Armadillo);
        m.insert("axolotl", EntityTypeEnum::Axolotl);
        m.insert("bat", EntityTypeEnum::Bat);
        m.insert("bee", EntityTypeEnum::Bee);
        m.insert("camel", EntityTypeEnum::Camel);
        m.insert("cat", EntityTypeEnum::Cat);
        m.insert("cave_spider", EntityTypeEnum::CaveSpider);
        m.insert("chicken", EntityTypeEnum::Chicken);
        m.insert("cod", EntityTypeEnum::Cod);
        m.insert("cow", EntityTypeEnum::Cow);
        m.insert("dolphin", EntityTypeEnum::Dolphin);
        m.insert("donkey", EntityTypeEnum::Donkey);
        m.insert("drowned", EntityTypeEnum::Drowned);
        m.insert("enderman", EntityTypeEnum::Enderman);
        m.insert("fox", EntityTypeEnum::Fox);
        m.insert("frog", EntityTypeEnum::Frog);
        m.insert("goat", EntityTypeEnum::Goat);
        m.insert("horse", EntityTypeEnum::Horse);
        m.insert("iron_golem", EntityTypeEnum::IronGolem);
        m.insert("llama", EntityTypeEnum::Llama);
        m.insert("mooshroom", EntityTypeEnum::Mooshroom);
        m.insert("ocelot", EntityTypeEnum::Ocelot);
        m.insert("panda", EntityTypeEnum::Panda);
        m.insert("parrot", EntityTypeEnum::Parrot);
        m.insert("pig", EntityTypeEnum::Pig);
        m.insert("piglin", EntityTypeEnum::Piglin);
        m.insert("polar_bear", EntityTypeEnum::PolarBear);
        m.insert("pufferfish", EntityTypeEnum::Pufferfish);
        m.insert("rabbit", EntityTypeEnum::Rabbit);
        m.insert("salmon", EntityTypeEnum::Salmon);
        m.insert("sheep", EntityTypeEnum::Sheep);
        m.insert("skeleton_horse", EntityTypeEnum::SkeletonHorse);
        m.insert("sniffer", EntityTypeEnum::Sniffer);
        m.insert("snow_golem", EntityTypeEnum::SnowGolem);
        m.insert("spider", EntityTypeEnum::Spider);
        m.insert("squid", EntityTypeEnum::Squid);
        m.insert("strider", EntityTypeEnum::Strider);
        m.insert("tadpole", EntityTypeEnum::Tadpole);
        m.insert("trader_llama", EntityTypeEnum::TraderLlama);
        m.insert("tropical_fish", EntityTypeEnum::TropicalFish);
        m.insert("turtle", EntityTypeEnum::Turtle);
        m.insert("villager", EntityTypeEnum::Villager);
        m.insert("wandering_trader", EntityTypeEnum::WanderingTrader);
        m.insert("wolf", EntityTypeEnum::Wolf);
        m.insert("zombie_horse", EntityTypeEnum::ZombieHorse);
        m.insert("zombified_piglin", EntityTypeEnum::ZombifiedPiglin);
        m.insert("glow_squid", EntityTypeEnum::GlowSquid);
        m.insert("mule", EntityTypeEnum::Mule);

        // Hostile entities
        m.insert("blaze", EntityTypeEnum::Blaze);
        m.insert("bogged", EntityTypeEnum::Bogged);
        m.insert("breeze", EntityTypeEnum::Breeze);
        m.insert("creaking", EntityTypeEnum::Creaking);
        m.insert("creeper", EntityTypeEnum::Creeper);
        m.insert("elder_guardian", EntityTypeEnum::ElderGuardian);
        m.insert("endermite", EntityTypeEnum::Endermite);
        m.insert("evoker", EntityTypeEnum::Evoker);
        m.insert("ghast", EntityTypeEnum::Ghast);
        m.insert("guardian", EntityTypeEnum::Guardian);
        m.insert("hoglin", EntityTypeEnum::Hoglin);
        m.insert("husk", EntityTypeEnum::Husk);
        m.insert("magma_cube", EntityTypeEnum::MagmaCube);
        m.insert("phantom", EntityTypeEnum::Phantom);
        m.insert("piglin_brute", EntityTypeEnum::PiglinBrute);
        m.insert("pillager", EntityTypeEnum::Pillager);
        m.insert("ravager", EntityTypeEnum::Ravager);
        m.insert("shulker", EntityTypeEnum::Shulker);
        m.insert("silverfish", EntityTypeEnum::Silverfish);
        m.insert("skeleton", EntityTypeEnum::Skeleton);
        m.insert("slime", EntityTypeEnum::Slime);
        m.insert("stray", EntityTypeEnum::Stray);
        m.insert("vex", EntityTypeEnum::Vex);
        m.insert("vindicator", EntityTypeEnum::Vindicator);
        m.insert("warden", EntityTypeEnum::Warden);
        m.insert("witch", EntityTypeEnum::Witch);
        m.insert("wither_skeleton", EntityTypeEnum::WitherSkeleton);
        m.insert("zoglin", EntityTypeEnum::Zoglin);
        m.insert("zombie", EntityTypeEnum::Zombie);
        m.insert("zombie_villager", EntityTypeEnum::ZombieVillager);

        m
    };
}

impl CommandArgument for EntityTypeArg {
    fn parse(ctx: &mut CommandContext) -> ParserResult<Self> {
        let str = ctx.input.read_string();

        let value = match MAPPED_ENTITIES.get_by_left(str.as_str()) {
            Some(&entity_type) => entity_type,
            None => {
                return Err(parser_error(
                    format!("Unknown entity type: {}", str).as_str(),
                ))
            }
        };

        Ok(EntityTypeArg(value))
    }

    fn primitive() -> PrimitiveArgument {
        // We're parsing a single word
        PrimitiveArgument::word()
    }

    fn suggest(ctx: &mut CommandContext) -> Vec<Suggestion> {
        ctx.input.read_string();

        MAPPED_ENTITIES
            .iter()
            .map(|(&name, _)| Suggestion::of(name))
            .collect()
    }
}

/// Spawns an entity in front of the player.
///
/// Usage: /spawn <entity_type>
/// Supported: allay, armadillo, axolotl, bat, bee, camel, cat, chicken, cod, cow, dolphin, donkey, fox, frog, goat, horse, llama, mooshroom, ocelot, panda, parrot, pig
#[command("spawn")]
fn spawn_command(
    #[sender] sender: Sender,
    #[arg] entity_type: EntityTypeArg,
    mut spawn_commands: MessageWriter<SpawnEntityCommand>,
) {
    match sender {
        Sender::Player(entity) => {
            // Write spawn command message - will be processed by spawn_command_processor system
            spawn_commands.write(SpawnEntityCommand {
                entity_type: entity_type.0,
                player_entity: entity,
            });

            // Get entity name for message
            let entity_name = MAPPED_ENTITIES
                .get_by_right(&entity_type.0)
                .unwrap_or(&"unknown");

            sender.send_message(
                TextComponent::from(format!("{} spawned!", entity_name)),
                false,
            );
        }
        Sender::Server => {
            sender.send_message(
                TextComponent::from("Only players can use this command"),
                false,
            );
        }
    }
}
