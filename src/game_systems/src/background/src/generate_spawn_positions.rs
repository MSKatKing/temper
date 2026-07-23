use bevy_ecs::prelude::Res;
use bevy_math::DVec3;
use std::time::Duration;
use temper_components::player::position::Position;
use temper_core::block_state_id::BlockStateId;
use temper_core::dimension::Dimension::Overworld;
use temper_macros::match_block;
use temper_state::GlobalStateResource;
use tracing::{info, trace};

const SPAWN_CENTER: (i32, i32) = (8, 8);
const MIN_RADIUS: u32 = 8;

// Basically the easiest way to have available spawn positions is to just
// generate a bunch and chuck them in a queue that can be pulled from
pub fn generate_spawn_positions(state: Res<GlobalStateResource>) {
    let start = std::time::Instant::now();

    if state.0.spawn_positions.is_full() {
        return;
    }

    let mut found_coords = 0;

    while !state.0.spawn_positions.is_full() {
        // How many attempts to find a spawn location we should try before upping the search radius
        let mut expand_cooldown = 4096;

        let mut radius = MIN_RADIUS;

        let mut found: Option<Position> = None;

        if start.elapsed() > Duration::from_millis(30) {
            info!("Generating spawn positions is taking longer than expected.");
            return;
        }

        while found.is_none() {
            if start.elapsed() > Duration::from_millis(30) {
                info!("Generating spawn positions is taking longer than expected.");
                return;
            }

            let x = SPAWN_CENTER.0 + rand::random_range(-(radius as i32)..radius as i32);
            let z = SPAWN_CENTER.1 + rand::random_range(-(radius as i32)..radius as i32);

            let pos = Position::new(f64::from(x), 0.0, f64::from(z));

            let chunk_pos = pos.chunk();

            let chunk = state
                .0
                .world
                .get_or_generate_chunk(chunk_pos, Overworld)
                .expect("Failed to generate chunk");

            let height = chunk
                .heightmaps
                .motion_blocking
                .get_height(chunk_column_pos(x), chunk_column_pos(z));

            let new_pos = Position::new(pos.x, f64::from(height), pos.z);

            let candidate_block = chunk.get_block((new_pos.as_ivec3()).into());

            if is_valid_spawn_surface(candidate_block) {
                found = Some(spawn_position_above_surface(new_pos));
                found_coords += 1;
                break;
            } else {
                if expand_cooldown > 0 {
                    expand_cooldown -= 1;
                } else {
                    radius += 10;
                }
            }
        }

        state
            .0
            .spawn_positions
            .push(found.expect("No coords found").xyz())
            .expect("Cannot push to queue");
    }
    trace!(
        "Finished generating {} spawn positions in {:.2} ms",
        found_coords,
        start.elapsed().as_secs_f32() * 1000.0
    );
}

fn is_valid_spawn_surface(block: BlockStateId) -> bool {
    !(match_block!("air", block)
        || match_block!("void_air", block)
        || match_block!("water", block)
        || match_block!("lava", block))
}

fn spawn_position_above_surface(surface_pos: Position) -> Position {
    (*surface_pos + DVec3::new(0.5, 1.0, 0.5)).into()
}

fn chunk_column_pos(coord: i32) -> u8 {
    coord.rem_euclid(16) as u8
}

#[cfg(test)]
mod tests {
    use temper_macros::block;

    use super::*;

    #[test]
    fn spawn_surface_rejects_air_and_fluids() {
        assert!(!is_valid_spawn_surface(block!("air")));
        assert!(!is_valid_spawn_surface(block!("void_air")));
        assert!(!is_valid_spawn_surface(block!("water", {level: 0})));
        assert!(!is_valid_spawn_surface(block!("lava", {level: 0})));
        assert!(is_valid_spawn_surface(
            block!("grass_block", {snowy: false})
        ));
        assert!(is_valid_spawn_surface(block!("dirt")));
    }

    #[test]
    fn spawn_position_sits_above_surface() {
        let spawn_pos = spawn_position_above_surface(Position::new(5.0, 32.0, 10.0));

        assert_eq!(spawn_pos.xyz(), (5.5, 33.0, 10.5));
    }

    #[test]
    fn chunk_column_positions_wrap_negative_coordinates() {
        assert_eq!(chunk_column_pos(-1), 15);
        assert_eq!(chunk_column_pos(-16), 0);
        assert_eq!(chunk_column_pos(17), 1);
    }
}
