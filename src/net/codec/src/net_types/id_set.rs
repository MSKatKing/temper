use crate::decode::errors::NetDecodeError;
use crate::decode::{NetDecode, NetDecodeOpts};
use crate::encode::errors::NetEncodeError;
use crate::encode::{NetEncode, NetEncodeOpts};
use crate::net_types::var_int::VarInt;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use type_hash::TypeHash;

#[derive(Hash, PartialEq, Debug, Clone, Serialize, Deserialize, TypeHash)]
pub enum IDSet {
    Indirect(String),
    Direct(Vec<VarInt>),
}

impl NetEncode for IDSet {
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        match self {
            IDSet::Indirect(tag_name) => {
                VarInt::new(0).encode(writer, opts)?;
                tag_name.encode(writer, opts)
            }
            IDSet::Direct(tags) => {
                VarInt::new((tags.len() - 1) as i32).encode(writer, opts)?;
                for tag in tags {
                    tag.encode(writer, opts)?;
                }
                Ok(())
            }
        }
    }
}

impl NetDecode for IDSet {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        let length = VarInt::decode(reader, opts)?.0;

        match length {
            0 => Ok(Self::Indirect(String::decode(reader, opts)?)),
            1.. => {
                let mut ids = Vec::with_capacity(length as usize + 1);
                for _ in 0..=length {
                    ids.push(VarInt::decode(reader, opts)?);
                }

                Ok(Self::Direct(ids))
            }
            _ => Err(NetDecodeError::ExternalError(
                format!("invalid ID set length: {length}").into(),
            )),
        }
    }
}
