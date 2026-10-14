use bevy_ecs::prelude::{Entity, MessageWriter, Query, World};
use bevy_ecs::system::SystemParamItem;
use temper_command_infra::args::EntityArg;
use temper_command_infra::{
    ArgumentSpec, CommandArg, CommandHandler, CommandReader, CommandResult, CommandSource,
    ParseError, ParserKind, SuggestionInput, SuggestionProviderKind,
};
use temper_components::entity_identity::Identity;
use temper_components::player::dimension::CurrentDimension;
use temper_components::player::player_marker::PlayerMarker;
use temper_core::dimension::Dimension;
use temper_macros::Command;
use temper_messages::dimension_change::DimensionChange;
use temper_text::{NamedColor, TextComponentBuilder};

#[derive(Command)]
#[command("dimension")]
enum DimensionCommand {
    #[literal("query")]
    Query { entity: EntityArg },
    #[literal("set")]
    Set {
        entity: EntityArg,
        dimension: DimensionArg,
    },
}

struct DimensionArg(Dimension);

impl CommandArg for DimensionArg {
    type Raw<'a> = &'a str;
    const SUGGESTIONS: SuggestionProviderKind = SuggestionProviderKind::None;

    fn recognize<'a>(reader: &mut CommandReader<'a>) -> Result<Self::Raw<'a>, ParseError> {
        reader.read_word_span()
    }

    fn parse(raw: Self::Raw<'_>) -> Result<Self, ParseError> {
        match raw {
            "overworld" => Ok(DimensionArg(Dimension::Overworld)),
            "nether" => Ok(DimensionArg(Dimension::Nether)),
            "end" => Ok(DimensionArg(Dimension::End)),
            _ => Err(ParseError::expected(0, "a dimension specifier")),
        }
    }

    fn argument_spec() -> ArgumentSpec {
        ArgumentSpec::new(ParserKind::Word)
    }

    fn suggest(input: SuggestionInput<'_>, _world: &mut World) -> Vec<String> {
        const DIMENSIONS: &[&str] = &["overworld", "nether", "end"]; // todo: replace with a world query of some sort

        DIMENSIONS
            .iter()
            .filter(|dim| dim.starts_with(input.current_token))
            .map(|dim| dim.to_string())
            .collect()
    }
}

impl CommandHandler for DimensionCommand {
    type SystemParam<'w, 's> = (
        Query<'w, 's, (Entity, &'static Identity, Option<&'static PlayerMarker>)>,
        Query<'w, 's, &'static CurrentDimension>,
        MessageWriter<'w, DimensionChange>,
    );

    fn handle(
        self,
        source: CommandSource,
        params: &mut SystemParamItem<'_, '_, Self::SystemParam<'_, '_>>,
    ) -> CommandResult {
        let (entities, dimensions, change_dim_writer) = params;

        let entity = match &self {
            Self::Query { entity } => entity,
            Self::Set { entity, .. } => entity,
        };

        let resolved = entity.resolve(entities.iter());

        if resolved.is_empty() {
            source.send_message(
                TextComponentBuilder::new("No entities selected.")
                    .color(NamedColor::Red)
                    .build(),
            );

            return Ok(());
        }

        match self {
            Self::Query { .. } => {
                for entity in resolved {
                    let (entity, identity, _) =
                        entities.get(entity).expect("should be present if resolved");
                    let identifier = format!(
                        "entity '{}' ({})",
                        identity.name.as_ref().map(String::as_ref).unwrap_or("None"),
                        identity.uuid
                    );

                    let dimension = match dimensions.get(entity) {
                        Ok(d) => d,
                        Err(_) => {
                            source.send_message(
                                TextComponentBuilder::new(format!(
                                    "Could not query dimension for {}",
                                    identifier
                                ))
                                .color(NamedColor::Red)
                                .build(),
                            );
                            continue;
                        }
                    };

                    source.send_message(
                        TextComponentBuilder::new(format!(
                            "Dimension of {} is '{:?}'",
                            identifier, dimension.0
                        ))
                        .build(),
                    );
                }
            }
            Self::Set { dimension, .. } => {
                for entity in resolved {
                    let (entity, identity, _) =
                        entities.get(entity).expect("should be present if resolved");
                    let identifier = format!(
                        "entity '{}' ({})",
                        identity.name.as_ref().map(String::as_ref).unwrap_or("None"),
                        identity.uuid
                    );

                    change_dim_writer.write(DimensionChange {
                        entity,
                        is_respawn: false,
                        new_dimension: dimension.0,
                        spawn_position: None, // will use the entities current position
                        spawn_rotation: None, // will use the entities current rotation
                    });
                    source.send_message(
                        TextComponentBuilder::new(format!(
                            "Changed dimension of {} to '{:?}'",
                            identifier, dimension.0
                        ))
                        .build(),
                    );
                }
            }
        }

        Ok(())
    }
}
