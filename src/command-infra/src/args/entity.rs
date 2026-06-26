use std::ops::Deref;

use bevy_ecs::entity::Entity;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;
use uuid::Uuid;

use crate::{ArgumentSpec, CommandArg, CommandReader, ParseError, ParserKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityArg(String);

impl Deref for EntityArg {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EntityArg {
    pub fn resolve<'a>(
        &self,
        iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
    ) -> Vec<Entity> {
        match &**self {
            "@e" => iter.map(|(entity, _, _)| entity).collect(),
            "@a" => iter
                .filter_map(|(entity, _, marker)| marker.map(|_| entity))
                .collect(),
            "@r" => iter
                .filter_map(|(entity, _, marker)| marker.map(|_| entity))
                .take(1)
                .collect(),
            raw => {
                let uuid = Uuid::parse_str(raw).ok();

                iter.filter_map(|(entity, identity, _)| {
                    if identity.name.as_deref() == Some(raw) || Some(identity.uuid) == uuid {
                        Some(entity)
                    } else {
                        None
                    }
                })
                .collect()
            }
        }
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
        ArgumentSpec::new(ParserKind::Entity).with_suggestions("minecraft:ask_server")
    }
}
