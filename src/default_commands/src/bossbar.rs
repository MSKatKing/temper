use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Query;
use bevy_ecs::system::ResMut;
use rand::prelude::IteratorRandom;
use temper_commands::arg::bossbar_set::{BossbarCommandColor, BossbarSetOptions};
use temper_commands::arg::primitive::string::{GreedyString, QuotableString};
use temper_commands::Sender;
use temper_components::entity_identity::Identity;
use temper_components::player::bossbar_sender::BossbarSender;
use temper_components::player::player_marker::PlayerMarker;
use temper_macros::command;
use temper_resources::bossbar::{BossBarData, BossBarResource, BossbarColor, BossbarDividers};
use temper_text::ClickEvent::CopyToClipboard;
use temper_text::HoverEvent::ShowText;
use temper_text::{TextComponent, TextComponentBuilder};
use uuid::Uuid;

#[command("bossbar add")]
fn add_bossbar_command(
    #[arg] name: GreedyString,
    #[sender] sender: Sender,
    args: ResMut<BossBarResource>,
) {
    let uuid = args.add_bar(BossBarData::new(
        TextComponent::from(name.as_str()),
        0.0,
        100.0,
        BossbarColor::Pink,
    ));

    sender.send_message(
        TextComponent::from(format!("Created bossbar with uuid: {}", uuid))
            .click_event(CopyToClipboard(String::from(uuid)))
            .hover_event(ShowText(TextComponent::from(String::from(uuid)).into())),
        false,
    );
}

#[command("bossbar get")]
fn get_bossbar_command(
    #[arg] uuid: GreedyString,
    #[sender] sender: Sender,
    args: ResMut<BossBarResource>,
) {
    let uuid_res = Uuid::parse_str(uuid.as_ref());

    if uuid_res.is_err() {
        sender.send_message(TextComponentBuilder::new("Not an UUID!").build(), false);
        return;
    }

    let uuid_str = uuid_res.unwrap();
    let bossbar = args.boss_bars.get(&uuid_str);

    if let Some(bossbar) = bossbar {
        sender.send_message(
            TextComponentBuilder::new("Bossbar: ")
                .extra(TextComponent::from(format!("{}", bossbar)))
                .build(),
            false,
        );
    } else {
        sender.send_message(
            TextComponentBuilder::new("Bossbar doesn't exist for uuid: ")
                .extra(TextComponent::from(format!("{}", uuid_str)))
                .build(),
            false,
        );
    }
}

#[command("bossbar list")]
fn get_all_bossbar_command(#[sender] sender: Sender, args: ResMut<BossBarResource>) {
    let bossbars: Vec<_> = args.boss_bars.keys().cloned().collect();

    if bossbars.is_empty() {
        sender.send_message(
            TextComponentBuilder::new("No bossbars exist.").build(),
            false,
        );
    } else {
        for uuid in &bossbars {
            sender.send_message(
                TextComponentBuilder::new("Bossbar: ")
                    .extra(
                        TextComponent::from(format!("{}", uuid))
                            .click_event(CopyToClipboard(uuid.to_string()))
                            .hover_event(ShowText(TextComponent::from(uuid.to_string()).into())),
                    )
                    .build(),
                false,
            );
        }
    }
}

#[command("bossbar remove")]
fn remove_bossbar_command(
    #[arg] uuid: GreedyString,
    #[sender] sender: Sender,
    args: ResMut<BossBarResource>,
) {
    let uuid_res = Uuid::parse_str(uuid.as_ref());

    if uuid_res.is_err() {
        sender.send_message(TextComponentBuilder::new("Not an UUID!").build(), false);
        return;
    }

    let uuid_str = uuid_res.unwrap();
    let bossbar = args.boss_bars.get(&uuid_str);

    if let Some(_) = bossbar {
        args.remove_bar(uuid_str);

        sender.send_message(TextComponentBuilder::new("removed bossbar").build(), false);
    } else {
        sender.send_message(
            TextComponentBuilder::new("Bossbar doesn't exist for uuid: ")
                .extra(TextComponent::from(format!("{}", uuid_str)))
                .build(),
            false,
        );
    }
}

