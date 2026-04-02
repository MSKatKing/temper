mod blocks;
mod errors;

use crate::errors::BlockPlaceError;
use bevy_math::DVec3;
use std::collections::HashMap;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_core::block_state_id::{
    BlockStateId, ITEM_TO_BLOCK_MAPPING, create_item_to_block_mapping,
};
use temper_core::dimension::Dimension;
use temper_core::pos::BlockPos;
use temper_inventories::item::ItemID;
use temper_state::GlobalState;

pub struct PlacedBlocks {
    pub blocks: HashMap<BlockPos, BlockStateId>,
    pub take_item: bool,
}

pub trait PlacableBlock {
    fn place(
        context: BlockPlaceContext,
        state: GlobalState,
    ) -> Result<PlacedBlocks, BlockPlaceError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    West,
    East,
}

pub struct BlockPlaceContext {
    pub block_clicked: BlockStateId,
    pub block_position: BlockPos,
    pub face_clicked: BlockFace,
    pub click_position: DVec3,
    pub player_position: Position,
    pub player_rotation: Rotation,
    pub item_used: ItemID,
}

pub fn place_item(
    state: GlobalState,
    context: BlockPlaceContext,
) -> Result<PlacedBlocks, BlockPlaceError> {
    let Some(item_name) = context.item_used.to_name() else {
        return Err(BlockPlaceError::ItemIdHasNoNameMapping(context.item_used));
    };
    let item_name = item_name.strip_prefix("minecraft:").unwrap_or(&item_name);
    if item_name == "torch" {
        blocks::torch::PlaceableTorch::place(context, state)
    } else if item_name.ends_with("_slab") {
        blocks::slab::PlaceableSlab::place(context, state)
    } else if item_name.ends_with("_door") {
        blocks::door::PlaceableDoor::place(context, state)
    } else if item_name.ends_with("_log") {
        blocks::logs::PlacableLog::place(context, state)
    } else if item_name.ends_with("_fence") {
        blocks::fence::PlaceableFence::place(context, state)
    } else {
        let block_opt = ITEM_TO_BLOCK_MAPPING
            .get_or_init(create_item_to_block_mapping)
            .get(&context.item_used.0.0);
        if let Some(block) = block_opt {
            match state
                .world
                .get_or_generate_mut(context.block_position.chunk(), Dimension::Overworld)
            {
                Ok(mut chunk) => {
                    chunk.set_block(context.block_position.chunk_block_pos(), *block);
                    Ok(PlacedBlocks {
                        blocks: HashMap::from([(context.block_position, *block)]),
                        take_item: true,
                    })
                }
                Err(e) => Err(e.into()),
            }
        } else {
            Err(BlockPlaceError::ItemNotPlaceable(context.item_used))
        }
    }
}
