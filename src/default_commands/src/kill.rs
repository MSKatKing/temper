use bevy_ecs::prelude::{Entity, MessageWriter, Query};
use temper_command_infra::CommandSource::Player;
use temper_command_infra::args::EntityArg;
use temper_command_infra::{CommandHandler, CommandSource};
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_macros::Command;
use temper_messages::destroy_entity::DestroyEntity;
use temper_permissions::player::PlayerPermission;

#[derive(Command)]
#[command("kill")]
enum KillCommand {
    SelfTarget,
    OtherTarget { target: EntityArg },
}

impl CommandHandler for KillCommand {
    type SystemParam<'w, 's> = (
        Query<'w, 's, (Entity, &'static Identity, Option<&'static PlayerMarker>)>,
        MessageWriter<'w, DestroyEntity>,
        Query<'w, 's, &'static PlayerPermission>,
    );

    fn handle(self, source: CommandSource, params: &mut Self::SystemParam<'_, '_>) {
        let &mut (query, ref mut writer, permissions) = params;

        let is_permitted = match source {
            Player(entity) => {
                if let Ok(player_perm) = permissions.get(entity) {
                    player_perm.can(temper_permissions::Permissions::Kill)
                } else {
                    false
                }
            }
            _ => true,
        };

        if !is_permitted {
            source.send_message("You don't have permission to use this command.".into());
            return;
        }

        let selected_entities = match self {
            KillCommand::SelfTarget => {
                if let Player(entity) = source {
                    vec![entity]
                } else {
                    source
                        .send_message("The server cannot target itself with this command.".into());
                    vec![]
                }
            }
            KillCommand::OtherTarget { target } => target.resolve(query.iter()),
        };

        selected_entities.iter().for_each(|e| {
            writer.write(DestroyEntity(*e));
        });

        source.send_message(
            format!(
                "Killed {} entities (excluding players).",
                selected_entities.len()
            )
            .into(),
        );
    }
}