#[command("bossbar set")]
fn set_bossbar_command(
    #[arg] uuid: QuotableString,
    #[arg] option: BossbarSetOptions,
    #[sender] sender: Sender,
    args: (
        ResMut<BossBarResource>,
        Query<(Entity, &Identity, &mut BossbarSender, Option<&PlayerMarker>)>,
    ),
) {
    let boss_res = args.0;
    let mut query = args.1;

    let uuid_res = Uuid::parse_str(uuid.as_ref());

    if uuid_res.is_err() {
        sender.send_message(TextComponentBuilder::new("Not an UUID!").build(), false);
        return;
    }

    let uuid_str = uuid_res.unwrap();
    let bossbar = boss_res.boss_bars.get(&uuid_str);

    if let Some(bossbar) = bossbar {
        match option {
            BossbarSetOptions::Color(color) => {
                let divider = &bossbar.dividers;
                let color = match color {
                    BossbarCommandColor::Blue => BossbarColor::Blue,
                    BossbarCommandColor::Green => BossbarColor::Green,
                    BossbarCommandColor::Pink => BossbarColor::Pink,
                    BossbarCommandColor::Purple => BossbarColor::Purple,
                    BossbarCommandColor::Red => BossbarColor::Red,
                    BossbarCommandColor::White => BossbarColor::White,
                    BossbarCommandColor::Yellow => BossbarColor::Yellow,
                };

                boss_res.update_style(uuid_str, color, divider.clone());
            }
            BossbarSetOptions::Name(title) => {
                boss_res.update_title(uuid_str, TextComponent::from(title.as_str()));
            }
            BossbarSetOptions::Players(players) => {
                let option_value = players.first().map(|s| s.as_str()).unwrap_or("");

                match option_value {
                    "@e" | "@a" => {
                        query
                            .iter_mut()
                            .map(|(_, _, sender, _)| sender)
                            .for_each(|mut sender| {
                                sender.add(uuid_str);
                            })
                    }
                    "@r" => {
                        let mut rng = rand::rng();
                        let sender_ret = query
                            .iter_mut()
                            .filter_map(|(_, _, sender, _)| Some(sender))
                            .choose(&mut rng);

                        sender_ret.unwrap().add(uuid_str);
                    }
                    _ => {
                        for (_, identity, mut sender, marker) in query {
                            if marker.is_some()
                                && identity
                                    .name
                                    .as_ref()
                                    .map(|n| n.eq_ignore_ascii_case(option_value))
                                    .unwrap_or(false)
                            {
                                if sender.0.contains(&uuid_str.as_u128()) {
                                    boss_res.queue_networking(uuid_str, false);
                                } else {
                                    sender.add(uuid_str);
                                    boss_res.queue_networking(uuid_str, true);
                                }
                            }
                        }
                    }
                }
            }
            BossbarSetOptions::Style((_, divider_str)) => {
                let divider = match divider_str.as_str() {
                    "notched_6" => BossbarDividers::SixNotches,
                    "notched_10" => BossbarDividers::TenNotches,
                    "notched_12" => BossbarDividers::TwelveNotches,
                    "notched_20" => BossbarDividers::TwentyNotches,
                    "progress" | _ => BossbarDividers::None,
                };

                let color = &bossbar.color;
                boss_res.update_style(uuid_str, color.clone(), divider);
            }
            BossbarSetOptions::Value(value) => {
                let max = boss_res.boss_bars.get(&uuid_str).unwrap().max;
                boss_res.update_health(uuid_str, value, max);
            }
            BossbarSetOptions::Max(value) => {
                let health = boss_res.boss_bars.get(&uuid_str).unwrap().health;
                boss_res.update_health(uuid_str, health, value);
            }
        }
    } else {
        sender.send_message(
            TextComponentBuilder::new("Bossbar doesn't exist for uuid: ")
                .extra(TextComponent::from(format!("{}", uuid_str)))
                .build(),
            false,
        );
    }
}
