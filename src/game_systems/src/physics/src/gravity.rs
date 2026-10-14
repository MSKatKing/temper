use bevy_ecs::prelude::{Has, Query, Res, With};
use temper_components::player::dimension::CurrentDimension;
use temper_components::player::grounded::OnGround;
use temper_components::player::position::Position;
use temper_components::player::velocity::Velocity;
use temper_core::block_state_id::BlockStateId;
use temper_core::pos::{ChunkBlockPos, ChunkPos};
use temper_entities::markers::{HasGravity, HasWaterDrag};
use temper_macros::match_block;
use temper_physics::GRAVITY_ACCELERATION;
use temper_state::GlobalStateResource;

type EntityQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Velocity,
        &'static OnGround,
        &'static Position,
        &'static CurrentDimension,
        Has<HasWaterDrag>,
    ),
    With<HasGravity>,
>;

// Just apply gravity to a mob's velocity. Application of velocity is handled elsewhere.
pub fn handle(mut entities: EntityQuery, state: Res<GlobalStateResource>) {
    for (mut vel, grounded, pos, dim, is_water) in entities.iter_mut() {
        if grounded.currently_grounded {
            continue;
        }

        if is_water {
            let chunk_pos = ChunkPos::from(pos.coords);
            let chunk = state
                .0
                .world
                .get_or_generate_mut(chunk_pos.full(dim.0))
                .expect("Failed to load or generate chunk");

            let feet_pos = pos.coords.as_ivec3();

            let is_in_water = match_block!("water", chunk.get_block(ChunkBlockPos::from(feet_pos)));

            // Only apply full gravity if NOT in water
            if !is_in_water {
                vel.vec += GRAVITY_ACCELERATION;
            }
        } else {
            // Apply gravity
            vel.vec += GRAVITY_ACCELERATION;
        }
    }
}
