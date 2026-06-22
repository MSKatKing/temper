use bevy_ecs::prelude::Query;
use temper_command_infra::CommandSource::*;
use temper_command_infra::args::GreedyStringArg;
use temper_command_infra::{CommandHandler, CommandSource};
use temper_components::entity_identity::Identity;
use temper_core::mq;
use temper_macros::Command;
use temper_text::{TextComponent, TextComponentBuilder};
use tracing::info;

#[derive(Command)]
#[command("echo")]
struct EchoCommand {
    message: GreedyStringArg,
}

impl CommandHandler for EchoCommand {
    type SystemParam<'w, 's> = Query<'w, 's, &'static Identity>;

    fn handle(self, source: CommandSource, identities: &mut Self::SystemParam<'_, '_>) {
        let username = match source {
            Server => "Server".to_string(),
            Player(entity) => identities
                .get(entity)
                .expect("sender does not exist")
                .name
                .as_ref()
                .expect("No Player Name")
                .clone(),
        };

        let message = TextComponentBuilder::new(format!("{username} said: "))
            .extra(TextComponent::from(self.message.to_string()))
            .build();

        match source {
            Player(entity) => mq::queue(message, false, entity),
            Server => info!("{}", message.to_plain_text()),
        }
    }
}
