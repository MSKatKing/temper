use crate::components::potion_effect::PotionEffect;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use temper_codec::decode::errors::NetDecodeError;
use temper_codec::decode::{NetDecode, NetDecodeOpts};
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_codec::net_types::var_int::VarInt;
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TypeHash)]
pub struct ConsumeEffect {
    pub type_id: VarInt,
    pub data: ConsumeEffectData,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TypeHash)]
pub enum ConsumeEffectData {
    ApplyEffects {
        effects: LengthPrefixedVec<PotionEffect>,
        probability: f32,
    },
}

impl NetEncode for ConsumeEffectData {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        match self {
            Self::ApplyEffects {
                effects,
                probability,
            } => {
                effects.encode(writer, &NetEncodeOpts::None)?;
                probability.encode(writer, &NetEncodeOpts::None)
            }
        }
    }
}

impl NetEncode for ConsumeEffect {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.type_id.encode(writer, &NetEncodeOpts::None)?;
        self.data.encode(writer, &NetEncodeOpts::None)
    }
}

impl NetDecode for ConsumeEffect {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        let type_id = VarInt::decode(reader, opts)?;
        let data = ConsumeEffectData::decode_for_type(type_id.0, reader)?;

        Ok(Self { type_id, data })
    }
}

impl ConsumeEffectData {
    fn decode_for_type<R: Read>(type_id: i32, reader: &mut R) -> Result<Self, NetDecodeError> {
        match type_id {
            0 => Ok(Self::ApplyEffects {
                effects: LengthPrefixedVec::decode(reader, &NetDecodeOpts::None)?,
                probability: f32::decode(reader, &NetDecodeOpts::None)?,
            }),
            _ => Err(NetDecodeError::InvalidEnumVariant),
        }
    }
}
