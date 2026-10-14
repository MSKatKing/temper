use serde::{Deserialize, Serialize};
use type_hash::TypeHash;
use temper_codec::net_types::var_int::VarInt;

/// Represents a dimension in the world. The first three variants are the standard Minecraft
/// dimensions, and the `Custom` variant can be used for modded dimensions or other custom implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TypeHash)]
pub enum Dimension {
    #[default]
    Overworld,
    Nether,
    End,
    Custom(u16),
}

impl Dimension {
    pub fn as_var_int(&self) -> VarInt {
        match self {
            Self::Overworld => VarInt::from(0),
            Self::Nether => VarInt::from(1),
            Self::End => VarInt::from(2),
            Self::Custom(v) => VarInt::from(*v as i32 + 2),
        }
    }

    pub fn as_identifier(&self) -> &'static str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::Nether => "minecraft:nether",
            Self::End => "minecraft:the_end",
            _ => "temper:custom_dimension",
        }
    }
}
