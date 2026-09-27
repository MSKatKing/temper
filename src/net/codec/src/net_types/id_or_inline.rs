use crate::decode::errors::NetDecodeError;
use crate::decode::{NetDecode, NetDecodeOpts};
use crate::encode::errors::NetEncodeError;
use crate::encode::{NetEncode, NetEncodeOpts};
use crate::net_types::var_int::VarInt;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use type_hash::TypeHash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeHash)]
pub enum IdOr<T> {
    Id(VarInt),
    Inline(T),
}

impl<T: NetEncode> NetEncode for IdOr<T> {
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        match self {
            Self::Id(id) => {
                let encoded_id = id.0.checked_add(1).filter(|id| *id > 0).ok_or_else(|| {
                    NetEncodeError::ExternalError(
                        format!("invalid registry ID for ID-or value: {id}").into(),
                    )
                })?;

                VarInt::new(encoded_id).encode(writer, opts)
            }
            Self::Inline(value) => {
                VarInt::new(0).encode(writer, opts)?;
                value.encode(writer, opts)
            }
        }
    }
}

impl<T: NetDecode> NetDecode for IdOr<T> {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        let id = VarInt::decode(reader, opts)?.0;

        match id {
            0 => Ok(Self::Inline(T::decode(reader, opts)?)),
            1.. => Ok(Self::Id(VarInt::new(id - 1))),
            _ => Err(NetDecodeError::ExternalError(
                format!("invalid ID-or discriminator: {id}").into(),
            )),
        }
    }
}

impl<T> IdOr<T> {
    pub fn id(id: VarInt) -> Self {
        Self::Id(id)
    }

    pub fn inline(value: T) -> Self {
        Self::Inline(value)
    }

    pub fn is_id(&self) -> bool {
        matches!(self, Self::Id(_))
    }

    pub fn is_inline(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    pub fn as_id(&self) -> Option<&VarInt> {
        match self {
            Self::Id(id) => Some(id),
            Self::Inline(_) => None,
        }
    }

    pub fn as_inline(&self) -> Option<&T> {
        match self {
            Self::Id(_) => None,
            Self::Inline(value) => Some(value),
        }
    }
}

impl<T> From<T> for IdOr<T> {
    fn from(value: T) -> Self {
        Self::Inline(value)
    }
}
