use bevy_ecs::entity::Entity;
use temper_command_infra::args::{EntityArg, PositionArg};
use temper_command_infra::{CommandRegistry, CommandSpec, static_commands};
use temper_macros::Command;

#[derive(Debug, PartialEq, Command)]
#[command("tp")]
enum TpCommand {
    TpToPos { location: PositionArg },
    TpToEntity { destination: EntityArg },
}

#[test]
fn registry_builds_graph_from_registered_commands() {
    let mut registry = CommandRegistry::default();
    registry.register::<TpCommand>();

    let graph = registry.build_graph_for_player(Entity::PLACEHOLDER);
    let root = &graph.nodes[graph.root_idx];
    let tp_idx = root.children[0];
    let tp = &graph.nodes[tp_idx];

    assert_eq!(registry.commands().len(), 1);
    assert_eq!(tp.name.as_deref(), Some("tp"));
    assert_eq!(tp.children.len(), 2);
    assert!(
        tp.children
            .iter()
            .all(|child| graph.nodes[*child].executable)
    );
}

#[test]
fn derived_commands_register_static_metadata() {
    let commands = static_commands();

    assert!(
        commands
            .iter()
            .any(|command| command.name == TpCommand::NAME),
        "derived command was not registered in static command metadata"
    );
}
