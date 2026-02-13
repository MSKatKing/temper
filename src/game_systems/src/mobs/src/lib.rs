use bevy_ecs::prelude::Schedule;

mod pig;
pub fn register_mob_systems(schedule: &mut Schedule) {
    schedule.add_systems(pig::tick_pig);
}
