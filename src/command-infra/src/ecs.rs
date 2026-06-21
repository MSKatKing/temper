use bevy_ecs::prelude::{Component, Entity, Message, Resource};

use crate::{CommandGraph, CommandPath, CommandSpec};

#[derive(Clone, Debug)]
pub struct RegisteredCommand {
    pub name: &'static str,
    pub paths: Vec<CommandPath>,
}

impl RegisteredCommand {
    pub fn of<C: CommandSpec>() -> Self {
        Self {
            name: C::NAME,
            paths: C::paths(),
        }
    }
}

#[derive(Default, Resource)]
pub struct CommandRegistry {
    commands: Vec<RegisteredCommand>,
}

impl CommandRegistry {
    pub fn register<C: CommandSpec>(&mut self) {
        self.commands.push(RegisteredCommand::of::<C>());
    }

    pub fn register_command(&mut self, command: RegisteredCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[RegisteredCommand] {
        &self.commands
    }

    pub fn paths_for_player(&self, _player: Entity) -> Vec<CommandPath> {
        self.commands
            .iter()
            .flat_map(|command| command.paths.iter().cloned())
            .collect()
    }

    pub fn build_graph_for_player(&self, player: Entity) -> CommandGraph {
        CommandGraph::from_paths(&self.paths_for_player(player))
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct PlayerCommandGraph {
    pub graph: CommandGraph,
    pub version: u64,
}

impl PlayerCommandGraph {
    pub fn new(graph: CommandGraph) -> Self {
        Self { graph, version: 0 }
    }

    pub fn next(graph: CommandGraph, previous: Option<&PlayerCommandGraph>) -> Self {
        Self {
            graph,
            version: previous.map(|graph| graph.version + 1).unwrap_or(0),
        }
    }
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RebuildCommandGraph {
    pub player: Entity,
}
