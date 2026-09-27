use super::ItemComponent;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use temper_codec::decode::errors::NetDecodeError;
use temper_codec::decode::{NetDecode, NetDecodeOpts};
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::id_set::IDSet;
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_codec::net_types::prefixed_optional::PrefixedOptional;
use temper_codec::net_types::var_int::VarInt;
use temper_macros::{Discriminant, InverseDiscriminant, NetDecode, NetEncode};
use temper_nbt::blob::NbtBlob;
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BlockPredicate {
    pub blocks: PrefixedOptional<IDSet>,
    pub properties: PrefixedOptional<LengthPrefixedVec<BlockPredicateProperty>>,
    pub nbt: PrefixedOptional<NbtBlob>,
    pub data_components: LengthPrefixedVec<ExactDataComponentMatcher>,
    pub partial_data_component_predicates: LengthPrefixedVec<PartialDataComponentMatcher>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BlockPredicateProperty {
    pub name: String,
    pub matcher: BlockPredicatePropertyMatcher,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TypeHash)]
pub enum BlockPredicatePropertyMatcher {
    Exact(String),
    Range {
        min_value: Option<String>,
        max_value: Option<String>,
    },
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TypeHash)]
pub struct ExactDataComponentMatcher {
    #[type_hash(foreign_type)]
    pub component: Box<ItemComponent>,
}

impl NetEncode for ExactDataComponentMatcher {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.component.as_ref().encode(writer, &NetEncodeOpts::None)
    }
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct PartialDataComponentMatcher {
    pub predicate_type: PartialDataComponentPredicateType,
    pub predicate: NbtBlob,
}
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum PartialDataComponentPredicateType {
    Damage,
    Enchantments,
    StoredEnchantments,
    PotionContents,
    CustomData,
    Container,
    BundleContents,
    FireworkExplosion,
    Fireworks,
    WritableBookContent,
    WrittenBookContent,
    AttributeModifiers,
    Trim,
    JukeboxPlayable,
}

impl NetEncode for BlockPredicatePropertyMatcher {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        match self {
            Self::Exact(value) => {
                true.encode(writer, &NetEncodeOpts::None)?;
                value.encode(writer, &NetEncodeOpts::None)
            }
            Self::Range {
                min_value,
                max_value,
            } => {
                false.encode(writer, &NetEncodeOpts::None)?;
                encode_optional_string(min_value, writer)?;
                encode_optional_string(max_value, writer)
            }
        }
    }
}

impl NetDecode for BlockPredicatePropertyMatcher {
    fn decode<R: Read>(reader: &mut R, _opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        if bool::decode(reader, &NetDecodeOpts::None)? {
            return Ok(Self::Exact(String::decode(reader, &NetDecodeOpts::None)?));
        }

        Ok(Self::Range {
            min_value: decode_optional_string(reader)?,
            max_value: decode_optional_string(reader)?,
        })
    }
}

impl NetEncode for PartialDataComponentPredicateType {
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        VarInt::new(self.discriminant()).encode(writer, opts)
    }
}

impl NetDecode for PartialDataComponentPredicateType {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        Self::from_discriminant(VarInt::decode(reader, opts)?)
            .ok_or(NetDecodeError::InvalidEnumVariant)
    }
}

impl NetDecode for ExactDataComponentMatcher {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        Ok(Self {
            component: Box::new(ItemComponent::decode(reader, opts)?),
        })
    }
}

fn encode_optional_string<W: Write>(
    value: &Option<String>,
    writer: &mut W,
) -> Result<(), NetEncodeError> {
    match value {
        Some(value) => {
            true.encode(writer, &NetEncodeOpts::None)?;
            value.encode(writer, &NetEncodeOpts::None)
        }
        None => false.encode(writer, &NetEncodeOpts::None),
    }
}

fn decode_optional_string<R: Read>(reader: &mut R) -> Result<Option<String>, NetDecodeError> {
    if bool::decode(reader, &NetDecodeOpts::None)? {
        Ok(Some(String::decode(reader, &NetDecodeOpts::None)?))
    } else {
        Ok(None)
    }
}
