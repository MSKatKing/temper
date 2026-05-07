use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule};

crate::define_standard_mob_save_load!(
    axolotl,
    marker = temper_entities::markers::entity_types::Axolotl,
    bundle = temper_entities::AxolotlBundle,
    entity_type = Axolotl,
    runtime_components = (
        temper_entities::markers::HasGravity,
        temper_entities::markers::HasCollisions
    )
);

crate::define_standard_mob_save_load!(
    magma_cube,
    marker = temper_entities::markers::entity_types::MagmaCube,
    bundle = temper_entities::MagmaCubeBundle,
    entity_type = MagmaCube,
    runtime_components = (
        temper_entities::markers::HasGravity,
        temper_entities::markers::HasCollisions
    )
);

crate::define_standard_mob_save_load!(
    slime,
    marker = temper_entities::markers::entity_types::Slime,
    bundle = temper_entities::SlimeBundle,
    entity_type = Slime,
    runtime_components = (
        temper_entities::markers::HasGravity,
        temper_entities::markers::HasCollisions
    )
);

crate::define_standard_mob_save_load!(
    strider,
    marker = temper_entities::markers::entity_types::Strider,
    bundle = temper_entities::StriderBundle,
    entity_type = Strider,
    runtime_components = (
        temper_entities::markers::HasGravity,
        temper_entities::markers::HasCollisions
    )
);

pub fn register_load_systems(schedule: &mut Schedule) {
    crate::add_systems_to_set!(
        schedule,
        crate::MobLoadSystems,
        [load_axolotl, load_magma_cube, load_slime, load_strider,]
    );
}

pub fn register_save_systems(schedule: &mut Schedule) {
    crate::add_systems_to_set!(
        schedule,
        crate::MobSaveSystems,
        [save_axolotl, save_magma_cube, save_slime, save_strider,]
    );
}
