//! # Commands in Temper (strap in chucklefucks)
//!
//! Defining commands is pretty simple, simple define an enum (or struct) and slap the derive macro on it:
//! ```
//! # use temper_command_infra::args::{EntityArg, PositionArg};
//!
//! #[derive(Command)]
//! #[command("example")]
//! enum ExampleCommand {
//!     WithEntity{ entity: EntityArg},
//!     WithoutEntity,
//!     #[subcommand("sub")]
//!     Subcommand(ExampleSingleSubcommand),
//!     
//! }
//!
//! #[derive(Command)]
//! #[command(subcommand)]
//! struct ExampleSingleSubcommand {
//!     WithPos: PositionArg
//! }
//! ```
//! and then implementing the [crate::CommandHandler] trait on it:
//! ```
//! # use bevy_ecs::prelude::{Query, Res};
//! # use temper_command_infra::CommandHandler;
//! # use temper_components::player::position::Position;
//! # use temper_components::player::rotation::Rotation;
//!
//! # use temper_state::GlobalState;
//!
//! impl CommandHandler for ExampleCommand {
//!     // These can be whatever ECS params you need
//!     type SystemParam<'w, 's> = (
//!         Res<'w, GlobalState>,
//!         Query<'w, 's, &'static Position, &'static Rotation>,
//!     );
//!
//!     fn handle(self, source: CommandSource, params: &mut Self::SystemParam<'_, '_>) {
//!         let (state: GlobalState, query: Query<'_, '_, &'static Position, &'static Rotation>) = params;
//!         match self {
//!             WithEntity{ entity } => {
//!                 // do something with the entity name/uuid/selector the player gave in the first argument
//!             },
//!             WithoutEntity => {
//!                 // do something without an entity
//!             },
//!             Subcommand(subcommand) => {
//!                 // do something with the subcommand
//!                 let sub_entity = subcommand.WithPos;
//!                 // do something with the position the player gave in the first argument of the subcommand
//!             }
//!         }
//!     }
//!
//!     // Optional error handler method
//!     fn handle_parse_error(
//!         source: CommandSource,
//!         error: ParseError,
//!         _params: &mut Self::SystemParam<'_, '_>,
//!     ) {}
//! }
//! ```
//! The entire system revolves around the [crate::CommandHandler] trait, which is implemented on the
//! command enum/struct. Under the hood the derive macro will generate all the code needed to have
//! the command wired into the ECS, provide suggestions and parse the arguments. All you need to do
//! is define what arguments a command needs and what it does with those args.
//!
//! There are several attribute macros available including literal args:
//! ```
//! #[derive(Command)]
//! #[command("example")]
//! enum ExampleCommand {
//!     #[literal("literal")]
//!     LiteralCommand,
//! }
//! ```
//! that skip the hassle of parsing and verifying an argument when you only allow a specific set of
//! options, aliases on both commands and literals:
//! ```
//! #[derive(Command)]
//! #[command("example", aliases = ["ex", "exmpl"])]
//! enum ExampleCommand {
//!     #[literal("literal", aliases = ["lit", "l"])]
//!     LiteralCommand,
//! }
//! ```
//! and permissions:
//! ```
//! #[derive(Command)]
//! #[command("example", permission = Permissions::ExamplePermission)]
//! enum ExampleCommand {
//!     #[literal("literal", permission = Permissions::LiteralExamplePermission)]
//!     LiteralCommand,
//! }
//! ```
//! (Note that this only limits what gets suggested to the client, you still need to verify
//! permissions in handlers to prevent manually typed commands being run when they shouldn't)
//!
//! This is the general gist of using commands with existing argument types, check out [crate::args]
//! for how to make your own argument types.

pub mod args;
pub mod ecs;
pub mod error;
pub mod graph;
pub mod metadata;
pub mod reader;
pub mod suggestions;

pub use ctor;
pub use ecs::{
    CommandDispatched, CommandHandler, CommandRegistry, CommandSource, PlayerCommandGraph,
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
    EntityProperties, IntegerProperties, ParserKind, ParserProperties, ResourceProperties,
    StringMode, SuggestionProviderKind,
};
pub use reader::{Checkpoint, CommandReader};
pub use suggestions::{
    SuggestionInput, command_arg_suggestion_id, register_command_arg_suggestions,
    suggest_command_arg,
};
pub use temper_permissions::Permissions;
