use bevy_ecs::prelude::*;
use std::sync::Arc;
use temper_command_infra::{CommandRegistry, CommandSource, NewCommandDispatched};
use temper_commands::{
    Sender,
    messages::{CommandDispatched, ResolvedCommandDispatched},
    resolve,
};
use temper_components::entity_identity::Identity;
use temper_core::mq;
use temper_protocol::ChatCommandPacketReceiver;
use temper_state::GlobalStateResource;
use tracing::info;

pub fn handle(
    receiver: Res<ChatCommandPacketReceiver>,
    registry: Res<CommandRegistry>,
    mut new_dispatch_msgs: MessageWriter<NewCommandDispatched>,
    mut dispatch_msgs: MessageWriter<CommandDispatched>,
    mut resolved_dispatch_msgs: MessageWriter<ResolvedCommandDispatched>,
    state: Res<GlobalStateResource>,
    query: Query<&Identity>,
) {
    for (event, entity) in receiver.0.try_iter() {
        if registry.owns_input(&event.command) {
            new_dispatch_msgs.write(NewCommandDispatched {
                input: Arc::from(event.command.clone()),
                source: CommandSource::Player(entity),
            });
            continue;
        }

        let sender = Sender::Player(entity);
        dispatch_msgs.write(CommandDispatched {
            command: event.command.clone(),
            sender,
        });

        let resolved = resolve::resolve(event.command.clone(), sender, state.0.clone());
        match resolved {
            Err(err) => {
                mq::queue(*err, false, entity);
            }

            Ok((command, ctx)) => {
                let Ok(player_id) = query.get(entity) else {
                    continue;
                };
                info!(
                    "Player {} executed command: /{}",
                    player_id.name.as_ref().expect("No Player Name"),
                    event.command
                );
                resolved_dispatch_msgs.write(ResolvedCommandDispatched {
                    command,
                    ctx,
                    sender,
                });
            }
        }
    }
}
