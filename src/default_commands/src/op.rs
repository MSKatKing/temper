use bevy_ecs::prelude::{Entity, Query};
use temper_command_infra::CommandSource::Player;
use temper_command_infra::args::EntitiesArg;
use temper_command_infra::{CommandHandler, CommandResult, CommandSource};
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use temper_macros::Command;
use temper_permissions::Access::Allow;
use temper_permissions::Permissions::{ALL, Op};
use temper_permissions::player::PlayerPermission;
use temper_text::TextComponent;

#[derive(Debug, Command)]
#[command("op")]
struct OpCommand {
    target: EntitiesArg,
}

impl CommandHandler for OpCommand {
    type SystemParam<'w, 's> = (
        Query<'w, 's, (Entity, &'static Identity, Option<&'static PlayerMarker>)>,
        Query<'w, 's, &'static mut PlayerPermission>,
    );

    fn handle(
        self,
        source: CommandSource,
        params: &mut Self::SystemParam<'_, '_>,
    ) -> CommandResult {
        let (entities, permissions) = params;

        let is_permitted = match source {
            Player(entity) => permissions
                .get(entity)
                .is_ok_and(|player_perm| player_perm.can(Op)),
            CommandSource::Server => true,
        };

        if !is_permitted {
            return Err("You don't have permission to use this command.".into());
        }

        for entity in self.target.resolve(entities.iter()) {
            if let Ok(mut player_permission) = permissions.get_mut(entity) {
                player_permission.set_permission(ALL, Allow);
                source.send_message(TextComponent::from("You have been opped".to_string()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::system::SystemState;
    use temper_command_infra::{CommandHandler, CommandSource, CommandSpec};
    use temper_components::entity_identity::Identity;
    use temper_components::player::player_marker::PlayerMarker;
    use temper_permissions::Access::Deny;
    use temper_permissions::Permissions::{ALL, Op};
    use temper_permissions::player::PlayerPermission;

    use super::OpCommand;

    #[test]
    fn op_parses_target_arg() {
        let command = OpCommand::parse("Steve").unwrap();

        assert_eq!(&*command.target, "Steve");
    }

    #[test]
    fn op_grants_all_permission_to_resolved_target() {
        let mut world = bevy_ecs::prelude::World::new();
        let target = world
            .spawn((identity("Steve"), PlayerMarker, PlayerPermission::new()))
            .id();
        let command = OpCommand::parse("Steve").unwrap();

        let mut params =
            SystemState::<<OpCommand as CommandHandler>::SystemParam<'_, '_>>::new(&mut world);
        let mut system_params = params.get_mut(&mut world);
        command
            .handle(CommandSource::Server, &mut system_params)
            .unwrap();
        params.apply(&mut world);

        assert!(
            world
                .get::<PlayerPermission>(target)
                .unwrap()
                .can(temper_permissions::Permissions::Kill)
        );
    }

    #[test]
    fn op_requires_op_permission_for_player_senders() {
        let mut world = bevy_ecs::prelude::World::new();
        let sender = world
            .spawn((
                identity("Sender"),
                PlayerMarker,
                player_permission(Op, Deny),
            ))
            .id();
        let target = world
            .spawn((identity("Steve"), PlayerMarker, PlayerPermission::new()))
            .id();
        let command = OpCommand::parse("Steve").unwrap();

        let mut params =
            SystemState::<<OpCommand as CommandHandler>::SystemParam<'_, '_>>::new(&mut world);
        let mut system_params = params.get_mut(&mut world);
        assert!(
            command
                .handle(CommandSource::Player(sender), &mut system_params)
                .is_err()
        );
        params.apply(&mut world);

        assert!(!world.get::<PlayerPermission>(target).unwrap().can(ALL));
    }

    fn identity(name: &str) -> Identity {
        Identity {
            entity_id: 0,
            uuid: uuid::Uuid::new_v4(),
            name: Some(name.to_string()),
        }
    }

    fn player_permission(
        permission: temper_permissions::Permissions,
        access: temper_permissions::Access,
    ) -> PlayerPermission {
        let mut player_permission = PlayerPermission::new();
        player_permission.set_permission(permission, access);
        player_permission
    }
}
