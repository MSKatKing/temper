use std::collections::HashMap;

use bevy_math::I8Vec3;
use temper_core::block_state_id::BlockStateId;
use temper_macros::block;

pub(crate) fn generate_tree() -> HashMap<I8Vec3, BlockStateId> {
    let mut blocks = HashMap::new();

    // Generate a simple tree structure
    let trunk_height = 5;
    let leaf_radius = 2;

    // Generate trunk
    for y in 0..trunk_height {
        blocks.insert(
            I8Vec3::new(0, y, 0),
            block!("minecraft:oak_log", {axis: "y"}).into(),
        );
    }

    // Generate leaves
    for x in -leaf_radius..=leaf_radius {
        for y in trunk_height..(trunk_height + leaf_radius) {
            for z in -leaf_radius..=leaf_radius {
                if x * x + z * z <= leaf_radius * leaf_radius {
                    blocks.insert(
                        I8Vec3::new(x, y, z),
                        block!("minecraft:oak_leaves", {distance: 1, persistent: true, waterlogged: false}).into(),
                    );
                }
            }
        }
    }

    blocks
}
