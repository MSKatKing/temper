pub mod banner_pattern;
pub mod block_predicate;
pub mod consume_effects;
pub mod firework;
pub mod instrument;
pub mod jukebox;
pub mod kinetic_weapon;
pub mod painting_variant;
pub mod potion_effect;
pub mod trim;

pub use banner_pattern::{BannerPattern, BannerPatternLayer};
pub use block_predicate::BlockPredicate;
pub use consume_effects::ConsumeEffect;
pub use firework::{FireworkExplosion, FireworkExplosionShape, Fireworks};
pub use instrument::Instrument;
pub use jukebox::JukeboxSong;
pub use kinetic_weapon::{KineticWeapon, KineticWeaponConditions};
pub use painting_variant::PaintingVariant;
pub use potion_effect::PotionEffect;
pub use trim::{Trim, TrimMaterial, TrimMaterialOverride, TrimPattern};

use crate::slot::InventorySlot;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use temper_codec::decode::errors::NetDecodeError;
use temper_codec::decode::{NetDecode, NetDecodeOpts};
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::id_or_inline::IdOr;
use temper_codec::net_types::id_set::IDSet;
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_codec::net_types::network_position::NetworkPosition;
use temper_codec::net_types::prefixed_optional::PrefixedOptional;
use temper_codec::net_types::var_int::VarInt;
use temper_macros::{Discriminant, InverseDiscriminant, NetDecode, NetEncode};
use temper_nbt::NBT;
use temper_nbt::blob::NbtBlob;
use temper_text::TextComponent;
use type_hash::TypeHash;

macro_rules! impl_discriminant_net_encode {
    ($type:ty) => {
        impl NetEncode for $type {
            fn encode<W: Write>(
                &self,
                writer: &mut W,
                opts: &NetEncodeOpts,
            ) -> Result<(), NetEncodeError> {
                VarInt::new(self.discriminant()).encode(writer, opts)
            }
        }
    };
}

