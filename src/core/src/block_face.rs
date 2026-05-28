use bevy_math::IVec3;
use std::io::Read;
use temper_codec::decode::errors::NetDecodeError;
use temper_codec::decode::{NetDecode, NetDecodeOpts};
use temper_codec::net_types::var_int::VarInt;

#[derive(Debug, Clone)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl BlockFace {
    pub fn is_x_axis(&self) -> bool {
        matches!(self, BlockFace::East | BlockFace::West)
    }

    pub fn is_y_axis(&self) -> bool {
        matches!(self, BlockFace::Top | BlockFace::Bottom)
    }

    pub fn is_z_axis(&self) -> bool {
        matches!(self, BlockFace::North | BlockFace::South)
    }

    /// Returns the translation vector that will get the block that touches this face.
    pub fn get_normal(&self) -> IVec3 {
        match self {
            BlockFace::Top => IVec3::new(0, 1, 0),
            BlockFace::Bottom => IVec3::new(0, -1, 0),
            BlockFace::North => IVec3::new(0, 0, -1),
            BlockFace::South => IVec3::new(0, 0, 1),
            BlockFace::East => IVec3::new(1, 0, 0),
            BlockFace::West => IVec3::new(-1, 0, 0),
        }
    }
}

impl TryFrom<u32> for BlockFace {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BlockFace::Bottom),
            1 => Ok(BlockFace::Top),
            2 => Ok(BlockFace::North),
            3 => Ok(BlockFace::South),
            4 => Ok(BlockFace::West),
            5 => Ok(BlockFace::East),
            _ => Err(()),
        }
    }
}

impl NetDecode for BlockFace {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        let VarInt(data) = VarInt::decode(reader, opts)?;

        BlockFace::try_from(data as u32).map_err(|_| NetDecodeError::InvalidEnumVariant)
    }
}
