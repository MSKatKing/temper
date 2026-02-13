use bevy_ecs::schedule::IntoScheduleConfigs;
mod collisions;
mod drag;
mod gravity;
mod unground;
mod velocity;

pub fn register_physics_systems(schedule: &mut bevy_ecs::schedule::Schedule) {
    schedule.add_systems(
        (
            unground::handle,
            gravity::handle,
            drag::handle,
            velocity::handle,
            collisions::handle,
        )
            .chain(),
    );
}
