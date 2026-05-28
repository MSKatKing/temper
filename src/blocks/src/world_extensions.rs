use temper_core::block_state_id::BlockStateId;
use temper_core::dimension::Dimension;
use temper_core::pos::BlockPos;
use temper_world::{World, WorldError};
use crate::BlockBehavior;

// TODO: add dimensions to all these functions

/// Trait to add useful functions to the World struct
pub trait WorldBlockUpdates {
    fn get_block_or_default(&self, block_pos: BlockPos) -> BlockStateId;
    
    fn update_block(&self, block_pos: BlockPos, callback: impl Fn(BlockStateId) -> BlockStateId) -> Result<BlockStateId, WorldError>;
    fn update_block_cast<T: BlockBehavior>(&self, block_pos: BlockPos, callback: impl Fn(&mut T)) -> Result<BlockStateId, WorldError>;

    fn block_is<T: BlockBehavior>(&self, block_pos: BlockPos) -> bool;
}

impl WorldBlockUpdates for World {
    fn get_block_or_default(&self, block_pos: BlockPos) -> BlockStateId {
        self
            .get_chunk(block_pos.chunk(), Dimension::Overworld)
            .map(|chunk| chunk.get_block(block_pos.chunk_block_pos()))
            .unwrap_or_default()
    }

    fn update_block(&self, block_pos: BlockPos, callback: impl Fn(BlockStateId) -> BlockStateId) -> Result<BlockStateId, WorldError> {
        self
            .get_chunk(block_pos.chunk(), Dimension::Overworld)
            .map(|chunk| {
                callback(chunk.get_block(block_pos.chunk_block_pos()))
            })
    }

    fn update_block_cast<T: BlockBehavior>(&self, block_pos: BlockPos, callback: impl Fn(&mut T)) -> Result<BlockStateId, WorldError> {
        self
            .get_chunk(block_pos.chunk(), Dimension::Overworld)
            .and_then(|chunk| {
                let id = chunk.get_block(block_pos.chunk_block_pos());
                let mut id = T::try_from(chunk.get_block(block_pos.chunk_block_pos()).raw()).map_err(|_| WorldError::InvalidBlock(id))?;
                callback(&mut id);
                
                id.try_into().map(BlockStateId::new).map_err(|_| WorldError::InvalidBlock(BlockStateId::default()))
            })
    }

    fn block_is<T: BlockBehavior>(&self, block_pos: BlockPos) -> bool {
        self
            .get_chunk(block_pos.chunk(), Dimension::Overworld)
            .map(|chunk| T::try_from(chunk.get_block(block_pos.chunk_block_pos()).raw()).is_ok())
            .unwrap_or_default()
    }
}