mod any_entity;
mod any_player;
mod entity_uuid;
mod player;
mod random_player;

use crate::arg::primitive::PrimitiveArgument;
use crate::arg::{CommandArgument, ParserResult};
use crate::{CommandContext, Suggestion};
use ::uuid::Uuid;
use bevy_ecs::prelude::Entity;
use temper_components::entity_identity::Identity;
use temper_components::player::player_marker::PlayerMarker;

/// Represents an entity argument in a command.
/// It can be a player name, UUID, or special selectors like @e, @p, @r, @a.
/// This won't get you an entity directly, use `resolve()` to get the entities.
///
/// # Example
/// ```ignore
/// # use temper_commands::arg::entities::EntityArgument;
/// # use temper_core::identity::entity_identity::EntityIdentity;
/// # use temper_core::identity::player_identity::Identity;
/// # use bevy_ecs::prelude::World;
///
/// fn my_command(query: Query<(Entity, Option<&EntityIdentity>, Option<&Identity>)>) {
///     let arg = EntityArgument::PlayerName("Steve".to_string());
///     let result = arg.resolve(query.iter());
///     assert_eq!(result, vec![entity]);
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum EntityArgument {
    PlayerName(String),
    Uuid(Uuid),
    AnyEntity,
    AnyPlayer,
    // NearestPlayer,
    RandomPlayer,
}

impl CommandArgument for EntityArgument {
    fn parse(ctx: &mut CommandContext) -> ParserResult<Self> {
        const PREFIXES: &[(&str, EntityArgument)] = &[
            ("@e", EntityArgument::AnyEntity),
            // ("@p", EntityArgument::NearestPlayer),
            ("@r", EntityArgument::RandomPlayer),
            ("@a", EntityArgument::AnyPlayer),
        ];
        let input = ctx.input.read_string();
        for (prefix, entity_type) in PREFIXES {
            if input == *prefix {
                return Ok(entity_type.clone());
            }
        }
        if input.len() == 36 && input.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            let uuid = Uuid::parse_str(&input)
                .map_err(|_| crate::arg::utils::parser_error("invalid UUID format"))?;
            Ok(EntityArgument::Uuid(uuid))
        } else {
            Ok(EntityArgument::PlayerName(input))
        }
    }

    fn primitive() -> PrimitiveArgument {
        PrimitiveArgument::word()
    }

    fn suggest(ctx: &mut CommandContext) -> Vec<Suggestion> {
        let mut suggestions = vec![
            Suggestion {
                content: "@e".to_string(),
                tooltip: Some(temper_nbt::NBT::new("Any Entity".into())),
            },
            // Suggestion {
            //     content: "@p".to_string(),
            //     tooltip: Some(temper_nbt::NBT::new("Nearest Player".into())),
            // },
            Suggestion {
                content: "@r".to_string(),
                tooltip: Some(temper_nbt::NBT::new("Random Player".into())),
            },
            Suggestion {
                content: "@a".to_string(),
                tooltip: Some(temper_nbt::NBT::new("All Players".into())),
            },
        ];
        let state = ctx.state.clone();
        for kv in &state.clone().players.player_list {
            let (_, (uuid, name)) = kv.pair();
            suggestions.push(Suggestion {
                content: name.clone(),
                tooltip: Some(temper_nbt::NBT::new(
                    Uuid::from_u128(*uuid)
                        .as_hyphenated()
                        .to_string()
                        .to_uppercase()
                        .into(),
                )),
            });
        }
        suggestions
    }
}

