use bevy_ecs::prelude::Query;
use temper_commands::Sender;
use temper_macros::command;
use temper_permissions::player::PlayerPermission;
use temper_text::TextComponentBuilder;

#[command("permissions")]
fn test_command(#[sender] sender: Sender, permissions_query: Query<&PlayerPermission>) {
    let Sender::Player(entity) = sender else {
        sender.send_message("This command can only be used by players.".into(), false);
        return;
    };

    let player_permissions = match permissions_query.get(entity) {
        Ok(perms) => perms,
        Err(_) => {
            sender.send_message("Could not find your player entity.".into(), false);
            return;
        }
    };

    let mut response = String::new();
    response.push_str("Individual Permissions:\n");
    for (perm, access) in &player_permissions.permissions {
        response.push_str(&format!("- {:?}: {:?}\n", perm, access));
    }

    sender.send_message(TextComponentBuilder::new(response).build(), false);
}
