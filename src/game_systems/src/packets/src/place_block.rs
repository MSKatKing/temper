use bevy_ecs::message::MessageWriter;
use bevy_ecs::prelude::{Entity, Query, Res};
use interactions::block_interactions::is_interactive;
use temper_codec::net_types::network_position::NetworkPosition;
use temper_components::player::position::Position;
use temper_components::{bounds::CollisionBounds, player::sneak::SneakState};
use temper_core::pos::BlockPos;
use temper_messages::BlockInteractMessage;

use temper_net_runtime::connection::StreamWriter;
use temper_protocol::PlaceBlockReceiver;
use temper_protocol::outgoing::block_change_ack::BlockChangeAck;
use temper_protocol::outgoing::block_update::BlockUpdate;
use temper_state::GlobalStateResource;
use tracing::{debug, error, trace};

use bevy_math::DVec3;
use block_placing::PlacedBlocks;
use std::collections::HashMap;
use temper_components::player::rotation::Rotation;
use temper_core::dimension::Dimension;
use temper_core::mq;
use temper_inventories::hotbar::Hotbar;
use temper_inventories::inventory::Inventory;
use temper_messages::{world_change::WorldChange, SpawnMobBundle};
use temper_entities::{MobBundle, entity_types::EntityTypeEnum};
use temper_text::{Color, NamedColor, TextComponentBuilder};

