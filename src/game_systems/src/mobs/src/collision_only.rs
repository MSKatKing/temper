use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule};

crate::define_standard_mob_save_load!(
    allay,
    marker = temper_entities::markers::entity_types::Allay,
    bundle = temper_entities::AllayBundle,
    entity_type = Allay,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    bat,
    marker = temper_entities::markers::entity_types::Bat,
    bundle = temper_entities::BatBundle,
    entity_type = Bat,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    bee,
    marker = temper_entities::markers::entity_types::Bee,
    bundle = temper_entities::BeeBundle,
    entity_type = Bee,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    blaze,
    marker = temper_entities::markers::entity_types::Blaze,
    bundle = temper_entities::BlazeBundle,
    entity_type = Blaze,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    breeze,
    marker = temper_entities::markers::entity_types::Breeze,
    bundle = temper_entities::BreezeBundle,
    entity_type = Breeze,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    cod,
    marker = temper_entities::markers::entity_types::Cod,
    bundle = temper_entities::CodBundle,
    entity_type = Cod,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    dolphin,
    marker = temper_entities::markers::entity_types::Dolphin,
    bundle = temper_entities::DolphinBundle,
    entity_type = Dolphin,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    drowned,
    marker = temper_entities::markers::entity_types::Drowned,
    bundle = temper_entities::DrownedBundle,
    entity_type = Drowned,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    elder_guardian,
    marker = temper_entities::markers::entity_types::ElderGuardian,
    bundle = temper_entities::ElderGuardianBundle,
    entity_type = ElderGuardian,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    ghast,
    marker = temper_entities::markers::entity_types::Ghast,
    bundle = temper_entities::GhastBundle,
    entity_type = Ghast,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    glow_squid,
    marker = temper_entities::markers::entity_types::GlowSquid,
    bundle = temper_entities::GlowSquidBundle,
    entity_type = GlowSquid,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    guardian,
    marker = temper_entities::markers::entity_types::Guardian,
    bundle = temper_entities::GuardianBundle,
    entity_type = Guardian,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    parrot,
    marker = temper_entities::markers::entity_types::Parrot,
    bundle = temper_entities::ParrotBundle,
    entity_type = Parrot,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    phantom,
    marker = temper_entities::markers::entity_types::Phantom,
    bundle = temper_entities::PhantomBundle,
    entity_type = Phantom,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    pufferfish,
    marker = temper_entities::markers::entity_types::Pufferfish,
    bundle = temper_entities::PufferfishBundle,
    entity_type = Pufferfish,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    salmon,
    marker = temper_entities::markers::entity_types::Salmon,
    bundle = temper_entities::SalmonBundle,
    entity_type = Salmon,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    squid,
    marker = temper_entities::markers::entity_types::Squid,
    bundle = temper_entities::SquidBundle,
    entity_type = Squid,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    tadpole,
    marker = temper_entities::markers::entity_types::Tadpole,
    bundle = temper_entities::TadpoleBundle,
    entity_type = Tadpole,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    tropical_fish,
    marker = temper_entities::markers::entity_types::TropicalFish,
    bundle = temper_entities::TropicalFishBundle,
    entity_type = TropicalFish,
    runtime_components = (temper_entities::markers::HasCollisions)
);

crate::define_standard_mob_save_load!(
    vex,
    marker = temper_entities::markers::entity_types::Vex,
    bundle = temper_entities::VexBundle,
    entity_type = Vex,
    runtime_components = (temper_entities::markers::HasCollisions)
);

pub fn register_load_systems(schedule: &mut Schedule) {
    crate::add_systems_to_set!(
        schedule,
        crate::MobLoadSystems,
        [
            load_allay,
            load_bat,
            load_bee,
            load_blaze,
            load_breeze,
            load_cod,
            load_dolphin,
            load_drowned,
            load_elder_guardian,
            load_ghast,
            load_glow_squid,
            load_guardian,
            load_parrot,
            load_phantom,
            load_pufferfish,
            load_salmon,
            load_squid,
            load_tadpole,
            load_tropical_fish,
            load_vex,
        ]
    );
}

pub fn register_save_systems(schedule: &mut Schedule) {
    crate::add_systems_to_set!(
        schedule,
        crate::MobSaveSystems,
        [
            save_allay,
            save_bat,
            save_bee,
            save_blaze,
            save_breeze,
            save_cod,
            save_dolphin,
            save_drowned,
            save_elder_guardian,
            save_ghast,
            save_glow_squid,
            save_guardian,
            save_parrot,
            save_phantom,
            save_pufferfish,
            save_salmon,
            save_squid,
            save_tadpole,
            save_tropical_fish,
            save_vex,
        ]
    );
}
