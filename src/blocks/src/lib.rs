use bevy_math::DVec3;
use temper_core::block_face::BlockFace;
use temper_core::dimension::Dimension;
use temper_core::pos::BlockPos;
use temper_world::World;

mod bed_block;
mod behavior_trait;
mod bubble_column;
mod building;
mod cake;
mod candle_cake;
mod decorative;
mod facing_block;
mod fire;
mod functional;
mod liquid;
mod nature;
mod redstone;
mod skull;
mod suspicious_block;
mod wall_skull;
mod waterloggable_block;

#[allow(unused_imports)] // Used in the include!
use crate::behavior_trait::BlockBehaviorTable;

pub use crate::behavior_trait::{BlockBehavior, BlockDispatch, StateBehaviorTable};
pub use temper_block_data::*;

pub const BLOCK_MAPPINGS: &[StateBehaviorTable] =
    include!(concat!(env!("OUT_DIR"), "/mappings.rs"));

pub struct PlacementContext<'a> {
    pub face: BlockFace,
    pub cursor: DVec3,
    pub block_clicked: BlockPos,
    pub block_pos: BlockPos,
    pub level: &'a World,
    pub dimension: Dimension,
}
