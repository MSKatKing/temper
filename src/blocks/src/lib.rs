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
mod world_extensions;

#[allow(unused_imports)] // Used in the include!
use crate::behavior_trait::BlockBehaviorTable;

pub use crate::behavior_trait::{BlockBehavior, BlockDispatch, StateBehaviorTable};
pub use temper_block_data::*;
use temper_core::block_state_id::BlockStateId;
use temper_core::dimension::Dimension;
use temper_core::pos::BlockPos;
use temper_world::World;

pub const BLOCK_MAPPINGS: &[StateBehaviorTable] =
    include!(concat!(env!("OUT_DIR"), "/mappings.rs"));

pub(crate) fn get_block(world: &World, block_pos: BlockPos, dimension: Dimension, default: BlockStateId) -> BlockStateId {
    world
        .get_chunk(block_pos.chunk(), dimension)
        .map(|chunk| chunk.get_block(block_pos.chunk_block_pos()))
        .unwrap_or(default)
}