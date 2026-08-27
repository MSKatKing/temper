use bevy_ecs::prelude::{Query, Res, ResMut};
use temper_components::active_effects::ActiveEffects;
use temper_components::entity_identity::Identity;
use temper_components::health::Health;
use temper_components::player::abilities::PlayerAbilities;
use temper_components::player::experience::Experience;
use temper_components::player::gamemode::GameModeComponent;
use temper_components::player::gameplay_state::ender_chest::EnderChest;
use temper_components::player::hunger::Hunger;
use temper_components::player::offline_player_data::OfflinePlayerData;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_inventories::inventory::Inventory;
use temper_permissions::player::PlayerPermission;
use temper_resources::world_sync_tracker::WorldSyncTracker;
use temper_state::GlobalStateResource;
use tracing::error;

pub fn sync_world(
    player_query: Query<(
        &Identity,
        &PlayerAbilities,
        &GameModeComponent,
        &Position,
        &Rotation,
        &Inventory,
        &Health,
        &Hunger,
        &Experience,
        &EnderChest,
        &ActiveEffects,
        &PlayerPermission,
    )>,
    state: Res<GlobalStateResource>,
    mut last_synced: ResMut<WorldSyncTracker>,
) {
    // collect player data in RAM
    let mut players_to_save = Vec::with_capacity(player_query.iter().len());

    for (
        identity,
        abilities,
        gamemode,
        position,
        rotation,
        inventory,
        health,
        hunger,
        experience,
        ender_chest,
        active_effects,
        permissions,
    ) in player_query.iter()
    {
        let data = OfflinePlayerData {
            abilities: *abilities,
            gamemode: gamemode.0,
            position: (*position).into(),
            rotation: *rotation,
            inventory: inventory.clone(),
            health: *health,
            hunger: *hunger,
            experience: *experience,
            ender_chest: ender_chest.clone(),
            active_effects: active_effects.clone(),
            permissions: permissions.clone(),
        };
        players_to_save.push((identity.uuid, data));
    }

    // dispatch disk I/O to the thread pool
    let state_clone = state.clone();
    state.0.thread_pool.oneshot(move || {
        // 1. Sync chunks to disk
        if let Err(e) = state_clone.0.world.sync() {
            error!("Failed to sync world chunks to disk: {}", e);
        }

        // 2. Save player data to disk
        for (uuid, data) in players_to_save {
            if let Err(e) = state_clone.0.world.save_player_data(uuid, &data) {
                error!("Failed to save player data for {}: {}", uuid, e);
            }
        }
    });

    last_synced.last_synced = std::time::Instant::now();
}
