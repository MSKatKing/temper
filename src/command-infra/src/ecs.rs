use std::sync::{LazyLock, RwLock};
use std::{cell::RefCell, sync::Arc};

use bevy_ecs::prelude::{
    Component, Entity, IntoScheduleConfigs, Message, MessageReader, Resource, Schedule,
};
use bevy_ecs::schedule::ScheduleConfigs;
use bevy_ecs::system::{ScheduleSystem, SystemParam};
use temper_core::mq;
use temper_text::{NamedColor, TextComponentBuilder};
use tracing::info;

use crate::{CommandGraph, CommandPath, CommandSpec, ParseError};

static STATIC_COMMANDS: LazyLock<RwLock<Vec<RegisteredCommand>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

thread_local! {
    static SYSTEMS_TO_BE_REGISTERED: RefCell<Vec<ScheduleConfigs<ScheduleSystem>>> = RefCell::new(Vec::new());
}

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

pub fn register_static_command(command: RegisteredCommand) {
    if let Ok(mut commands) = STATIC_COMMANDS.write() {
        commands.push(command);
    }
}

pub fn static_commands() -> Vec<RegisteredCommand> {
    STATIC_COMMANDS
        .read()
        .map(|commands| commands.clone())
        .unwrap_or_default()
}

pub fn add_system<M>(system: impl IntoScheduleConfigs<ScheduleSystem, M>) {
    SYSTEMS_TO_BE_REGISTERED.with(|systems| {
        systems.borrow_mut().push(system.into_configs());
    });
}

pub fn register_command_systems(schedule: &mut Schedule) {
    SYSTEMS_TO_BE_REGISTERED.with(|systems| {
        let mut systems = systems.borrow_mut();
        while let Some(system) = systems.pop() {
            schedule.add_systems(system);
        }
    });
}

pub trait CommandHandler: CommandSpec + Sized + Send + Sync + 'static {
    type SystemParam<'w, 's>: SystemParam;

    fn handle<'w, 's>(self, source: CommandSource, params: &mut Self::SystemParam<'w, 's>);

    fn handle_parse_error<'w, 's>(
        source: CommandSource,
        error: ParseError,
        _params: &mut Self::SystemParam<'w, 's>,
    ) {
        send_parse_error(source, &error);
    }
}

pub fn send_parse_error(source: CommandSource, error: &ParseError) {
    let message = TextComponentBuilder::new(format!("failed parsing command: {}", error.message))
        .color(NamedColor::Red)
        .build();

    match source {
        CommandSource::Player(entity) => mq::queue(message, false, entity),
        CommandSource::Server => info!("{}", message.to_plain_text()),
    }
}

pub fn dispatch_command<C: CommandHandler>(
    mut commands: MessageReader<NewCommandDispatched>,
    mut params: C::SystemParam<'_, '_>,
) {
    for event in commands.read() {
        if command_root(&event.input) != Some(C::NAME) {
            continue;
        }

        let input = command_args(&event.input, C::NAME);
        match C::parse(input) {
            Ok(command) => command.handle(event.source, &mut params),
            Err(error) => C::handle_parse_error(event.source, error, &mut params),
        }
    }
}

#[derive(Default, Resource)]
pub struct CommandRegistry {
    commands: Vec<RegisteredCommand>,
}

impl CommandRegistry {
    pub fn from_static_commands() -> Self {
        Self {
            commands: static_commands(),
        }
    }

    pub fn register<C: CommandSpec>(&mut self) {
        self.commands.push(RegisteredCommand::of::<C>());
    }

    pub fn register_command(&mut self, command: RegisteredCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[RegisteredCommand] {
        &self.commands
    }

    pub fn owns_input(&self, input: &str) -> bool {
        command_root(input).is_some_and(|input_root| {
            self.commands
                .iter()
                .filter_map(|command| command_root(command.name))
                .any(|command_root| command_root == input_root)
        })
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

fn command_root(input: &str) -> Option<&str> {
    input.split_whitespace().next()
}

fn command_args<'a>(input: &'a str, root: &str) -> &'a str {
    input.strip_prefix(root).unwrap_or(input).trim_start()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandSource {
    Player(Entity),
    Server,
}

#[derive(Message, Clone, Debug)]
pub struct NewCommandDispatched {
    pub input: Arc<str>,
    pub source: CommandSource,
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
