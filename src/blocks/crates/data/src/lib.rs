use std::collections::HashMap;
use temper_core::block_state_id::BlockStateId;
use temper_core::pos::BlockPos;

#[derive(Default)]
pub struct PlacedBlocks {
    pub blocks: HashMap<BlockPos, BlockStateId>,
    pub take_item: bool,
}