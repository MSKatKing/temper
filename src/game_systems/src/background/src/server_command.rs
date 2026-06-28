use bevy_ecs::change_detection::Res;
use bevy_ecs::message::MessageWriter;
use std::sync::Arc;
use temper_command_infra::{CommandRegistry, CommandSource, NewCommandDispatched};
use temper_commands::Sender;
use temper_commands::messages::{CommandDispatched, ResolvedCommandDispatched};
use temper_commands::resolve::resolve;
use temper_resources::server_command_rx::ServerCommandReceiver;
use temper_state::GlobalStateResource;
use tracing::error;

pub fn handle(
    receiver: Res<ServerCommandReceiver>,
    registry: Res<CommandRegistry>,
    mut new_dispatch_msgs: MessageWriter<NewCommandDispatched>,
    mut dispatch_msgs: MessageWriter<CommandDispatched>,
    mut resolved_dispatch_msgs: MessageWriter<ResolvedCommandDispatched>,
    state: Res<GlobalStateResource>,
) {
    for command in receiver.0.try_iter() {
        if registry.owns_input(&command) {
            new_dispatch_msgs.write(NewCommandDispatched {
                input: Arc::from(command),
                source: CommandSource::Server,
            });
            continue;
        }

        let sender = Sender::Server;
        dispatch_msgs.write(CommandDispatched {
            command: command.clone(),
            sender,
        });

        let resolved = resolve(command, sender, state.0.clone());
        match resolved {
            Err(err) => {
                error!("Error resolving server command: {}", err.to_plain_text());
            }

            Ok((command, ctx)) => {
                resolved_dispatch_msgs.write(ResolvedCommandDispatched {
                    command,
                    ctx,
                    sender,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::message::MessageRegistry;
    use bevy_ecs::prelude::{Messages, Schedule, World};
    use std::sync::Arc;
    use temper_command_infra::{
        CommandPath, CommandRegistry, CommandSource, NewCommandDispatched, RegisteredCommand,
    };
    use temper_commands::messages::{CommandDispatched, ResolvedCommandDispatched};
    use temper_resources::server_command_rx::ServerCommandReceiver;
    use temper_state::create_test_state;

    use super::handle;

    #[test]
    fn tui_server_commands_owned_by_new_registry_emit_new_dispatch_messages() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        sender.send("stop".to_string()).unwrap();

        let (state, _temp_dir) = create_test_state();
        let mut world = World::new();
        MessageRegistry::register_message::<NewCommandDispatched>(&mut world);
        MessageRegistry::register_message::<CommandDispatched>(&mut world);
        MessageRegistry::register_message::<ResolvedCommandDispatched>(&mut world);
        world.insert_resource(ServerCommandReceiver(receiver));
        world.insert_resource(state);
        world.insert_resource(new_command_registry());

        let mut schedule = Schedule::default();
        schedule.add_systems(handle);
        schedule.run(&mut world);

        let new_message_resource = world.resource::<Messages<NewCommandDispatched>>();
        let mut new_cursor = new_message_resource.get_cursor();
        let new_messages = new_cursor
            .read(new_message_resource)
            .cloned()
            .collect::<Vec<_>>();
        let old_message_resource = world.resource::<Messages<CommandDispatched>>();
        let mut old_cursor = old_message_resource.get_cursor();
        let old_messages = old_cursor.read(old_message_resource).collect::<Vec<_>>();

        assert_eq!(new_messages.len(), 1);
        assert_eq!(new_messages[0].input, Arc::from("stop"));
        assert_eq!(new_messages[0].source, CommandSource::Server);
        assert!(old_messages.is_empty());
    }

    fn new_command_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::default();
        registry.register_command(RegisteredCommand {
            name: "stop",
            aliases: &[],
            permission: None,
            paths: vec![CommandPath::new("stop", Vec::new())],
        });
        registry
    }
}
