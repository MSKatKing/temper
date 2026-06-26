use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{MessageWriter, Query};
use temper_commands::Sender;
use temper_commands::Sender::Player;
use temper_commands::arg::entities::EntityArgument;
use temper_commands::arg::position::CommandPosition;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_macros::command;
use temper_messages::teleport_entity::TeleportEntity;

#[command("tp pos")]
fn tp_command(
    #[sender] sender: Sender,
    #[arg] pos: CommandPosition,
    args: (Query<(&Rotation, &Position)>, MessageWriter<TeleportEntity>),
) {
    let (mut query, mut tp_player_msg) = args;
    let Player(entity) = sender else {
        sender.send_message("This command can only be used by players.".into(), false);
        return;
    };

    let Ok((rot, position)) = query.get_mut(entity) else {
        sender.send_message("Could not find your player entity.".into(), false);
        return;
    };
    let resolved_pos = pos.resolve(position);

    tp_player_msg.write(TeleportEntity::new(entity, resolved_pos, *rot));

    sender.send_message(format!("Teleported to ({}).", resolved_pos).into(), false);
}

#[command("tp entity")]
fn tp_to_command(
    #[sender] sender: Sender,
    #[arg] target: EntityArgument,
    args: (
        Query<(&Rotation, &Position)>,
        MessageWriter<TeleportEntity>,
        Query<(Entity, &Identity, Option<&PlayerMarker>)>,
    ),
) {
    let (query, mut tp_player_msg, resolve_q) = args;

    let resolved_targets = target.resolve(resolve_q.iter());

    if resolved_targets.len() != 1 {
        sender.send_message(
            "You must specify exactly one target to teleport to.".into(),
            false,
        );
        return;
    } else if matches!(sender, Sender::Server) {
        sender.send_message("This command can only be used by players.".into(), false);
        return;
    }

    let target_entity = resolved_targets.first().expect("Checked above; qed");

    let Player(sender_e) = sender else {
        unreachable!();
    };

    let Ok([(sender_rot, _), (_, target_pos)]) = query.get_many([sender_e, *target_entity]) else {
        sender.send_message("Could not find player entities.".into(), false);
        return;
    };

    tp_player_msg.write(TeleportEntity::new(sender_e, *target_pos, *sender_rot));

    sender.send_message(
        format!("Teleported to the entity at {}.", target_pos).into(),
        false,
    );
}
