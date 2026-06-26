pub mod args;
pub mod ecs;
pub mod error;
pub mod graph;
pub mod metadata;
pub mod reader;

pub use ctor;
pub use ecs::{
    CommandHandler, CommandRegistry, CommandSource, NewCommandDispatched, PlayerCommandGraph,
    RebuildCommandGraph, RegisteredCommand,
};
pub use ecs::{
    add_system, dispatch_command, register_command_systems, register_static_command,
    send_parse_error, static_commands,
};
pub use error::ParseError;
pub use graph::{CommandGraph, CommandNode, CommandNodeKind};
pub use metadata::SubcommandSpec;
pub use metadata::{
    ArgKind, ArgumentSpec, CommandArg, CommandPath, CommandPathSegment, CommandSpec,
    IntegerProperties, ParserKind, ParserProperties, StringMode,
};
pub use reader::{Checkpoint, CommandReader};
pub use temper_permissions::Permissions;