impl EntityArgument {
    pub fn resolve<'a>(
        &self,
        iter: impl Iterator<Item = (Entity, &'a Identity, Option<&'a PlayerMarker>)>,
    ) -> Vec<Entity> {
        match self {
            EntityArgument::PlayerName(name) => player::resolve_player_name(name.clone(), iter)
                .map(|e| vec![e])
                .unwrap_or_default(),
            EntityArgument::Uuid(uuid) => entity_uuid::resolve_uuid(*uuid, iter)
                .map(|e| vec![e])
                .unwrap_or_default(),
            EntityArgument::AnyEntity => any_entity::resolve_any_entity(iter),
            EntityArgument::AnyPlayer => any_player::resolve_any_player(iter),
            // EntityArgument::NearestPlayer => {
            //     // TODO: Figure this out
            //     vec![]
            // }
            EntityArgument::RandomPlayer => random_player::resolve_random_player(iter)
                .map(|e| vec![e])
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandInput, Sender};
    use bevy_ecs::prelude::World;
    use std::sync::Arc;
    use temper_components::entity_identity::Identity;
    use temper_state::create_test_state;

    #[test]
    fn test_parse_entity_argument() {
        let mut ctx = CommandContext {
            input: CommandInput {
                input: "Steve".to_string(),
                cursor: 0,
            },
            command: Arc::new(Command {
                name: "",
                args: vec![],
            }),
            sender: Sender::Server,
            state: create_test_state().0.0,
        };
        let arg = EntityArgument::parse(&mut ctx).unwrap();
        assert_eq!(arg, EntityArgument::PlayerName("Steve".to_string()));

        let mut ctx = CommandContext {
            input: CommandInput {
                input: "@e".to_string(),
                cursor: 0,
            },
            command: Arc::new(Command {
                name: "",
                args: vec![],
            }),
            sender: Sender::Server,
            state: create_test_state().0.0,
        };
        let arg = EntityArgument::parse(&mut ctx).unwrap();
        assert_eq!(arg, EntityArgument::AnyEntity);

        // let mut ctx = CommandContext {
        //     input: CommandInput {
        //         input: "@p".to_string(),
        //         cursor: 0,
        //     },
        //     command: Arc::new(Command {
        //         name: "",
        //         args: vec![],
        //     }),
        //     sender: Sender::Server,
        // };
        // let arg = EntityArgument::parse(&mut ctx).unwrap();
        // assert_eq!(arg, EntityArgument::NearestPlayer);

        let mut ctx = CommandContext {
            input: CommandInput {
                input: "@r".to_string(),
                cursor: 0,
            },
            command: Arc::new(Command {
                name: "",
                args: vec![],
            }),
            sender: Sender::Server,
            state: create_test_state().0.0,
        };
        let arg = EntityArgument::parse(&mut ctx).unwrap();
        assert_eq!(arg, EntityArgument::RandomPlayer);

        let mut ctx = CommandContext {
            input: CommandInput {
                input: "@a".to_string(),
                cursor: 0,
            },
            command: Arc::new(Command {
                name: "",
                args: vec![],
            }),
            sender: Sender::Server,
            state: create_test_state().0.0,
        };
        let arg = EntityArgument::parse(&mut ctx).unwrap();
        assert_eq!(arg, EntityArgument::AnyPlayer);

        let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
        let mut ctx = CommandContext {
            input: CommandInput {
                input: uuid_str.to_string(),
                cursor: 0,
            },
            command: Arc::new(Command {
                name: "",
                args: vec![],
            }),
            sender: Sender::Server,
            state: create_test_state().0.0,
        };
        let arg = EntityArgument::parse(&mut ctx).unwrap();
        assert_eq!(
            arg,
            EntityArgument::Uuid(Uuid::parse_str(uuid_str).unwrap())
        );
    }

    #[test]
    fn test_resolves_name() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Identity {
                    name: Some("Steve".to_string()),
                    uuid: Default::default(),
                    entity_id: 0,
                },
                PlayerMarker,
            ))
            .id();

        let arg = EntityArgument::PlayerName("Steve".to_string());
        let result = arg.resolve(
            world
                .query::<(Entity, &Identity, Option<&PlayerMarker>)>()
                .iter(&world),
        );
        assert_eq!(result, vec![entity]);
    }

    #[test]
    fn test_resolves_uuid() {
        let mut world = World::new();
        let test_uuid = Uuid::new_v4();
        let entity = world
            .spawn((Identity {
                entity_id: 0,
                uuid: test_uuid,
                name: None,
            },))
            .id();
        let arg = EntityArgument::Uuid(test_uuid);
        let result = arg.resolve(
            world
                .query::<(Entity, &Identity, Option<&PlayerMarker>)>()
                .iter(&world),
        );
        assert_eq!(result, vec![entity]);
    }

    #[test]
    fn test_resolves_any_entity() {
        let mut world = World::new();
        let entity1 = world
            .spawn(Identity {
                entity_id: 0,
                uuid: Uuid::new_v4(),
                name: None,
            })
            .id();
        let entity2 = world
            .spawn(Identity {
                entity_id: 1,
                uuid: Uuid::new_v4(),
                name: None,
            })
            .id();
        let arg = EntityArgument::AnyEntity;
        let result = arg.resolve(
            world
                .query::<(Entity, &Identity, Option<&PlayerMarker>)>()
                .iter(&world),
        );
        assert_eq!(result.len(), 2);
        assert!(result.contains(&entity1));
        assert!(result.contains(&entity2));
    }

    #[test]
    fn test_resolves_any_player() {
        let mut world = World::new();
        let entity1 = world
            .spawn((
                Identity {
                    name: Some("Steve".to_string()),
                    uuid: Uuid::new_v4(),
                    entity_id: 0,
                },
                PlayerMarker,
            ))
            .id();
        let entity2 = world
            .spawn((
                Identity {
                    name: Some("Alex".to_string()),
                    entity_id: 1,
                    uuid: Uuid::new_v4(),
                },
                PlayerMarker,
            ))
            .id();
        let non_player_entity = world
            .spawn((Identity {
                entity_id: 2,
                uuid: Uuid::new_v4(),
                name: None,
            },))
            .id();
        let arg = EntityArgument::AnyPlayer;
        let result = arg.resolve(
            world
                .query::<(Entity, &Identity, Option<&PlayerMarker>)>()
                .iter(&world),
        );
        assert_eq!(result.len(), 2);
        assert!(result.contains(&entity1));
        assert!(result.contains(&entity2));
        assert!(!result.contains(&non_player_entity));
    }

    #[test]
    fn resolves_random_player() {
        let mut world = World::new();
        let entity1 = world
            .spawn((
                Identity {
                    name: Some("Steve".to_string()),
                    uuid: Uuid::new_v4(),
                    entity_id: 0,
                },
                PlayerMarker,
            ))
            .id();
        let entity2 = world
            .spawn((
                Identity {
                    name: Some("Alex".to_string()),
                    entity_id: 1,
                    uuid: Uuid::new_v4(),
                },
                PlayerMarker,
            ))
            .id();
        let non_player_entity = world
            .spawn((Identity {
                entity_id: 2,
                uuid: Uuid::new_v4(),
                name: None,
            },))
            .id();
        let arg = EntityArgument::RandomPlayer;
        let result = arg.resolve(
            world
                .query::<(Entity, &Identity, Option<&PlayerMarker>)>()
                .iter(&world),
        );
        assert_eq!(result.len(), 1);
        assert!(result.contains(&entity1) || result.contains(&entity2));
        assert!(!result.contains(&non_player_entity));

        // Run the test 500 times to ensure randomness
        // Technically this could actually result in 500 identical results, but the odds of that are astronomically low (about 1 in 3.27e150)
        let mut results = vec![];
        for _ in 0..500 {
            let result = arg.resolve(
                world
                    .query::<(Entity, &Identity, Option<&PlayerMarker>)>()
                    .iter(&world),
            );
            assert_eq!(result.len(), 1);
            results.push(result[0]);
        }
        let unique_results: std::collections::HashSet<_> = results.into_iter().collect();
        assert_eq!(
            unique_results.len(),
            2,
            "Random player selection is not random enough"
        );
    }
}
