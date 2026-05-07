use crate::errors::BlockPlaceError;
use crate::{BlockPlaceContext, PlacableBlock, PlacedBlocks};
use bevy_math::DVec3;
use temper_state::GlobalState;

pub(crate) struct PlaceableSlab;

impl PlacableBlock for PlaceableSlab {
    fn place(
        context: BlockPlaceContext,
        state: GlobalState,
    ) -> Result<PlacedBlocks, BlockPlaceError> {
        fn get_block_data_at(
            pos: &temper_core::pos::BlockPos,
            state: &GlobalState,
        ) -> temper_core::block_data::BlockData {
            let chunk = state
                .world
                .get_or_generate_chunk(pos.chunk(), temper_core::dimension::Dimension::Overworld)
                .expect("Could not load chunk");
            let block = chunk.get_block(pos.chunk_block_pos());
            temper_core::block_data::BlockData::from_block_state_id(block)
        }

        fn is_same_slab_block(data: &temper_core::block_data::BlockData, block_name: &str) -> bool {
            data.name == block_name
                && data
                    .properties
                    .as_ref()
                    .and_then(|p| p.get("type"))
                    .map(|t| t != "double")
                    .unwrap_or(false)
        }

        fn get_clicked_pos(
            block_position: &temper_core::pos::BlockPos,
            face: &crate::BlockFace,
        ) -> temper_core::pos::BlockPos {
            match face {
                crate::BlockFace::Top => temper_core::pos::BlockPos::of(
                    block_position.pos.x,
                    block_position.pos.y - 1,
                    block_position.pos.z,
                ),
                crate::BlockFace::Bottom => temper_core::pos::BlockPos::of(
                    block_position.pos.x,
                    block_position.pos.y + 1,
                    block_position.pos.z,
                ),
                crate::BlockFace::North => temper_core::pos::BlockPos::of(
                    block_position.pos.x,
                    block_position.pos.y,
                    block_position.pos.z + 1,
                ),
                crate::BlockFace::South => temper_core::pos::BlockPos::of(
                    block_position.pos.x,
                    block_position.pos.y,
                    block_position.pos.z - 1,
                ),
                crate::BlockFace::East => temper_core::pos::BlockPos::of(
                    block_position.pos.x - 1,
                    block_position.pos.y,
                    block_position.pos.z,
                ),
                crate::BlockFace::West => temper_core::pos::BlockPos::of(
                    block_position.pos.x + 1,
                    block_position.pos.y,
                    block_position.pos.z,
                ),
            }
        }

        fn get_half(
            face: &crate::BlockFace,
            click_position: &DVec3,
            block_position: &temper_core::pos::BlockPos,
        ) -> &'static str {
            match face {
                crate::BlockFace::Top => "bottom",
                crate::BlockFace::Bottom => "top",
                _ => {
                    if click_position.y - f64::from(block_position.pos.y) > 0.5 {
                        "top"
                    } else {
                        "bottom"
                    }
                }
            }
        }

        let block_name = match context.item_used.to_name() {
            Some(name) => name,
            None => return Err(BlockPlaceError::ItemIdHasNoNameMapping(context.item_used)),
        };

        let clicked_pos = get_clicked_pos(&context.block_position, &context.face_clicked);

        let clicked_block_data = get_block_data_at(&clicked_pos, &state);

        let should_combine = is_same_slab_block(&clicked_block_data, &block_name);

        let (place_position, existing_block_data) = if should_combine {
            (clicked_pos, clicked_block_data)
        } else {
            let existing_block_data = get_block_data_at(&context.block_position, &state);
            (context.block_position, existing_block_data)
        };

        let is_same_slab = is_same_slab_block(&existing_block_data, &block_name);

        let block_data = if is_same_slab {
            temper_core::block_data::BlockData {
                name: block_name.to_string(),
                properties: Some(std::collections::BTreeMap::<String, String>::from([
                    ("type".to_string(), "double".to_string()),
                    ("waterlogged".to_string(), "false".to_string()),
                ])),
            }
        } else if existing_block_data.name == "minecraft:air" {
            let half = get_half(
                &context.face_clicked,
                &context.click_position,
                &context.block_position,
            );

            temper_core::block_data::BlockData {
                name: block_name.to_string(),
                properties: Some(std::collections::BTreeMap::<String, String>::from([
                    ("type".to_string(), half.to_string()),
                    ("waterlogged".to_string(), "false".to_string()),
                ])),
            }
        } else {
            // Cancel placement if the location is occupied by a block other than air or a combinable slab, or if it's already a double slab
            return Ok(PlacedBlocks {
                blocks: std::collections::HashMap::new(),
                take_item: false,
            });
        };

        let Some(block_id) = block_data.try_to_block_state_id() else {
            return Err(BlockPlaceError::BlockNotMappedToBlockStateId(block_data));
        };

