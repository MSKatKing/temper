use bevy_ecs::world::World;

use crate::SuggestionInput;
use crate::{CommandReader, ParseError};
use temper_permissions::Permissions;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgKind {
    Normal,
    GreedyTail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StringMode {
    Word,
    Quotable,
    Greedy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegerProperties {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityProperties {
    pub single: bool,
    pub players_only: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserProperties {
    String(StringMode),
    Integer(IntegerProperties),
    Entity(EntityProperties),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserKind {
    Word,
    Integer,
    String,
    Position,
    Entity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuggestionProviderKind {
    None,
    Client(&'static str),
    Server,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArgumentSpec {
    pub parser: ParserKind,
    pub properties: Option<ParserProperties>,
    pub protocol_suggestions: Option<&'static str>,
    pub server_suggestions: Option<&'static str>,
}

impl ArgumentSpec {
    pub const fn new(parser: ParserKind) -> Self {
        Self {
            parser,
            properties: None,
            protocol_suggestions: None,
            server_suggestions: None,
        }
    }

    pub const fn with_properties(parser: ParserKind, properties: ParserProperties) -> ArgumentSpec {
        Self {
            parser,
            properties: Some(properties),
            protocol_suggestions: None,
            server_suggestions: None,
        }
    }

    pub const fn with_suggestions(mut self, suggestions: &'static str) -> ArgumentSpec {
        self.protocol_suggestions = Some(suggestions);
        self
    }

    pub const fn with_protocol_suggestions(mut self, suggestions: &'static str) -> ArgumentSpec {
        self.protocol_suggestions = Some(suggestions);
        self
    }

    pub const fn with_server_suggestions(mut self, suggestions: &'static str) -> ArgumentSpec {
        self.server_suggestions = Some(suggestions);
        self
    }

    pub const fn entity(single: bool, players_only: bool) -> ArgumentSpec {
        Self::with_properties(
            ParserKind::Entity,
            ParserProperties::Entity(EntityProperties {
                single,
                players_only,
            }),
        )
    }
}

pub trait CommandArg: Sized {
    type Raw<'a>;

    const KIND: ArgKind = ArgKind::Normal;
    const SUGGESTIONS: SuggestionProviderKind;

    fn recognize<'a>(reader: &mut CommandReader<'a>) -> Result<Self::Raw<'a>, ParseError>;

    fn parse(raw: Self::Raw<'_>) -> Result<Self, ParseError>;

    fn argument_spec() -> ArgumentSpec;

    fn suggest(_input: SuggestionInput<'_>, _world: &mut World) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandPathSegment {
    Literal {
        name: &'static str,
        permission: Option<Permissions>,
    },
    Argument {
        name: &'static str,
        spec: ArgumentSpec,
        permission: Option<Permissions>,
    },
}

impl CommandPathSegment {
    pub const fn literal(name: &'static str) -> Self {
        Self::Literal {
            name,
            permission: None,
        }
    }

    pub const fn argument(name: &'static str, spec: ArgumentSpec) -> Self {
        Self::Argument {
            name,
            spec,
            permission: None,
        }
    }

    pub const fn with_permission(mut self, permission: Permissions) -> Self {
        match &mut self {
            Self::Literal {
                permission: segment_permission,
                ..
            }
            | Self::Argument {
                permission: segment_permission,
                ..
            } => *segment_permission = Some(permission),
        }
        self
    }

    pub const fn permission(&self) -> Option<Permissions> {
        match self {
            Self::Literal { permission, .. } | Self::Argument { permission, .. } => *permission,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPath {
    pub root: &'static str,
    pub permission: Option<Permissions>,
    pub segments: Vec<CommandPathSegment>,
}

impl CommandPath {
    pub fn new(root: &'static str, segments: Vec<CommandPathSegment>) -> Self {
        Self {
            root,
            permission: None,
            segments,
        }
    }

    pub fn with_permission(mut self, permission: Option<Permissions>) -> Self {
        self.permission = permission;
        self
    }

    pub fn with_root(mut self, root: &'static str) -> Self {
        self.root = root;
        self
    }

    pub fn is_allowed_by(&self, can_use: impl Fn(Permissions) -> bool) -> bool {
        self.permission.map_or(true, &can_use)
            && self
                .segments
                .iter()
                .all(|segment| segment.permission().map_or(true, &can_use))
    }
}

pub trait CommandSpec: Sized {
    const NAME: &'static str;

    fn aliases() -> &'static [&'static str] {
        &[]
    }

    fn permission() -> Option<Permissions> {
        None
    }

    fn parse_reader(reader: &mut CommandReader<'_>) -> Result<Self, ParseError>;

    fn parse_reader_with_permissions(
        reader: &mut CommandReader<'_>,
        _can_use: &dyn Fn(Permissions) -> bool,
    ) -> Result<Self, ParseError> {
        Self::parse_reader(reader)
    }

    fn paths() -> Vec<CommandPath>;

    fn parse(input: &str) -> Result<Self, ParseError> {
        let mut reader = CommandReader::new(input);
        Self::parse_reader(&mut reader)
    }
}

pub trait SubcommandSpec: Sized {
    fn parse_reader(reader: &mut CommandReader<'_>) -> Result<Self, ParseError>;

    fn parse_reader_with_permissions(
        reader: &mut CommandReader<'_>,
        _can_use: &dyn Fn(Permissions) -> bool,
    ) -> Result<Self, ParseError> {
        Self::parse_reader(reader)
    }

    fn segments() -> Vec<Vec<CommandPathSegment>>;
}
