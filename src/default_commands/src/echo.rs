use bevy_ecs::prelude::*;
use temper_commands::{arg::primitive::string::GreedyString, Sender};
use temper_components::entity_identity::Identity;
use temper_macros::command;
use temper_text::{TextComponent, TextComponentBuilder};

#[command("echo")]
fn test_command(#[arg] message: GreedyString, #[sender] sender: Sender, query: Query<&Identity>) {
    let username = match sender {
        Sender::Server => "Server".to_string(),
        Sender::Player(entity) => query
            .get(entity)
            .expect("sender does not exist")
            .name
            .as_ref()
            .expect("No Player Name")
            .clone(),
    };

    sender.send_message(
        TextComponentBuilder::new(format!("{} said: ", username))
            .extra(TextComponent::from(message.clone()))
            .build(),
        false,
    );
}