macro_rules! impl_discriminant_net_decode {
    ($type:ty) => {
        impl NetDecode for $type {
            fn decode<R: Read>(
                reader: &mut R,
                opts: &NetDecodeOpts,
            ) -> Result<Self, NetDecodeError> {
                Self::from_discriminant(VarInt::decode(reader, opts)?)
                    .ok_or(NetDecodeError::InvalidEnumVariant)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TypeHash)]
pub enum ItemComponent {
    CustomData(NbtBlob),
    MaxStackSize(VarInt),
    MaxDamage(VarInt),
    Damage(VarInt),
    Unbreakable,

    CustomName(Box<TextComponent>),
    ItemName(Box<TextComponent>),
    ItemModel(String),
    Lore(LengthPrefixedVec<TextComponent>),
    Rarity(Rarity),
    Enchantments(LengthPrefixedVec<Enchantment>),

    CanPlaceOn(LengthPrefixedVec<BlockPredicate>),
    CanBreak(LengthPrefixedVec<BlockPredicate>),

    AttributeModifiers(LengthPrefixedVec<AttributeModifier>),
    CustomModelData(CustomModelData),
    TooltipDisplay(TooltipDisplay),
    RepairCost(VarInt),
    CreativeSlotLock,
    EnchantmentGlintOverride(bool),
    IntangibleProjectile(NbtBlob),
    Food(Food),

    Consumable(Consumable),

    UseRemainder(Box<InventorySlot>),
    UseCooldown(UseCooldown),
    UseEffects(UseEffects),
    MinimumAttackCharge(f32),
    DamageType(VarInt),

    DamageResistant(IDSet),

    AttackRange(AttackRange),

    Tool(Tool),

    Weapon(Weapon),
    Enchantable(VarInt),

    Equippable(Equippable),

    Repairable(IDSet),

    Glider,
    TooltipStyle(String),

    DeathProtection(LengthPrefixedVec<ConsumeEffect>),

    BlocksAttacks(BlocksAttacks),

    StoredEnchantments(LengthPrefixedVec<Enchantment>),
    DyedColor(i32),
    MapColor(i32),
    MapId(VarInt),
    MapDecorations(NbtBlob),
    MapPostProcessing(MapPostProcessing),
    ChargedProjectiles(LengthPrefixedVec<InventorySlot>),
    BundleContents(LengthPrefixedVec<InventorySlot>),

    PotionContents(PotionContents),

    PotionDurationScale(f32),
    SuspiciousStewEffects(LengthPrefixedVec<SuspiciousStewEffect>),
    WritableBookContent(LengthPrefixedVec<WritableBookPage>),
    WrittenBookContent(WrittenBookContent),

    Trim(Box<Trim>),

    DebugStickState(NbtBlob),
    EntityData(EntityData),
    BucketEntityData(NbtBlob),
    BlockEntityData(BlockEntityData),

    Instrument(IdOr<Instrument>),

    PiercingWeapon(PiercingWeapon),

    KineticWeapon(KineticWeapon),

    SwingAnimation(SwingAnimation),
    AdditionalTradeCost(VarInt),
    Dye(DyeColor),

    ProvidesTrimMaterial(IdOr<TrimMaterial>),

    OminousBottleAmplifier(VarInt),

    JukeboxPlayable(IdOr<JukeboxSong>),

    ProvidesBannerPatterns(IDSet),

    Recipes(NbtBlob),
    LodestoneTracker(LodestoneTracker),

    FireworkExplosion(FireworkExplosion),
    Fireworks(Fireworks),

    // not implemented
    Profile,

    NoteBlockSound(String),

    BannerPatterns(LengthPrefixedVec<BannerPatternLayer>),

    BaseColor(DyeColor),
    PotDecorations(LengthPrefixedVec<VarInt>),
    Container(LengthPrefixedVec<InventorySlot>),
    BlockState(LengthPrefixedVec<BlockStateProperty>),
    Bees(LengthPrefixedVec<Bee>),
    Lock(NbtBlob),
    ContainerLoot(NbtBlob),

    BreakSound(IdOr<SoundEvent>),

    SulfurCubeContent(Box<InventorySlot>),
    VillagerVariant(VarInt),
    WolfVariant(VarInt),
    WolfSoundVariant(VarInt),
    WolfCollar(DyeColor),
    FoxVariant(VarInt),
    SalmonSize(VarInt),
    ParrotVariant(VarInt),
    TropicalFishPattern(VarInt),
    TropicalFishBaseColor(DyeColor),
    TropicalFishPatternColor(DyeColor),
    MooshroomVariant(VarInt),
    RabbitVariant(VarInt),
    PigVariant(VarInt),
    PigSoundVariant(VarInt),
    CowVariant(VarInt),
    CowSoundVariant(VarInt),
    ChickenVariant(VarInt),
    ChickenSoundVariant(VarInt),
    FrogVariant(VarInt),
    HorseVariant(VarInt),

    PaintingVariant(IdOr<PaintingVariant>),

    LlamaVariant(VarInt),
    AxolotlVariant(VarInt),
    ZombieNautilusVariant(VarInt),
    CatVariant(VarInt),
    CatSoundVariant(VarInt),
    CatCollar(DyeColor),
    SheepColor(DyeColor),
    ShulkerColor(DyeColor),
}

impl Default for ItemComponent {
    fn default() -> Self {
        Self::CustomData(NbtBlob::default())
    }
}

include!(concat!(env!("OUT_DIR"), "/item_component_ids.rs"));

#[derive(
    Discriminant, InverseDiscriminant, Clone, Debug, PartialEq, Serialize, Deserialize, TypeHash,
)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl_discriminant_net_encode!(Rarity);
impl_discriminant_net_decode!(Rarity);

#[derive(NetEncode, NetDecode, PartialEq, Debug, Clone, Serialize, Deserialize, TypeHash)]
pub struct Enchantment {
    pub type_id: VarInt,
    pub level: VarInt,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct AttributeModifier {
    pub attribute_id: VarInt,
    pub modifier_id: String,
    pub value: f64,
    pub operation: AttributeModifierOperation,
    pub slot: AttributeModifierSlot,
}

#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum AttributeModifierOperation {
    Add,
    MultiplyBase,
    MultiplyTotal,
}

impl_discriminant_net_encode!(AttributeModifierOperation);
impl_discriminant_net_decode!(AttributeModifierOperation);
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum AttributeModifierSlot {
    Any,
    MainHand,
    OffHand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
}

impl_discriminant_net_encode!(AttributeModifierSlot);
impl_discriminant_net_decode!(AttributeModifierSlot);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct CustomModelData {
    pub floats: LengthPrefixedVec<f32>,
    pub flags: LengthPrefixedVec<bool>,
    pub strings: LengthPrefixedVec<String>,
    pub colors: LengthPrefixedVec<i32>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct TooltipDisplay {
    pub hide_tooltip: bool,
    pub hidden_components: LengthPrefixedVec<VarInt>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Food {
    pub nutrition: VarInt,
    pub saturation_modifier: f32,
    pub can_always_eat: bool,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Consumable {
    pub consume_seconds: f32,
    pub animation: ConsumeAnimation,
    pub sound: IdOr<SoundEvent>,
    pub has_consume_particles: bool,
    pub effects: LengthPrefixedVec<ConsumeEffect>,
}
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum ConsumeAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Spear,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
}

impl_discriminant_net_encode!(ConsumeAnimation);
impl_discriminant_net_decode!(ConsumeAnimation);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct UseCooldown {
    pub seconds: f32,
    pub cooldown_group: PrefixedOptional<String>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct UseEffects {
    pub can_sprint: bool,
    pub interact_vibrations: bool,
    pub speed_multiplier: f32,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct AttackRange {
    pub min_reach: f32,
    pub max_reach: f32,
    pub min_creative_reach: f32,
    pub max_creative_reach: f32,
    pub hitbox_margin: f32,
    pub mob_factor: f32,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Weapon {
    pub damage_per_attack: VarInt,
    pub disable_blocking_for: f32,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Tool {
    pub rules: LengthPrefixedVec<ToolRule>,
    pub default_mining_speed: f32,
    pub damage_per_block: VarInt,
    pub can_destroy_blocks_in_creative: bool,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct ToolRule {
    pub blocks: IDSet,
    pub speed: PrefixedOptional<f32>,
    pub correct_drop_for_blocks: PrefixedOptional<bool>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Equippable {
    pub slot: EquippableSlot,
    pub equip_sound: IdOr<SoundEvent>,
    pub model: PrefixedOptional<String>,
    pub camera_overlay: PrefixedOptional<String>,
    pub allowed_entities: PrefixedOptional<IDSet>,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub can_be_sheared: bool,
    pub shearing_sound: IdOr<SoundEvent>,
}
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum EquippableSlot {
    MainHand,
    Feet,
    Legs,
    Chest,
    Head,
    OffHand,
    Body,
}

impl_discriminant_net_encode!(EquippableSlot);
impl_discriminant_net_decode!(EquippableSlot);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BlocksAttacks {
    pub block_delay_seconds: f32,
    pub disable_cooldown_scale: f32,
    pub damage_reductions: LengthPrefixedVec<DamageReduction>,
    pub item_damage_threshold: f32,
    pub item_damage_base: f32,
    pub item_damage_factor: f32,
    pub bypassed_by: PrefixedOptional<IDSet>,
    pub block_sound: PrefixedOptional<IdOr<SoundEvent>>,
    pub disable_sound: PrefixedOptional<IdOr<SoundEvent>>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct DamageReduction {
    pub horizontal_blocking_angle: f32,
    pub r#type: PrefixedOptional<IDSet>,
    pub base: f32,
    pub factor: f32,
}
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum MapPostProcessing {
    Lock,
    Scale,
}

impl_discriminant_net_encode!(MapPostProcessing);
impl_discriminant_net_decode!(MapPostProcessing);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct SuspiciousStewEffect {
    pub type_id: VarInt,
    pub duration: VarInt,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct PotionContents {
    pub potion_id: PrefixedOptional<VarInt>,
    pub custom_color: PrefixedOptional<i32>,
    pub custom_effects: LengthPrefixedVec<PotionEffect>,
    pub custom_name: PrefixedOptional<String>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct WritableBookPage {
    pub raw_content: String,
    pub filtered_content: PrefixedOptional<String>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetDecode, TypeHash)]
pub struct WrittenBookContent {
    pub raw_title: String,
    pub filtered_title: PrefixedOptional<String>,
    pub author: String,
    pub generation: VarInt,
    pub pages: LengthPrefixedVec<WrittenBookPage>,
    pub resolved: bool,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetDecode, TypeHash)]
pub struct WrittenBookPage {
    pub raw_content: TextComponent,
    pub filtered_content: PrefixedOptional<TextComponent>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct EntityData {
    pub entity_type: VarInt,
    pub data: NbtBlob,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BlockEntityData {
    pub block_entity_type: VarInt,
    pub data: NbtBlob,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct PiercingWeapon {
    pub deals_knockback: bool,
    pub dismounts: bool,
    pub sound: PrefixedOptional<SoundEvent>,
    pub hit_sound: PrefixedOptional<SoundEvent>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct SwingAnimation {
    pub r#type: SwingAnimationType,
    pub duration: VarInt,
}
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum SwingAnimationType {
    None,
    Whack,
    Stab,
}

impl_discriminant_net_encode!(SwingAnimationType);
impl_discriminant_net_decode!(SwingAnimationType);
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

impl_discriminant_net_encode!(DyeColor);
impl_discriminant_net_decode!(DyeColor);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, TypeHash)]
pub struct LodestoneTracker {
    pub has_global_position: bool,
    pub dimension: Option<String>,
    pub position: Option<NetworkPosition>,
    pub tracked: bool,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct SoundEvent {
    pub sound_id: String,
    pub fixed_range: PrefixedOptional<f32>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BlockStateProperty {
    pub name: String,
    pub value: String,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Bee {
    pub entity_type: VarInt,
    pub entity_data: NbtBlob,
    pub ticks_in_hive: VarInt,
    pub min_ticks_in_hive: VarInt,
}

pub(crate) fn encode_text_component<W: Write>(
    text: &TextComponent,
    writer: &mut W,
    opts: &NetEncodeOpts,
) -> Result<(), NetEncodeError> {
    NBT::from(text.clone()).encode(writer, opts)
}

fn encode_text_components<W: Write>(
    text: &LengthPrefixedVec<TextComponent>,
    writer: &mut W,
) -> Result<(), NetEncodeError> {
    text.length.encode(writer, &NetEncodeOpts::None)?;
    for component in &text.data {
        encode_text_component(component, writer, &NetEncodeOpts::None)?;
    }
    Ok(())
}

fn encode_component_body<W: Write>(
    component: &ItemComponent,
    writer: &mut W,
) -> Result<(), NetEncodeError> {
    VarInt::new(component.protocol_id()).encode(writer, &NetEncodeOpts::None)?;
    encode_component_value(component, writer)
}

fn encode_component_value<W: Write>(
    component: &ItemComponent,
    writer: &mut W,
) -> Result<(), NetEncodeError> {
    match component {
        ItemComponent::CustomData(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MaxStackSize(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MaxDamage(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Damage(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Unbreakable => Ok(()),
        ItemComponent::CustomName(value) => {
            encode_text_component(value.as_ref(), writer, &NetEncodeOpts::None)
        }
        ItemComponent::ItemName(value) => {
            encode_text_component(value.as_ref(), writer, &NetEncodeOpts::None)
        }
        ItemComponent::ItemModel(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Lore(value) => encode_text_components(value, writer),
        ItemComponent::Rarity(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Enchantments(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CanPlaceOn(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CanBreak(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::AttributeModifiers(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CustomModelData(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::TooltipDisplay(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::RepairCost(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CreativeSlotLock => Ok(()),
        ItemComponent::EnchantmentGlintOverride(value) => {
            value.encode(writer, &NetEncodeOpts::None)
        }
        ItemComponent::IntangibleProjectile(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Food(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Consumable(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::UseRemainder(value) => {
            value.as_ref().encode_template(writer, &NetEncodeOpts::None)
        }
        ItemComponent::UseCooldown(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::UseEffects(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MinimumAttackCharge(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::DamageType(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::DamageResistant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::AttackRange(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Tool(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Weapon(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Enchantable(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Equippable(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Repairable(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Glider => Ok(()),
        ItemComponent::TooltipStyle(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::DeathProtection(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BlocksAttacks(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::StoredEnchantments(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::DyedColor(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MapColor(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MapId(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MapDecorations(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::MapPostProcessing(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ChargedProjectiles(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BundleContents(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PotionContents(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PotionDurationScale(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::SuspiciousStewEffects(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::WritableBookContent(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::WrittenBookContent(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Trim(value) => value.as_ref().encode(writer, &NetEncodeOpts::None),
        ItemComponent::DebugStickState(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::EntityData(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BucketEntityData(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BlockEntityData(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Instrument(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PiercingWeapon(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::KineticWeapon(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::SwingAnimation(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::AdditionalTradeCost(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Dye(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ProvidesTrimMaterial(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::OminousBottleAmplifier(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::JukeboxPlayable(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ProvidesBannerPatterns(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Recipes(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::LodestoneTracker(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::FireworkExplosion(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Fireworks(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Profile => unimplemented!(),
        ItemComponent::NoteBlockSound(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BannerPatterns(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BaseColor(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PotDecorations(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Container(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BlockState(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Bees(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::Lock(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ContainerLoot(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::BreakSound(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::SulfurCubeContent(value) => {
            value.as_ref().encode(writer, &NetEncodeOpts::None)
        }
        ItemComponent::VillagerVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::WolfVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::WolfSoundVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::WolfCollar(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::FoxVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::SalmonSize(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ParrotVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::TropicalFishPattern(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::TropicalFishBaseColor(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::TropicalFishPatternColor(value) => {
            value.encode(writer, &NetEncodeOpts::None)
        }
        ItemComponent::MooshroomVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::RabbitVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PigVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PigSoundVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CowVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CowSoundVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ChickenVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ChickenSoundVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::FrogVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::HorseVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::PaintingVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::LlamaVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::AxolotlVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ZombieNautilusVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CatVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CatSoundVariant(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::CatCollar(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::SheepColor(value) => value.encode(writer, &NetEncodeOpts::None),
        ItemComponent::ShulkerColor(value) => value.encode(writer, &NetEncodeOpts::None),
    }
}

impl NetEncode for ItemComponent {
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        match opts {
            NetEncodeOpts::None => encode_component_body(self, writer),
            NetEncodeOpts::WithLength => {
                let mut body = Vec::new();
                encode_component_body(self, &mut body)?;

                VarInt::new(body.len() as i32).encode(writer, &NetEncodeOpts::None)?;
                writer.write_all(&body)?;
                Ok(())
            }
            e => unimplemented!("Unsupported option for NetEncode: {:?}", e),
        }
    }
}

impl NetEncode for WrittenBookContent {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.raw_title.encode(writer, &NetEncodeOpts::None)?;
        self.filtered_title.encode(writer, &NetEncodeOpts::None)?;
        self.author.encode(writer, &NetEncodeOpts::None)?;
        self.generation.encode(writer, &NetEncodeOpts::None)?;
        self.pages.encode(writer, &NetEncodeOpts::None)?;
        self.resolved.encode(writer, &NetEncodeOpts::None)
    }
}

impl NetEncode for WrittenBookPage {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        encode_text_component(&self.raw_content, writer, &NetEncodeOpts::None)?;

        match &self.filtered_content {
            PrefixedOptional::None => false.encode(writer, &NetEncodeOpts::None),
            PrefixedOptional::Some(value) => {
                true.encode(writer, &NetEncodeOpts::None)?;
                encode_text_component(value, writer, &NetEncodeOpts::None)
            }
        }
    }
}

impl NetDecode for LodestoneTracker {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        let has_global_position = bool::decode(reader, opts)?;
        let dimension = if has_global_position {
            Some(String::decode(reader, opts)?)
        } else {
            None
        };
        let position = if has_global_position {
            Some(NetworkPosition::decode(reader, opts)?)
        } else {
            None
        };
        let tracked = bool::decode(reader, opts)?;

        Ok(Self {
            has_global_position,
            dimension,
            position,
            tracked,
        })
    }
}

impl NetDecode for ItemComponent {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        match opts {
            NetDecodeOpts::None => decode_component_body(reader),
            NetDecodeOpts::IsSizePrefixed => {
                let length = VarInt::decode(reader, &NetDecodeOpts::None)?.0;
                if length < 0 {
                    return Err(NetDecodeError::ExternalError(
                        format!("negative item component length: {length}").into(),
                    ));
                }

                let mut buf = vec![0; length as usize];
                reader.read_exact(&mut buf)?;
                decode_component_body(&mut Cursor::new(buf))
            }
        }
    }
}

fn decode_component_body<R: Read>(reader: &mut R) -> Result<ItemComponent, NetDecodeError> {
    decode_component_value(VarInt::decode(reader, &NetDecodeOpts::None)?.0, reader)
}

pub fn decode_component_value<R: Read>(
    component_id: i32,
    reader: &mut R,
) -> Result<ItemComponent, NetDecodeError> {
    match ItemComponentKind::from_protocol_id(component_id)
        .ok_or(NetDecodeError::InvalidEnumVariant)?
    {
        ItemComponentKind::CustomData => Ok(ItemComponent::CustomData(NbtBlob::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MaxStackSize => Ok(ItemComponent::MaxStackSize(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MaxDamage => Ok(ItemComponent::MaxDamage(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Damage => Ok(ItemComponent::Damage(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Unbreakable => Ok(ItemComponent::Unbreakable),
        ItemComponentKind::CustomName => Ok(ItemComponent::CustomName(decode_box(reader)?)),
        ItemComponentKind::ItemName => Ok(ItemComponent::ItemName(decode_box(reader)?)),
        ItemComponentKind::ItemModel => Ok(ItemComponent::ItemModel(String::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Lore => Ok(ItemComponent::Lore(LengthPrefixedVec::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Rarity => Ok(ItemComponent::Rarity(Rarity::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Enchantments => Ok(ItemComponent::Enchantments(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::CanPlaceOn => Ok(ItemComponent::CanPlaceOn(LengthPrefixedVec::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::CanBreak => Ok(ItemComponent::CanBreak(LengthPrefixedVec::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::AttributeModifiers => Ok(ItemComponent::AttributeModifiers(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::CustomModelData => Ok(ItemComponent::CustomModelData(
            CustomModelData::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::TooltipDisplay => Ok(ItemComponent::TooltipDisplay(
            TooltipDisplay::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::RepairCost => Ok(ItemComponent::RepairCost(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::CreativeSlotLock => Ok(ItemComponent::CreativeSlotLock),
        ItemComponentKind::EnchantmentGlintOverride => Ok(ItemComponent::EnchantmentGlintOverride(
            bool::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::IntangibleProjectile => Ok(ItemComponent::IntangibleProjectile(
            NbtBlob::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Food => Ok(ItemComponent::Food(Food::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Consumable => Ok(ItemComponent::Consumable(Consumable::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::UseRemainder => Ok(ItemComponent::UseRemainder(Box::new(
            InventorySlot::decode_template(reader, &NetDecodeOpts::None)?,
        ))),
        ItemComponentKind::UseCooldown => Ok(ItemComponent::UseCooldown(UseCooldown::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::UseEffects => Ok(ItemComponent::UseEffects(UseEffects::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MinimumAttackCharge => Ok(ItemComponent::MinimumAttackCharge(
            f32::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::DamageType => Ok(ItemComponent::DamageType(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::DamageResistant => Ok(ItemComponent::DamageResistant(IDSet::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::AttackRange => Ok(ItemComponent::AttackRange(AttackRange::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Tool => Ok(ItemComponent::Tool(Tool::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Weapon => Ok(ItemComponent::Weapon(Weapon::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Enchantable => Ok(ItemComponent::Enchantable(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Equippable => Ok(ItemComponent::Equippable(Equippable::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Repairable => Ok(ItemComponent::Repairable(IDSet::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Glider => Ok(ItemComponent::Glider),
        ItemComponentKind::TooltipStyle => Ok(ItemComponent::TooltipStyle(String::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::DeathProtection => Ok(ItemComponent::DeathProtection(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::BlocksAttacks => Ok(ItemComponent::BlocksAttacks(
            BlocksAttacks::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::StoredEnchantments => Ok(ItemComponent::StoredEnchantments(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::DyedColor => Ok(ItemComponent::DyedColor(i32::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MapColor => Ok(ItemComponent::MapColor(i32::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MapId => Ok(ItemComponent::MapId(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MapDecorations => Ok(ItemComponent::MapDecorations(NbtBlob::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::MapPostProcessing => Ok(ItemComponent::MapPostProcessing(
            MapPostProcessing::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::ChargedProjectiles => Ok(ItemComponent::ChargedProjectiles(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::BundleContents => Ok(ItemComponent::BundleContents(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::PotionContents => Ok(ItemComponent::PotionContents(
            PotionContents::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::PotionDurationScale => Ok(ItemComponent::PotionDurationScale(
            f32::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::SuspiciousStewEffects => Ok(ItemComponent::SuspiciousStewEffects(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::WritableBookContent => Ok(ItemComponent::WritableBookContent(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::WrittenBookContent => Ok(ItemComponent::WrittenBookContent(
            WrittenBookContent::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Trim => Ok(ItemComponent::Trim(Box::new(Trim {
            material: IdOr::decode(reader, &NetDecodeOpts::None)?,
            pattern: IdOr::decode(reader, &NetDecodeOpts::None)?,
        }))),
        ItemComponentKind::DebugStickState => Ok(ItemComponent::DebugStickState(NbtBlob::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::EntityData => Ok(ItemComponent::EntityData(EntityData::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::BucketEntityData => Ok(ItemComponent::BucketEntityData(
            NbtBlob::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::BlockEntityData => Ok(ItemComponent::BlockEntityData(
            BlockEntityData::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Instrument => Ok(ItemComponent::Instrument(IdOr::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::PiercingWeapon => Ok(ItemComponent::PiercingWeapon(
            PiercingWeapon::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::KineticWeapon => Ok(ItemComponent::KineticWeapon(
            KineticWeapon::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::SwingAnimation => Ok(ItemComponent::SwingAnimation(
            SwingAnimation::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::AdditionalTradeCost => Ok(ItemComponent::AdditionalTradeCost(
            VarInt::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Dye => Ok(ItemComponent::Dye(DyeColor::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ProvidesTrimMaterial => Ok(ItemComponent::ProvidesTrimMaterial(
            IdOr::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::OminousBottleAmplifier => Ok(ItemComponent::OminousBottleAmplifier(
            VarInt::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::JukeboxPlayable => Ok(ItemComponent::JukeboxPlayable(IdOr::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ProvidesBannerPatterns => Ok(ItemComponent::ProvidesBannerPatterns(
            IDSet::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Recipes => Ok(ItemComponent::Recipes(NbtBlob::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::LodestoneTracker => Ok(ItemComponent::LodestoneTracker(
            LodestoneTracker::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::FireworkExplosion => Ok(ItemComponent::FireworkExplosion(
            FireworkExplosion::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Fireworks => Ok(ItemComponent::Fireworks(Fireworks::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Profile => unsupported_component("profile"),
        ItemComponentKind::NoteBlockSound => Ok(ItemComponent::NoteBlockSound(String::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::BannerPatterns => Ok(ItemComponent::BannerPatterns(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::BaseColor => Ok(ItemComponent::BaseColor(DyeColor::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::PotDecorations => Ok(ItemComponent::PotDecorations(
            LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::Container => Ok(ItemComponent::Container(LengthPrefixedVec::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::BlockState => Ok(ItemComponent::BlockState(LengthPrefixedVec::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Bees => Ok(ItemComponent::Bees(LengthPrefixedVec::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::Lock => Ok(ItemComponent::Lock(NbtBlob::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ContainerLoot => Ok(ItemComponent::ContainerLoot(NbtBlob::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::BreakSound => Ok(ItemComponent::BreakSound(IdOr::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::SulfurCubeContent => {
            Ok(ItemComponent::SulfurCubeContent(decode_box(reader)?))
        }
        ItemComponentKind::VillagerVariant => Ok(ItemComponent::VillagerVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::WolfVariant => Ok(ItemComponent::WolfVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::WolfSoundVariant => Ok(ItemComponent::WolfSoundVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::WolfCollar => Ok(ItemComponent::WolfCollar(DyeColor::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::FoxVariant => Ok(ItemComponent::FoxVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::SalmonSize => Ok(ItemComponent::SalmonSize(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ParrotVariant => Ok(ItemComponent::ParrotVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::TropicalFishPattern => Ok(ItemComponent::TropicalFishPattern(
            VarInt::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::TropicalFishBaseColor => Ok(ItemComponent::TropicalFishBaseColor(
            DyeColor::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::TropicalFishPatternColor => Ok(ItemComponent::TropicalFishPatternColor(
            DyeColor::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::MooshroomVariant => Ok(ItemComponent::MooshroomVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::RabbitVariant => Ok(ItemComponent::RabbitVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::PigVariant => Ok(ItemComponent::PigVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::PigSoundVariant => Ok(ItemComponent::PigSoundVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::CowVariant => Ok(ItemComponent::CowVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::CowSoundVariant => Ok(ItemComponent::CowSoundVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ChickenVariant => Ok(ItemComponent::ChickenVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ChickenSoundVariant => Ok(ItemComponent::ChickenSoundVariant(
            VarInt::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::FrogVariant => Ok(ItemComponent::FrogVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::HorseVariant => Ok(ItemComponent::HorseVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::PaintingVariant => Ok(ItemComponent::PaintingVariant(IdOr::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::LlamaVariant => Ok(ItemComponent::LlamaVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::AxolotlVariant => Ok(ItemComponent::AxolotlVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ZombieNautilusVariant => Ok(ItemComponent::ZombieNautilusVariant(
            VarInt::decode(reader, &NetDecodeOpts::None)?,
        )),
        ItemComponentKind::CatVariant => Ok(ItemComponent::CatVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::CatSoundVariant => Ok(ItemComponent::CatSoundVariant(VarInt::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::CatCollar => Ok(ItemComponent::CatCollar(DyeColor::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::SheepColor => Ok(ItemComponent::SheepColor(DyeColor::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
        ItemComponentKind::ShulkerColor => Ok(ItemComponent::ShulkerColor(DyeColor::decode(
            reader,
            &NetDecodeOpts::None,
        )?)),
    }
}

fn decode_box<T, R>(reader: &mut R) -> Result<Box<T>, NetDecodeError>
where
    T: NetDecode,
    R: Read,
{
    Ok(Box::new(T::decode(reader, &NetDecodeOpts::None)?))
}

fn unsupported_component<T>(component_name: &str) -> Result<T, NetDecodeError> {
    Err(NetDecodeError::ExternalError(
        format!("decoding item component {component_name} is not implemented yet").into(),
    ))
}
