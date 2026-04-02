use bevy_ecs::prelude::*;
use temper_commands::Sender;
use temper_components::entity_identity::Identity;
use temper_macros::command;
use temper_text::TextComponent;

#[command("nested")]
fn nested_command(#[sender] sender: Sender, query: Query<&Identity>) {
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
        TextComponent::from(format!("{} executed /nested", username)),
        false,
    );
}

#[command("nested nested")]
fn nested_nested_command(#[sender] sender: Sender, query: Query<&Identity>) {
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
        TextComponent::from(format!("{} executed /nested nested", username)),
        false,
    );
}