        state
            .world
            .get_or_generate_mut(
                place_position.chunk(),
                temper_core::dimension::Dimension::Overworld,
            )
            .expect("Could not load chunk")
            .set_block(place_position.chunk_block_pos(), block_id);

        Ok(PlacedBlocks {
            blocks: std::collections::HashMap::from([(place_position, block_id)]),
            take_item: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockStateId;
    use std::collections::BTreeMap;
    use temper_components::player::rotation::Rotation;
    use temper_core::block_data::BlockData;
    use temper_core::dimension::Dimension;
    use temper_core::pos::BlockPos;
    use temper_macros::{block, item};

    fn slab_block_id(name: &str, slab_type: &str) -> BlockStateId {
        let bd = BlockData {
            name: name.to_string(),
            properties: Some(BTreeMap::from([
                ("type".to_string(), slab_type.to_string()),
                ("waterlogged".to_string(), "false".to_string()),
            ])),
        };
        bd.try_to_block_state_id()
            .expect("slab block id should exist")
    }

    #[test]
    fn test_combine_slab_into_double() {
        let (state, _tmp) = temper_state::create_test_state();

        // Place an oak bottom slab at (0,64,0)
        let bottom_id = slab_block_id("minecraft:oak_slab", "bottom");
        {
            let mut chunk = state
                .0
                .world
                .get_or_generate_mut(BlockPos::of(0, 64, 0).chunk(), Dimension::Overworld)
                .expect("Could not load chunk");
            chunk.set_block(BlockPos::of(0, 64, 0).chunk_block_pos(), bottom_id);
        }

        // Now simulate placing an oak slab by clicking the top face of the block above (so clicked_pos points to 0,64,0)
        let context = BlockPlaceContext {
            block_clicked: Default::default(),
            block_position: BlockPos::of(0, 65, 0),
            face_clicked: crate::BlockFace::Top,
            click_position: Default::default(),
            item_used: item!("oak_slab"),
            player_rotation: Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
            player_position: Default::default(),
        };

        let result = PlaceableSlab::place(context, state.0.clone());
        assert!(result.is_ok());
        let placed = result.unwrap();

        // Expect the bottom slab to have been converted to a double at (0,64,0)
        let double_id = slab_block_id("minecraft:oak_slab", "double");
        assert_eq!(placed.blocks.len(), 1);
        assert_eq!(
            placed.blocks.get(&BlockPos::of(0, 64, 0)).copied(),
            Some(double_id)
        );

        // And world should reflect the double slab
        let chunk = state
            .0
            .world
            .get_or_generate_chunk(BlockPos::of(0, 64, 0).chunk(), Dimension::Overworld)
            .expect("Could not load chunk");
        assert_eq!(
            chunk.get_block(BlockPos::of(0, 64, 0).chunk_block_pos()),
            double_id
        );
    }

    #[test]
    fn test_cancel_when_target_not_air() {
        let (state, _tmp) = temper_state::create_test_state();

        // Put a solid block (stone) at the target position
        {
            let mut chunk = state
                .0
                .world
                .get_or_generate_mut(BlockPos::of(0, 64, 0).chunk(), Dimension::Overworld)
                .expect("Could not load chunk");
            chunk.set_block(BlockPos::of(0, 64, 0).chunk_block_pos(), block!("stone"));
        }

        let context = BlockPlaceContext {
            block_clicked: Default::default(),
            block_position: BlockPos::of(0, 64, 0),
            face_clicked: crate::BlockFace::Top,
            click_position: Default::default(),
            item_used: item!("oak_slab"),
            player_rotation: Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
            player_position: Default::default(),
        };

        let result = PlaceableSlab::place(context, state.0.clone());
        assert!(result.is_ok());
        let placed = result.unwrap();
        // Should cancel placement
        assert!(placed.blocks.is_empty());
        assert!(!placed.take_item);
    }

    #[test]
    fn test_cancel_when_target_already_double() {
        let (state, _tmp) = temper_state::create_test_state();

        // Put a double oak slab at (0,64,0)
        let double_id = slab_block_id("minecraft:oak_slab", "double");
        {
            let mut chunk = state
                .0
                .world
                .get_or_generate_mut(BlockPos::of(0, 64, 0).chunk(), Dimension::Overworld)
                .expect("Could not load chunk");
            chunk.set_block(BlockPos::of(0, 64, 0).chunk_block_pos(), double_id);
        }

        let context = BlockPlaceContext {
            block_clicked: Default::default(),
            block_position: BlockPos::of(0, 64, 0),
            face_clicked: crate::BlockFace::Top,
            click_position: Default::default(),
            item_used: item!("oak_slab"),
            player_rotation: Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
            player_position: Default::default(),
        };

        let result = PlaceableSlab::place(context, state.0.clone());
        assert!(result.is_ok());
        let placed = result.unwrap();
        // Should cancel placement because target is already double
        assert!(placed.blocks.is_empty());
        assert!(!placed.take_item);
    }
}
