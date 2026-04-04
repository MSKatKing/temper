use crate::{BlockBehavior, BlockDispatch, PlacementContext};
use temper_block_properties::SlabType;
use temper_blocks_generated::SlabBlock;
use temper_core::block_face::BlockFace;
use temper_core::block_state_id::BlockStateId;
use temper_macros::match_block;

impl BlockBehavior for SlabBlock {
    #[inline(always)]
    fn get_placement_state(&mut self, context: PlacementContext) {
        let block = context
            .level
            .get_chunk(context.block_pos.chunk(), context.dimension)
            .map(|c| c.get_block(context.block_pos.chunk_block_pos()))
            .unwrap_or(BlockStateId::new(0));

        self.ty = if block.try_cast::<SlabBlock>().is_some() {
            SlabType::Double
        } else {
            match context.face {
                BlockFace::Top => SlabType::Bottom,
                BlockFace::Bottom => SlabType::Top,
                _ => {
                    if context.cursor.y > 0.5 {
                        SlabType::Top
                    } else {
                        SlabType::Bottom
                    }
                }
            }
        };

        self.waterlogged = match_block!("water", block);
    }

    #[inline(always)]
    fn can_be_replaced(&self, _context: PlacementContext) -> bool {
        true
    }
}
