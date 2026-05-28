use bevy_math::DVec3;
use std::collections::HashMap;
use temper_components::player::rotation::Rotation;
use temper_core::block_face::BlockFace;
use temper_core::block_state_id::BlockStateId;
use temper_core::dimension::Dimension;
use temper_core::pos::BlockPos;
use temper_world::World;

pub struct PlacementContext<'a> {
    pub face: BlockFace,
    pub cursor: DVec3,
    pub block_clicked: BlockPos,
    pub block_pos: BlockPos,
    pub level: &'a World,
    pub dimension: Dimension,
    pub player_rotation: &'a Rotation,
}

/// Result of the get_placement_state function
pub struct PlacedBlocks {
    /// Any extra blocks placed by the function
    pub blocks: HashMap<BlockPos, BlockStateId>,

    /// Whether an item is taken from the player's inventory or not
    pub take_item: bool,
}

impl Default for PlacedBlocks {
    fn default() -> Self {
        Self {
            blocks: HashMap::default(),
            take_item: true,
        }
    }
}
