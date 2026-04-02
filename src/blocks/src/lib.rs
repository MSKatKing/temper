use bevy_math::DVec2;
use temper_block_properties::SlabType;
use temper_blocks_generated::{SlabBlock, SnowyBlock};
use temper_core::block_face::BlockFace;
use temper_core::block_state_id::BlockStateId;
use temper_core::dimension::Dimension;
use temper_core::pos::BlockPos;
use temper_macros::match_block;
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

pub const BLOCK_MAPPINGS: &[StateBehaviorTable] =
    include!(concat!(env!("OUT_DIR"), "/mappings.rs"));

pub struct PlacementContext {
    pub face: BlockFace,
    pub cursor: DVec2,
}

impl BlockBehavior for SlabBlock {
    #[inline(always)]
    fn get_placement_state(&mut self, context: PlacementContext, world: &World, pos: BlockPos) {
        let block = world
            .get_chunk(pos.chunk(), Dimension::Overworld)
            .map(|c| c.get_block(pos.chunk_block_pos()))
            .unwrap_or(BlockStateId::new(0));

        self.waterlogged = match_block!("water", block);
        self.ty = match context.face {
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
    }

    #[inline(always)]
    fn test(&self) {
        panic!("hello")
    }
}

fn has_snow_above(world: &World, pos: BlockPos) -> bool {
    let pos = pos + (0, 1, 0);

    world
        .get_chunk(pos.chunk(), Dimension::Overworld)
        .map(|c| c.get_block(pos.chunk_block_pos()))
        .is_ok_and(|id| match_block!("snow", id))
}

impl BlockBehavior for SnowyBlock {
    fn get_placement_state(&mut self, _context: PlacementContext, world: &World, pos: BlockPos) {
        self.snowy = has_snow_above(world, pos);
    }

    fn update(&mut self, world: &World, pos: BlockPos) {
        self.snowy = has_snow_above(world, pos);
    }
}

#[cfg(test)]
mod tests {
    use crate::BLOCK_MAPPINGS;

    #[test]
    #[ignore]
    fn test() {
        BLOCK_MAPPINGS[12051].test();
    }
}
