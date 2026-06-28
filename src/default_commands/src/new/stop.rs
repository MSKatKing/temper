use std::sync::atomic::Ordering::Relaxed;

use bevy_ecs::prelude::Res;
use temper_command_infra::CommandSource::*;
use temper_command_infra::{CommandHandler, CommandSource};
use temper_core::mq;
use temper_macros::Command;
use temper_state::GlobalStateResource;
use tracing::info;

#[derive(Command)]
#[command(name = "stop", aliases = ["quit"])]
struct StopCommand;

impl CommandHandler for StopCommand {
    type SystemParam<'w, 's> = Res<'w, GlobalStateResource>;

    fn handle(self, source: CommandSource, state: &mut Self::SystemParam<'_, '_>) {
        if let Player(player) = source {
            mq::queue(
                "This command can only be used by the server.".into(),
                false,
                player,
            );
            return;
        }

        info!("Shutting down server...");
        state
            .0
            .world
            .sync()
            .expect("Failed to sync world before shutdown");
        state.0.shut_down.store(true, Relaxed);
    }
}
