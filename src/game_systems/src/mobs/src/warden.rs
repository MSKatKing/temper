use bevy_ecs::prelude::*;
use temper_components::bossbar::BossbarOwner;
use temper_components::player::bossbar_sender::BossbarSender;
use temper_components::player::grounded::OnGround;
use temper_components::player::player_marker::PlayerMarker;
use temper_components::player::position::Position;
use temper_components::player::velocity::Velocity;
use temper_entities::markers::entity_types::Warden;
use temper_resources::bossbar::{BossBarData, BossBarResource, BossbarColor};
use temper_text::TextComponent;
use uuid::Uuid;

type WardenQuery<'a> = (
    &'a Position,
    &'a mut Velocity,
    &'a OnGround,
    &'a BossbarOwner,
);

pub fn init_warden(
    mut commands: Commands,
    warden: Query<Entity, (With<Warden>, Without<BossbarOwner>)>,
    boss_bar_resource: ResMut<BossBarResource>,
) {
    for entity in warden.iter() {
        let data = BossBarData::new(
            TextComponent::from("Warden"),
            500.0,
            500.0,
            BossbarColor::Blue,
        );

        let uuid = boss_bar_resource.add_bar(data);

        let owner = BossbarOwner::new(uuid.as_u128());

        commands.entity(entity).insert(owner);
    }
}

pub fn tick_warden(
    warden: Query<WardenQuery, (With<Warden>, With<BossbarOwner>)>,
    mut players: Query<(&Position, &mut BossbarSender), With<PlayerMarker>>,
    boss_bar_resource: ResMut<BossBarResource>,
) {
    for (warden_pos, _, _, owned_bossbar) in warden.iter() {
        let id = owned_bossbar.id();
        let uuid = Uuid::from_u128(id);

        for (player_pos, mut bossbar_sender) in players.iter_mut() {
            let dx = warden_pos.x - player_pos.x;
            let dy = warden_pos.y - player_pos.y;
            let dz = warden_pos.z - player_pos.z;

            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

            let current = bossbar_sender.0.get(&id).copied();

            // --- ENTER RANGE ---
            if distance <= 10.0 {
                if current.is_none() {
                    bossbar_sender.add(uuid);
                    boss_bar_resource.queue_networking(uuid, true);
                }
            }
            // --- EXIT RANGE ---
            else if distance > 12.0
                && let Some(_) = current
            {
                bossbar_sender.remove(uuid);
                boss_bar_resource.queue_networking(uuid, false);
            }

            // --- NO CHANGE ZONE (10–12) ---
        }
    }
}
