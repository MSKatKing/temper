use bevy_ecs::prelude::Query;
use ionic_commands::arg::primitive::string::GreedyString;
use ionic_commands::Sender;
use ionic_components::player::player_identity::PlayerIdentity;
use ionic_core::mq;
use ionic_macros::command;
use ionic_net_runtime::connection::StreamWriter;

#[command("say")]
fn say_command(
    #[sender] sender: Sender,
    #[arg] message: GreedyString,
    query: Query<(&StreamWriter, &PlayerIdentity)>,
) {
    let full_message = match sender {
        Sender::Server => format!("<Server> {}", message.as_str()),
        Sender::Player(entity) => {
            let player_identity = query.get(entity).expect("sender does not exist").1;
            format!("<{}> {}", player_identity.username, message.as_str())
        }
    };

    mq::broadcast(full_message.into(), false);
}
