use std::ops::Deref;

use crate::{ArgumentSpec, CommandArg, CommandReader, ParseError, ParserKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityArg(String);

impl Deref for EntityArg {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CommandArg for EntityArg {
    type Raw<'a> = &'a str;

    fn recognize<'a>(reader: &mut CommandReader<'a>) -> Result<Self::Raw<'a>, ParseError> {
        let cursor = reader.cursor();
        let span = reader.read_word_span()?;

        if span.is_empty() {
            Err(ParseError::expected(cursor, "entity"))
        } else {
            Ok(span)
        }
    }

    fn parse(raw: Self::Raw<'_>) -> Result<Self, ParseError> {
        Ok(Self(raw.to_string()))
    }

    fn argument_spec() -> ArgumentSpec {
        ArgumentSpec::new(ParserKind::Entity).with_suggestions("ask_server")
    }
}
