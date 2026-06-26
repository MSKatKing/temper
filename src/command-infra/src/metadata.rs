use crate::{CommandReader, ParseError};

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
pub enum ParserProperties {
    String(StringMode),
    Integer(IntegerProperties),
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
pub struct ArgumentSpec {
    pub parser: ParserKind,
    pub properties: Option<ParserProperties>,
    pub suggestions: Option<&'static str>,
}

impl ArgumentSpec {
    pub const fn new(parser: ParserKind) -> Self {
        Self {
            parser,
            properties: None,
            suggestions: None,
        }
    }

    pub const fn with_properties(parser: ParserKind, properties: ParserProperties) -> ArgumentSpec {
        Self {
            parser,
            properties: Some(properties),
            suggestions: None,
        }
    }

    pub const fn with_suggestions(mut self, suggestions: &'static str) -> ArgumentSpec {
        self.suggestions = Some(suggestions);
        self
    }
}

pub trait CommandArg: Sized {
    type Raw<'a>;

    const KIND: ArgKind = ArgKind::Normal;

    fn recognize<'a>(reader: &mut CommandReader<'a>) -> Result<Self::Raw<'a>, ParseError>;

    fn parse(raw: Self::Raw<'_>) -> Result<Self, ParseError>;

    fn argument_spec() -> ArgumentSpec;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandPathSegment {
    Literal(&'static str),
    Argument {
        name: &'static str,
        spec: ArgumentSpec,
    },
}

impl CommandPathSegment {
    pub const fn literal(name: &'static str) -> Self {
        Self::Literal(name)
    }

    pub const fn argument(name: &'static str, spec: ArgumentSpec) -> Self {
        Self::Argument { name, spec }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPath {
    pub root: &'static str,
    pub segments: Vec<CommandPathSegment>,
}

impl CommandPath {
    pub fn new(root: &'static str, segments: Vec<CommandPathSegment>) -> Self {
        Self { root, segments }
    }
}

pub trait CommandSpec: Sized {
    const NAME: &'static str;

    fn parse_reader(reader: &mut CommandReader<'_>) -> Result<Self, ParseError>;

    fn paths() -> Vec<CommandPath>;

    fn parse(input: &str) -> Result<Self, ParseError> {
        let mut reader = CommandReader::new(input);
        Self::parse_reader(&mut reader)
    }
}

pub trait SubcommandSpec: Sized {
    fn parse_reader(reader: &mut CommandReader<'_>) -> Result<Self, ParseError>;

    fn segments() -> Vec<Vec<CommandPathSegment>>;
}
