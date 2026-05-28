use bevy_ecs::message::MessageWriter;
use bevy_ecs::prelude::{Entity, Query, Res};
use interactions::block_interactions::is_interactive;
use temper_codec::net_types::network_position::NetworkPosition;
use temper_components::player::position::Position;
use temper_components::{bounds::CollisionBounds, player::sneak::SneakState};
use temper_core::pos::BlockPos;
use temper_messages::BlockInteractMessage;

use bevy_math::DVec3;
use temper_blocks::BlockDispatch;
use temper_components::player::rotation::Rotation;
use temper_config::server_config::get_global_config;
use temper_core::block_state_id::ITEM_TO_BLOCK_MAPPING;
use temper_core::dimension::Dimension;
use temper_core::mq;
use temper_inventories::hotbar::Hotbar;
use temper_inventories::inventory::Inventory;
use temper_messages::world_change::WorldChange;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::PlaceBlockReceiver;
use temper_protocol::outgoing::block_change_ack::BlockChangeAck;
use temper_protocol::outgoing::block_update::BlockUpdate;
use temper_state::GlobalStateResource;
use temper_text::{Color, NamedColor, TextComponentBuilder};
use tracing::{debug, error, trace};

pub fn handle(
    receiver: Res<PlaceBlockReceiver>,
    state: Res<GlobalStateResource>,
    query: Query<(
        Entity,
        &StreamWriter,
        &Inventory,
        &Hotbar,
        &Position,
        &Rotation,
        &SneakState,
    )>,
    pos_q: Query<(&Position, &CollisionBounds)>,
    mut world_change: MessageWriter<WorldChange>,
    mut block_interact: MessageWriter<BlockInteractMessage>,
) {
    'ev_loop: for (event, eid) in receiver.0.try_iter() {
        let Ok((entity, conn, inventory, hotbar, _pos, rot, sneak)) = query.get(eid) else {
            debug!("Could not get connection for entity {:?}", eid);
            continue;
        };
        if !state.0.players.is_connected(entity) {
            trace!("Entity {:?} is not connected", entity);
            continue;
        }
        // If the clicked block is interactive and the player is not sneaking,
        // dispatch an interaction and skip block placement entirely.
        {
            let clicked_pos: BlockPos = event.position.clone().into();
            let chunk = state
                .0
                .world
                .get_or_generate_chunk(clicked_pos.chunk(), Dimension::Overworld)
                .expect("Failed to load chunk for interaction check");
            let clicked_block = chunk.get_block(clicked_pos.chunk_block_pos());
            if !sneak.is_sneaking && is_interactive(clicked_block) {
                block_interact.write(BlockInteractMessage {
                    player: entity,
                    position: clicked_pos,
                    sequence: event.sequence,
                });
                continue 'ev_loop;
            }
        }

        match event.hand.0 {
            0 => {
                let Ok(slot) = hotbar.get_selected_item(inventory) else {
                    error!("Could not fetch {:?}", eid);
                    continue 'ev_loop;
                };
                if let Some(selected_item) = slot {
                    let Some(item_id) = selected_item.item_id else {
                        error!("Selected item has no item ID");
                        continue 'ev_loop;
                    };
                    let block_pos: BlockPos = event.position.into();
                    if block_pos.pos.y >= 319 {
                        mq::queue(
                            TextComponentBuilder::new(
                                "Build limit is 319! Cannot place block here.".to_string(),
                            )
                            .color(Color::Named(NamedColor::Red))
                            .bold()
                            .build(),
                            true,
                            entity,
                        );
                        trace!("Block placement out of bounds: {}", block_pos);
                        continue 'ev_loop;
                    } else if block_pos.pos.y <= -64 {
                        mq::queue(
                            TextComponentBuilder::new(
                                "Cannot place block below Y=-64.".to_string(),
                            )
                            .color(Color::Named(NamedColor::Red))
                            .bold()
                            .build(),
                            true,
                            entity,
                        );
                        trace!("Block placement out of bounds: {}", block_pos);
                        continue 'ev_loop;
                    }
                    let offset_pos = block_pos + event.face.get_normal().into();

                    // Check if the block collides with any entities
                    let does_collide = {
                        pos_q.into_iter().any(|(pos, bounds)| {
                            bounds.collides(
                                (pos.x, pos.y, pos.z),
                                &CollisionBounds {
                                    x_offset_start: 0.0,
                                    x_offset_end: 1.0,
                                    y_offset_start: 0.0,
                                    y_offset_end: 1.0,
                                    z_offset_start: 0.0,
                                    z_offset_end: 1.0,
                                },
                                (
                                    f64::from(offset_pos.pos.x),
                                    f64::from(offset_pos.pos.y),
                                    f64::from(offset_pos.pos.z),
                                ),
                            )
                        })
                    };

                    if does_collide {
                        trace!("Block placement collided with entity");
                        continue 'ev_loop;
                    }

                    let mut block_state = ITEM_TO_BLOCK_MAPPING
                        .get()
                        .unwrap()
                        .get(&(item_id.as_u32() as i32))
                        .copied()
                        .unwrap();

                    let mut placed_blocks = block_state.get_placement_state(temper_blocks::PlacementContext {
                        face: event.face,
                        cursor: DVec3::new(
                            f64::from(event.cursor_x),
                            f64::from(event.cursor_y),
                            f64::from(event.cursor_z),
                        ),
                        block_clicked: block_pos,
                        block_pos: offset_pos,
                        level: &state.0.world,
                        dimension: Dimension::Overworld,
                        player_rotation: rot
                    });

                    placed_blocks
                        .blocks
                        .insert(offset_pos, block_state);

                    for (block_pos, block_state) in placed_blocks.blocks.iter() {
                        let block_chunk = block_pos.chunk();
                        world_change.write(WorldChange {
                            chunk: Some(block_chunk),
                        });

                        let chunk_packet = BlockUpdate {
                            location: NetworkPosition {
                                x: block_pos.pos.x,
                                y: block_pos.pos.y as i16,
                                z: block_pos.pos.z,
                            },
                            block_state_id: block_state.to_varint(),
                        };

                        let (block_chunk_x, block_chunk_z) = (block_chunk.x(), block_chunk.z());
                        let render_distance = get_global_config().chunk_render_distance as i32;
                        for (_, conn, _, _, pos, _, _) in query.iter() {
                            let chunk = pos.chunk();
                            let (chunk_x, chunk_z) = (chunk.x(), chunk.z());

                            // Only send block update if the player is within the render distance of the block being updated
                            if (block_chunk_x - chunk_x).abs() <= render_distance
                                && (block_chunk_z - chunk_z).abs() <= render_distance
                                && let Err(err) = conn.send_packet_ref(&chunk_packet)
                            {
                                error!("Failed to send block update packet: {:?}", err);
                            }
                        }
                    }
                }
                let ack_packet = BlockChangeAck {
                    sequence: event.sequence,
                };

                if let Err(err) = conn.send_packet_ref(&ack_packet) {
                    error!("Failed to send block change ack packet: {:?}", err);
                    continue 'ev_loop;
                }
            }
            1 => {
                trace!("Offhand block placement not implemented");
            }
            _ => {
                debug!("Invalid hand");
            }
        }
    }
}
