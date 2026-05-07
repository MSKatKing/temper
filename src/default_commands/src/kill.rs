use bevy_ecs::prelude::{Entity, MessageWriter, Query};
use temper_commands::arg::entities::EntityArgument;
use temper_commands::Sender;
use temper_commands::Sender::Player;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_macros::command;
use temper_messages::destroy_entity::DestroyEntity;
use temper_permissions::player::PlayerPermission;

#[command("kill")]
fn kill_command(
    #[sender] sender: Sender,
    #[arg] entity_argument: EntityArgument,
    args: (
        Query<(Entity, &Identity, Option<&PlayerMarker>)>,
        MessageWriter<DestroyEntity>,
        Query<&PlayerPermission>,
    ),
) {
    let (query, mut writer, permissions) = args;

    let is_permitted = match sender {
        Player(entity) => {
            if let Ok(player_perm) = permissions.get(entity) {
                player_perm.can(temper_permissions::Permissions::Kill)
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

    let selected_entities = entity_argument.resolve(query.iter());

    selected_entities.iter().for_each(|e| {
        writer.write(DestroyEntity(*e));
    });

    sender.send_message(
        format!(
            "Killed {} entities (excluding players).",
            selected_entities.len()
        )
        .into(),
        false,
    );
}
