use bevy_ecs::prelude::{Query, ResMut};
use temper_components::player::bossbar_sender::BossbarSender;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::boss_event::BossbarPacket;
use temper_resources::bossbar::{BossBarResource, UpdateBBKind};
use tracing::warn;
use uuid::Uuid;

pub fn handle(
    mut player_query: Query<(&StreamWriter, &mut BossbarSender)>,
    mut boss_bar_resource: ResMut<BossBarResource>,
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
            UpdateBBKind::UpdateHealth {
                new_health,
                new_max,
            } => {
                if let Some(data) = boss_bar_resource.boss_bars.get_mut(&uuid) {
                    data.health = *new_health;
                    data.max = *new_max;
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
            UpdateBBKind::UpdateNetworking {
                additive: _additive,
            } => if let Some(_data) = boss_bar_resource.boss_bars.get_mut(&uuid) {},
        }
        updated.push((uuid, update_kind));
    }

    for (writer, mut bossbar_sender) in player_query.iter_mut() {
        for (uuid, update_kind) in &updated {
            match update_kind {
                UpdateBBKind::Add { .. } => continue,
                UpdateBBKind::Remove => {
                    if bossbar_sender.0.contains(&uuid.as_u128()) {
                        bossbar_sender.remove(*uuid);
                        let packet = BossbarPacket::remove_bossbar(uuid.as_u128());
                        writer.send_packet_ref(&packet).unwrap_or_else(|_| {
                            warn!("Failed to send Bossbar Packet to player");
                        });
                    }
                    boss_bar_resource.remove_bar(*uuid);
                    continue;
                }
                UpdateBBKind::UpdateNetworking { additive } => {
                    if *additive {
                        let bar = boss_bar_resource.boss_bars.get(uuid).unwrap();
                        bossbar_sender.add(*uuid);
                        let packet = BossbarPacket::add_bossbar(
                            uuid.as_u128(),
                            bar.title.clone(),
                            bar.health / bar.max,
                            bar.color.discriminant().into(),
                            bar.dividers.discriminant().into(),
                            bar.flags.get(),
                        );
                        writer.send_packet_ref(&packet).unwrap_or_else(|_| {
                            warn!("Failed to send Bossbar Packet to player");
                        });
                    } else {
                        if bossbar_sender.0.contains(&uuid.as_u128()) {
                            bossbar_sender.remove(*uuid);
                            let packet = BossbarPacket::remove_bossbar(uuid.as_u128());
                            writer.send_packet_ref(&packet).unwrap_or_else(|_| {
                                warn!("Failed to send Bossbar Packet to player");
                            });
                        }
                    }
                    continue;
                }
                _ => {}
            }

            if !bossbar_sender.0.contains(&uuid.as_u128()) {
                continue;
            }

            let packet = match update_kind {
                UpdateBBKind::UpdateHealth {
                    new_health,
                    new_max,
                } => BossbarPacket::update_health(uuid.as_u128(), *new_health, *new_max),
                UpdateBBKind::UpdateTitle { title } => {
                    BossbarPacket::update_title(uuid.as_u128(), title.clone())
                }
                UpdateBBKind::UpdateStyle { color, dividers } => BossbarPacket::update_style(
                    uuid.as_u128(),
                    color.discriminant().into(),
                    dividers.discriminant().into(),
                ),
                UpdateBBKind::UpdateFlags { flags } => {
                    BossbarPacket::update_flags(uuid.as_u128(), flags.get())
                }
                _ => unreachable!(),
            };

            writer.send_packet_ref(&packet).unwrap_or_else(|_| {
                warn!("Failed to send Bossbar Packet to player");
            });
        }
    }
}