fn from_spawn_egg_name(name: &str) -> Option<EntityTypeEnum> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    let name = name.strip_suffix("_spawn_egg")?;
    match name {
        "allay" => Some(EntityTypeEnum::Allay),
        "armadillo" => Some(EntityTypeEnum::Armadillo),
        "axolotl" => Some(EntityTypeEnum::Axolotl),
        "bat" => Some(EntityTypeEnum::Bat),
        "bee" => Some(EntityTypeEnum::Bee),
        "camel" => Some(EntityTypeEnum::Camel),
        "cat" => Some(EntityTypeEnum::Cat),
        "cave_spider" => Some(EntityTypeEnum::CaveSpider),
        "chicken" => Some(EntityTypeEnum::Chicken),
        "cod" => Some(EntityTypeEnum::Cod),
        "cow" => Some(EntityTypeEnum::Cow),
        "dolphin" => Some(EntityTypeEnum::Dolphin),
        "donkey" => Some(EntityTypeEnum::Donkey),
        "drowned" => Some(EntityTypeEnum::Drowned),
        "enderman" => Some(EntityTypeEnum::Enderman),
        "fox" => Some(EntityTypeEnum::Fox),
        "frog" => Some(EntityTypeEnum::Frog),
        "goat" => Some(EntityTypeEnum::Goat),
        "horse" => Some(EntityTypeEnum::Horse),
        "iron_golem" => Some(EntityTypeEnum::IronGolem),
        "llama" => Some(EntityTypeEnum::Llama),
        "mooshroom" => Some(EntityTypeEnum::Mooshroom),
        "ocelot" => Some(EntityTypeEnum::Ocelot),
        "panda" => Some(EntityTypeEnum::Panda),
        "parrot" => Some(EntityTypeEnum::Parrot),
        "pig" => Some(EntityTypeEnum::Pig),
        "piglin" => Some(EntityTypeEnum::Piglin),
        "polar_bear" => Some(EntityTypeEnum::PolarBear),
        "pufferfish" => Some(EntityTypeEnum::Pufferfish),
        "rabbit" => Some(EntityTypeEnum::Rabbit),
        "salmon" => Some(EntityTypeEnum::Salmon),
        "sheep" => Some(EntityTypeEnum::Sheep),
        "skeleton_horse" => Some(EntityTypeEnum::SkeletonHorse),
        "sniffer" => Some(EntityTypeEnum::Sniffer),
        "snow_golem" => Some(EntityTypeEnum::SnowGolem),
        "spider" => Some(EntityTypeEnum::Spider),
        "squid" => Some(EntityTypeEnum::Squid),
        "strider" => Some(EntityTypeEnum::Strider),
        "tadpole" => Some(EntityTypeEnum::Tadpole),
        "trader_llama" => Some(EntityTypeEnum::TraderLlama),
        "tropical_fish" => Some(EntityTypeEnum::TropicalFish),
        "turtle" => Some(EntityTypeEnum::Turtle),
        "villager" => Some(EntityTypeEnum::Villager),
        "wandering_trader" => Some(EntityTypeEnum::WanderingTrader),
        "wolf" => Some(EntityTypeEnum::Wolf),
        "zombie_horse" => Some(EntityTypeEnum::ZombieHorse),
        "zombified_piglin" => Some(EntityTypeEnum::ZombifiedPiglin),
        "glow_squid" => Some(EntityTypeEnum::GlowSquid),
        "mule" => Some(EntityTypeEnum::Mule),
        "blaze" => Some(EntityTypeEnum::Blaze),
        "bogged" => Some(EntityTypeEnum::Bogged),
        "breeze" => Some(EntityTypeEnum::Breeze),
        "creaking" => Some(EntityTypeEnum::Creaking),
        "creeper" => Some(EntityTypeEnum::Creeper),
        "elder_guardian" => Some(EntityTypeEnum::ElderGuardian),
        "endermite" => Some(EntityTypeEnum::Endermite),
        "evoker" => Some(EntityTypeEnum::Evoker),
        "ghast" => Some(EntityTypeEnum::Ghast),
        "guardian" => Some(EntityTypeEnum::Guardian),
        "hoglin" => Some(EntityTypeEnum::Hoglin),
        "husk" => Some(EntityTypeEnum::Husk),
        "magma_cube" => Some(EntityTypeEnum::MagmaCube),
        "phantom" => Some(EntityTypeEnum::Phantom),
        "piglin_brute" => Some(EntityTypeEnum::PiglinBrute),
        "pillager" => Some(EntityTypeEnum::Pillager),
        "ravager" => Some(EntityTypeEnum::Ravager),
        "shulker" => Some(EntityTypeEnum::Shulker),
        "silverfish" => Some(EntityTypeEnum::Silverfish),
        "skeleton" => Some(EntityTypeEnum::Skeleton),
        "slime" => Some(EntityTypeEnum::Slime),
        "stray" => Some(EntityTypeEnum::Stray),
        "vex" => Some(EntityTypeEnum::Vex),
        "vindicator" => Some(EntityTypeEnum::Vindicator),
        "warden" => Some(EntityTypeEnum::Warden),
        "witch" => Some(EntityTypeEnum::Witch),
        "wither_skeleton" => Some(EntityTypeEnum::WitherSkeleton),
        "zoglin" => Some(EntityTypeEnum::Zoglin),
        "zombie" => Some(EntityTypeEnum::Zombie),
        "zombie_villager" => Some(EntityTypeEnum::ZombieVillager),
        _ => None,
    }
}

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
    mut mob_bundle_events: MessageWriter<SpawnMobBundle>,
) {
    'ev_loop: for (event, eid) in receiver.0.try_iter() {
        let Ok((entity, conn, inventory, hotbar, pos, rot, sneak)) = query.get(eid) else {
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
                    let offset_pos = block_pos
                        + match event.face.0 {
                            0 => (0, -1, 0),
                            1 => (0, 1, 0),
                            2 => (0, 0, -1),
                            3 => (0, 0, 1),
                            4 => (-1, 0, 0),
                            5 => (1, 0, 0),
                            _ => (0, 0, 0),
                        };

                    if let Some(item_name) = item_id.to_name() {
                        if item_name.ends_with("_spawn_egg") {
                            if let Some(entity_type) = from_spawn_egg_name(&item_name) {
                                let spawn_pos_vec = Position::new(
                                    f64::from(offset_pos.pos.x) + 0.5,
                                    f64::from(offset_pos.pos.y),
                                    f64::from(offset_pos.pos.z) + 0.5,
                                );
                                mob_bundle_events.write(SpawnMobBundle {
                                    bundle: MobBundle::new(entity_type, spawn_pos_vec),
                                    persist: true,
                                });

                                let ack_packet = BlockChangeAck {
                                    sequence: event.sequence,
                                };

                                if let Err(err) = conn.send_packet_ref(&ack_packet) {
                                    error!("Failed to send block change ack packet: {:?}", err);
                                }
                                continue 'ev_loop;
                            }
                        }
                    }

                    let block_clicked = {
                        let chunk = state
                            .0
                            .world
                            .get_or_generate_chunk(block_pos.chunk(), Dimension::Overworld)
                            .expect("Failed to load or generate chunk");
                        chunk.get_block(block_pos.chunk_block_pos())
                    };

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

                    let _block_at_pos = {
                        let chunk = state
                            .0
                            .world
                            .get_or_generate_chunk(offset_pos.chunk(), Dimension::Overworld)
                            .expect("Failed to load or generate chunk");
                        chunk.get_block(offset_pos.chunk_block_pos())
                    };

                    let placed_blocks = block_placing::place_item(
                        state.0.clone(),
                        block_placing::BlockPlaceContext {
                            block_clicked,
                            block_position: offset_pos,
                            face_clicked: match event.face.0 {
                                0 => block_placing::BlockFace::Bottom,
                                1 => block_placing::BlockFace::Top,
                                2 => block_placing::BlockFace::North,
                                3 => block_placing::BlockFace::South,
                                4 => block_placing::BlockFace::West,
                                5 => block_placing::BlockFace::East,
                                _ => {
                                    debug!("Invalid block face");
                                    continue 'ev_loop;
                                }
                            },
                            click_position: DVec3::new(
                                f64::from(event.cursor_x),
                                f64::from(event.cursor_y),
                                f64::from(event.cursor_z),
                            ),
                            player_position: *pos,
                            player_rotation: *rot,
                            item_used: item_id,
                        },
                    )
                    .unwrap_or_else(|err| {
                        error!("Block placement failed: {:?}", err);
                        PlacedBlocks {
                            blocks: HashMap::new(),
                            take_item: false,
                        }
                    });

                    for (block_pos, block_state) in placed_blocks.blocks {
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
                        let render_distance = state.0.config.chunk_render_distance as i32;
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
