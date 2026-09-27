use serde::{Deserialize, Serialize};
use std::io::Write;
use temper_codec::decode::errors::NetDecodeError;
use temper_codec::decode::{NetDecode, NetDecodeOpts};
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_codec::net_types::var_int::VarInt;
use temper_macros::{Discriminant, InverseDiscriminant, NetDecode, NetEncode};
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Fireworks {
    pub flight_duration: VarInt,
    pub explosions: LengthPrefixedVec<FireworkExplosion>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct FireworkExplosion {
    pub shape: FireworkExplosionShape,
    pub colors: LengthPrefixedVec<i32>,
    pub fade_colors: LengthPrefixedVec<i32>,
    pub has_trail: bool,
    pub has_twinkle: bool,
}
#[derive(
    PartialEq, Debug, Clone, Serialize, Deserialize, Discriminant, InverseDiscriminant, TypeHash,
)]
pub enum FireworkExplosionShape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

impl NetEncode for FireworkExplosionShape {
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        VarInt::new(self.discriminant()).encode(writer, opts)
    }
}

impl NetDecode for FireworkExplosionShape {
    fn decode<R: std::io::Read>(
        reader: &mut R,
        opts: &NetDecodeOpts,
    ) -> Result<Self, NetDecodeError> {
        Self::from_discriminant(VarInt::decode(reader, opts)?)
            .ok_or(NetDecodeError::InvalidEnumVariant)
    }
}
