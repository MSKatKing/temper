pub mod args;
pub mod ecs;
pub mod error;
pub mod graph;
pub mod metadata;
pub mod reader;
pub mod suggestions;

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
    EntityProperties, IntegerProperties, ParserKind, ParserProperties, StringMode,
    SuggestionProviderKind,
};
pub use reader::{Checkpoint, CommandReader};
pub use suggestions::{
    SuggestionInput, command_arg_suggestion_id, register_command_arg_suggestions,
    suggest_command_arg,
};
pub use temper_permissions::Permissions;
