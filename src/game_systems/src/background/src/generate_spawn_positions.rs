use bevy_ecs::prelude::Res;
use bevy_math::DVec3;
use std::time::Duration;
use temper_components::player::position::Position;
use temper_core::block_state_id::BlockStateId;
use temper_core::dimension::Dimension::Overworld;
use temper_macros::match_block;
use temper_state::GlobalStateResource;
use tracing::{info, trace};

// Basically the easiest way to have available spawn positions is to just
// generate a bunch and chuck them in a queue that can be pulled from
pub fn generate_spawn_positions(state: Res<GlobalStateResource>) {
    // TODO: Make dynamic with config or something
    /// Where the spawn radius is centered
    const SPAWN_CENTER: (i32, i32) = (100, 100);

    /// The minimum radius to spawn a player
    const MIN_RADIUS: u32 = 100;

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
            let x = SPAWN_CENTER.0 + (rand::random_range(-(radius as i32)..radius as i32));
            let z = SPAWN_CENTER.1 + (rand::random_range(-(radius as i32)..radius as i32));

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
                .get_height((pos.x as u32 % 16) as u8, (pos.z as u32 % 16) as u8);

            let new_pos = Position::new(pos.x, f64::from(height) - 1.0, pos.z);

            let candidate_block = chunk.get_block((new_pos.as_ivec3()).into());

            if !(match_block!("water", candidate_block) || match_block!("lava", candidate_block)) {
                found = Some((*new_pos + DVec3::new(0.5, 2.0, 0.5)).into());
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
