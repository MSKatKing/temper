use bevy_ecs::prelude::Query;
use tracing::warn;
use temper_components::player::bossbar_sender::BossbarSender;
use temper_net_runtime::connection::StreamWriter;
use temper_resources::bossbar::{BossBarResource, UpdateBBKind};
use uuid::Uuid;
use temper_protocol::outgoing::bossbar_packet::BossbarPacket;

pub fn update_bossbars(
    player_query: Query<(&StreamWriter, &BossbarSender)>,
    boss_bar_resource: &mut BossBarResource,
) {
    let mut updated: Vec<(Uuid, UpdateBBKind)> = Vec::new();

    while let Some((uuid, update_kind)) = boss_bar_resource.update_queue.pop() {
        match &update_kind {
            UpdateBBKind::Add { data } => {
                boss_bar_resource.boss_bars.insert(uuid, data.clone());
            }
            UpdateBBKind::Remove => {
                boss_bar_resource.boss_bars.remove(&uuid);
            }
            UpdateBBKind::UpdateHealth { new_health } => {
                if let Some(data) = boss_bar_resource.boss_bars.get_mut(&uuid) {
                    data.health = *new_health;
                }
            }
            UpdateBBKind::UpdateTitle { title } => {
                if let Some(data) = boss_bar_resource.boss_bars.get_mut(&uuid) {
                    data.title = title.clone();
                }
            }
            UpdateBBKind::UpdateStyle { color, dividers } => {
                if let Some(data) = boss_bar_resource.boss_bars.get_mut(&uuid) {
                    data.color = color.clone();
                    data.dividers = dividers.clone();
                }
            }
            UpdateBBKind::UpdateFlags { flags } => {
                if let Some(data) = boss_bar_resource.boss_bars.get_mut(&uuid) {
                    data.flags = flags.clone();
                }
            }
        }
        updated.push((uuid, update_kind));
    }

    for (writer, _bossbar_sender) in player_query.iter() {
        for (uuid, update_kind) in &updated {
            let packet = match update_kind {
                UpdateBBKind::Add { data } => {
                    BossbarPacket::add_bossbar(uuid.as_u128(), data.title.clone(), data.health, data.color.discriminant().into(), data.dividers.discriminant().into(), data.flags.get())
                }
                UpdateBBKind::Remove => BossbarPacket::remove_bossbar(uuid.as_u128()),
                UpdateBBKind::UpdateHealth { new_health } => BossbarPacket::update_health(uuid.as_u128(), *new_health),
                UpdateBBKind::UpdateTitle { title } => BossbarPacket::update_title(uuid.as_u128(), title.clone()),
                UpdateBBKind::UpdateStyle { color, dividers } => {
                    BossbarPacket::update_style(uuid.as_u128(), color.discriminant().into(), dividers.discriminant().into())
                }
                UpdateBBKind::UpdateFlags { flags } => BossbarPacket::update_flags(uuid.as_u128(), flags.get()),
            };
            writer.send_packet_ref(&packet).unwrap_or_else(|_| {
                warn!("Failed to send Bossbar Packet to player");
            });
        }
    }
}