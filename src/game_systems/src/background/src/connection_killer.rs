use bevy_ecs::prelude::{Commands, Entity, MessageWriter, Query, Res};
use temper_components::entity_identity::Identity;
use temper_components::player::offline_player_data::OfflinePlayerData;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_components::{
    active_effects::ActiveEffects,
    health::Health,
    player::{
        abilities::PlayerAbilities, experience::Experience, gamemode::GameModeComponent,
        gameplay_state::ender_chest::EnderChest, hunger::Hunger,
    },
};
use temper_inventories::inventory::Inventory;
use temper_messages::player_leave::PlayerLeft;
use temper_net_runtime::connection::StreamWriter;
use temper_state::GlobalStateResource;
use temper_text::TextComponent;
use tracing::{debug, info, trace, warn};

// This type alias defines all the components of a "full" player
type PlayerCacheQuery<'a> = (
    Entity,
    &'a StreamWriter,
    &'a Identity,
    &'a PlayerAbilities,
    &'a GameModeComponent,
    &'a Position,
    &'a Rotation,
    &'a Inventory,
    &'a Health,
    &'a Hunger,
    &'a Experience,
    &'a EnderChest,
    &'a ActiveEffects,
);

// This query is a "fallback" for half-connected players
type IdentityQuery<'a> = &'a Identity;

pub fn connection_killer(
    full_player_query: Query<PlayerCacheQuery>,
    identity_query: Query<IdentityQuery>,
    mut cmd: Commands,
    state: Res<GlobalStateResource>,
    mut leave_events: MessageWriter<PlayerLeft>,
) {
    // Loop through all entities marked for disconnection
    while let Some((disconnecting_entity, reason)) = state.0.players.disconnection_queue.pop() {
        trace!(
            "Processing disconnect for entity: {:?}",
            disconnecting_entity
        );

        // --- 1. Try to get the "full" player ---
        if let Ok((
            _entity,
            conn,
            player_identity,
            abilities,
            gamemode,
            pos,
            rot,
            inv,
            health,
            hunger,
            exp,
            echest,
            effects,
        )) = full_player_query.get(disconnecting_entity)
        {
            let username = player_identity.name.as_ref().expect("No Player Name");
            // --- SUCCESS: This is a fully-joined player ---
            info!(
                "Player {} ({}) disconnected: {}.",
                username,
                player_identity.uuid,
                reason.as_deref().unwrap_or("No reason")
            );

            // Send disconnect packet
            if conn.running.load(std::sync::atomic::Ordering::Relaxed) {
                trace!("Sending disconnect packet to player {}", username);
                if let Err(e) =
                    conn.send_packet_ref(&temper_protocol::outgoing::disconnect::DisconnectPacket {
                        reason: TextComponent::from(reason.as_deref().unwrap_or("Disconnected"))
                            .into(),
                    })
                {
                    warn!(
                        "Failed to send disconnect packet to player {}: {:?}",
                        username, e
                    );
                }
            } else {
                trace!(
                    "Connection for player {} is not running, skipping disconnect packet",
                    username
                );
            }

            // Save data to disk
            let data_to_cache = OfflinePlayerData {
                abilities: *abilities,
                gamemode: gamemode.0,
                position: (*pos).into(),
                rotation: *rot,
                inventory: inv.clone(),
                health: *health,
                hunger: *hunger,
                experience: *exp,
                ender_chest: echest.clone(),
                active_effects: effects.clone(),
            };
            if let Err(err) = state
                .0
                .world
                .save_player_data(player_identity.uuid, &data_to_cache)
            {
                warn!("Failed to save player data for {}: {:?}", username, err);
            } else {
                debug!("Successfully saved player data for {}", username);
            }

            // --- 3. Fire PlayerLeaveEvent ---
            leave_events.write(PlayerLeft(player_identity.clone()));
        } else {
            // --- FAILURE: This is a "half-player" or zombie ---
            warn!(
                "Player's entity {:?} is missing components (likely a failed handshake). Despawning...",
                disconnecting_entity
            );

            // Try to get at least the identity to broadcast the leave message
            if let Ok(player_identity) = identity_query.get(disconnecting_entity) {
                warn!(
                    "-> (Half-player had identity: {})",
                    player_identity.name.as_ref().expect("No Player Name")
                );
                leave_events.write(PlayerLeft(player_identity.clone()));
            } else {
                warn!("-> (Half-player didn't even have an identity component!)");
            }
        }

        // --- 2. ALWAYS Despawn (but safely) ---
        // We do this check to prevent a crash if the queue had a duplicate
        if let Ok(mut entity_commands) = cmd.get_entity(disconnecting_entity) {
            trace!("Despawning entity {:?}", disconnecting_entity);
            entity_commands.despawn();
        } else {
            trace!(
                "Entity {:?} was already despawned (duplicate disconnect message).",
                disconnecting_entity
            );
        }
    }
}
