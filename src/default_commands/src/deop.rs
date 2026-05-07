use bevy_ecs::prelude::{Entity, Query};
use temper_commands::arg::entities::EntityArgument;
use temper_commands::Sender;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_macros::command;
use temper_permissions::player::PlayerPermission;
use temper_permissions::Permissions::ALL;
use temper_text::TextComponent;

#[command("deop")]
pub fn deop_command(
    #[arg] target: EntityArgument,
    #[sender] sender: Sender,
    args: (
        Query<(Entity, &Identity, Option<&PlayerMarker>)>,
        Query<&mut PlayerPermission>,
    ),
) {
    let (query, mut permissions_query) = args;
    let entities = target.resolve(query.into_iter());

    let is_permitted = match sender {
        Sender::Player(entity) => {
            if let Ok(player_perm) = permissions_query.get(entity) {
                player_perm.can(temper_permissions::Permissions::DeOp)
            } else {
                false
            }
        }
        _ => true, // Non-player senders are always permitted
    };

    if !is_permitted {
        sender.send_message(
            "You don't have permission to use this command.".into(),
            false,
        );
        return;
    }

    for entity in entities {
        if let Ok(mut player_permission) = permissions_query.get_mut(entity) {
            player_permission.remove_permission(&ALL);
            temper_core::mq::queue(
                TextComponent::from("You have been de-opped".to_string()),
                false,
                entity,
            );
        }
    }
}
