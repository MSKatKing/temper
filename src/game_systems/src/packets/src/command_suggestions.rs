use std::{collections::HashSet, sync::Arc};

use bevy_ecs::prelude::*;
use temper_codec::net_types::{
    length_prefixed_vec::LengthPrefixedVec, prefixed_optional::PrefixedOptional, var_int::VarInt,
};
use temper_command_infra::{CommandPathSegment, CommandRegistry, ParserKind};
use temper_commands::{Command, CommandContext, CommandInput, ROOT_COMMAND, Sender};
use temper_net_runtime::connection::StreamWriter;
use temper_permissions::player::PlayerPermission;
use temper_protocol::CommandSuggestionRequestReceiver;
use temper_protocol::outgoing::command_suggestions::{CommandSuggestionsPacket, Match};
use temper_state::{GlobalState, GlobalStateResource};
use tracing::error;

fn find_command(input: String) -> Option<Arc<Command>> {
    let mut input = input;
    if input.starts_with("/") {
        input.remove(0);
    }

    if let Some(command) = temper_commands::infrastructure::get_command_by_name(&input) {
        return Some(command);
    }

    if let Some(command) = temper_commands::infrastructure::find_command(&input) {
        return Some(command);
    }

    while !input.is_empty() {
        // remove the last word and retry
        if let Some(pos) = input.rfind(char::is_whitespace) {
            input.truncate(pos);

            if let Some(command) = temper_commands::infrastructure::get_command_by_name(&input) {
                return Some(command);
            }

            if let Some(command) = temper_commands::infrastructure::find_command(&input) {
                return Some(command);
            }
        } else {
            break; // string does not have any further words, meaning it's just whitespace?
        }
    }

    None
}

fn create_ctx(
    input: String,
    command: Option<Arc<Command>>,
    sender: Sender,
    state: GlobalState,
) -> CommandContext {
    let input = input
        .strip_prefix(command.clone().map(|c| c.name).unwrap_or_default())
        .unwrap_or(&input)
        .trim_start();

    let input = CommandInput::of(input.to_string());
    CommandContext {
        input: input.clone(),
        command: command.unwrap_or(ROOT_COMMAND.clone()),
        sender,
        state,
    }
}

pub fn handle(
    receiver: Res<CommandSuggestionRequestReceiver>,
    query: Query<&StreamWriter>,
    permissions: Query<&PlayerPermission>,
    registry: Res<CommandRegistry>,
    state: Res<GlobalStateResource>,
) {
    for (request, entity) in receiver.0.try_iter() {
        if !state.0.players.is_connected(entity) {
            return;
        }

        let input = request.input;
        let Ok(writer) = query.get(entity) else {
            continue;
        };

        if let Some(response) =
            new_command_suggestions(&input, &registry, &state, permissions.get(entity).ok())
        {
            send_suggestions(writer, request.transaction_id, response);
            continue;
        }

        let command = find_command(input.clone());
        let command_arg = input
            .clone()
            .strip_prefix(&format!(
                "/{} ",
                command.clone().map(|c| c.name).unwrap_or_default()
            ))
            .unwrap_or(&input)
            .to_string();
        let mut ctx = create_ctx(
            command_arg.clone(),
            command.clone(),
            Sender::Player(entity),
            state.0.clone(),
        );
        let command_arg = command_arg.clone();
        let tokens = command_arg.split(" ").collect::<Vec<&str>>();
        let Some(current_token) = tokens.last() else {
            return; // whitespace
        };

        let mut suggestions = Vec::new();

        if let Some(command) = command {
            for arg in command.args.clone() {
                let arg_suggestions = (arg.suggester)(&mut ctx);
                ctx.input.skip_whitespace(u32::MAX, true);
                if !ctx.input.has_remaining_input() {
                    suggestions = arg_suggestions;
                    break;
                }
            }
        }

        let start = input.len() - current_token.len();
        let length = current_token.len();

        send_suggestions(
            writer,
            request.transaction_id,
            SuggestionResponse {
                start,
                length,
                matches: suggestions
                    .into_iter()
                    .filter(|sug| {
                        sug.content
                            .to_lowercase()
                            .starts_with(&current_token.to_lowercase())
                    })
                    .map(|sug| Match {
                        content: sug.content,
                        tooltip: PrefixedOptional::new(sug.tooltip),
                    })
                    .collect(),
            },
        );
    }
}

struct SuggestionResponse {
    start: usize,
    length: usize,
    matches: Vec<Match>,
}

fn new_command_suggestions(
    input: &str,
    registry: &CommandRegistry,
    state: &GlobalStateResource,
    permissions: Option<&PlayerPermission>,
) -> Option<SuggestionResponse> {
    let command_input = input.strip_prefix('/').unwrap_or(input);
    let root_end = command_input
        .find(char::is_whitespace)
        .unwrap_or(command_input.len());
    let root = &command_input[..root_end];
    let command = registry
        .commands()
        .iter()
        .find(|command| command.matches_root(root))?;
    let rest = command_input[root_end..].trim_start();
    let current_token = current_token(rest);
    let completed_tokens = completed_tokens(rest);
    let current_token_lower = current_token.to_lowercase();
    let mut seen = HashSet::new();

    let matches = command
        .paths
        .iter()
        .filter(|path| path.root == root)
        .filter(|path| {
            path.is_allowed_by(|permission| permissions.is_some_and(|p| p.can(permission)))
        })
        .filter_map(|path| candidate_segment(&path.segments, &completed_tokens))
        .filter_map(|segment| segment_suggestions(segment, state))
        .flatten()
        .filter(|suggestion| suggestion.to_lowercase().starts_with(&current_token_lower))
        .filter(|suggestion| seen.insert(suggestion.clone()))
        .map(|content| Match {
            content,
            tooltip: PrefixedOptional::new(None),
        })
        .collect();

    Some(SuggestionResponse {
        start: input.len() - current_token.len(),
        length: current_token.len(),
        matches,
    })
}

fn current_token(input: &str) -> &str {
    if input.ends_with(char::is_whitespace) {
        ""
    } else {
        input.split_whitespace().last().unwrap_or("")
    }
}

fn completed_tokens(input: &str) -> Vec<&str> {
    let mut tokens = input.split_whitespace().collect::<Vec<_>>();

    if !input.ends_with(char::is_whitespace) {
        tokens.pop();
    }

    tokens
}

fn candidate_segment<'a>(
    segments: &'a [CommandPathSegment],
    completed_tokens: &[&str],
) -> Option<&'a CommandPathSegment> {
    let mut token_index = 0;

    for segment in segments {
        let Some(token) = completed_tokens.get(token_index) else {
            return Some(segment);
        };

        if !segment_accepts_token(segment, token) {
            return None;
        }

        token_index += segment_width(segment);
    }

    None
}

fn segment_accepts_token(segment: &CommandPathSegment, token: &str) -> bool {
    match segment {
        CommandPathSegment::Literal { name, .. } => name == &token,
        CommandPathSegment::Argument { spec, .. } => match spec.parser {
            ParserKind::Integer => token.parse::<i32>().is_ok(),
            ParserKind::Position => is_coordinate_token(token),
            ParserKind::Word | ParserKind::String | ParserKind::Entity => !token.is_empty(),
        },
    }
}

fn segment_width(segment: &CommandPathSegment) -> usize {
    match segment {
        CommandPathSegment::Argument { spec, .. } if spec.parser == ParserKind::Position => 3,
        _ => 1,
    }
}

fn is_coordinate_token(token: &str) -> bool {
    if let Some(relative) = token.strip_prefix('~') {
        relative.is_empty() || relative.parse::<f64>().is_ok()
    } else {
        token.parse::<f64>().is_ok()
    }
}

fn segment_suggestions(
    segment: &CommandPathSegment,
    state: &GlobalStateResource,
) -> Option<Vec<String>> {
    match segment {
        CommandPathSegment::Literal { name, .. } => Some(vec![(*name).to_string()]),
        CommandPathSegment::Argument { spec, .. } if is_ask_server(spec.suggestions) => {
            match spec.parser {
                ParserKind::Entity => Some(entity_suggestions(state)),
                _ => Some(Vec::new()),
            }
        }
        _ => None,
    }
}

fn is_ask_server(suggestions: Option<&str>) -> bool {
    matches!(suggestions, Some("ask_server" | "minecraft:ask_server"))
}

fn entity_suggestions(state: &GlobalStateResource) -> Vec<String> {
    let mut suggestions = vec!["@e".to_string(), "@r".to_string(), "@a".to_string()];
    suggestions.extend(
        state
            .0
            .players
            .player_list
            .iter()
            .map(|player| player.value().1.clone()),
    );
    suggestions
}

fn send_suggestions(writer: &StreamWriter, transaction_id: VarInt, response: SuggestionResponse) {
    if let Err(e) = writer.send_packet(CommandSuggestionsPacket {
        transaction_id,
        matches: LengthPrefixedVec::new(response.matches),
        length: VarInt::new(response.length as i32),
        start: VarInt::new(response.start as i32),
    }) {
        error!("failed sending command suggestions to player: {e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::entity::Entity;
    use temper_command_infra::{ArgumentSpec, CommandPath, RegisteredCommand};
    use temper_state::create_test_state;

    fn registry() -> CommandRegistry {
        let mut registry = CommandRegistry::default();
        registry.register_command(RegisteredCommand {
            name: "tp",
            aliases: &[],
            permission: None,
            paths: vec![
                CommandPath::new(
                    "tp",
                    vec![CommandPathSegment::argument(
                        "location",
                        ArgumentSpec::new(ParserKind::Position),
                    )],
                ),
                CommandPath::new(
                    "tp",
                    vec![CommandPathSegment::argument(
                        "destination",
                        ArgumentSpec::new(ParserKind::Entity)
                            .with_suggestions("minecraft:ask_server"),
                    )],
                ),
                CommandPath::new(
                    "tp",
                    vec![
                        CommandPathSegment::argument(
                            "target",
                            ArgumentSpec::new(ParserKind::Entity)
                                .with_suggestions("minecraft:ask_server"),
                        ),
                        CommandPathSegment::argument(
                            "location",
                            ArgumentSpec::new(ParserKind::Position),
                        ),
                    ],
                ),
                CommandPath::new(
                    "tp",
                    vec![
                        CommandPathSegment::argument(
                            "target",
                            ArgumentSpec::new(ParserKind::Entity)
                                .with_suggestions("minecraft:ask_server"),
                        ),
                        CommandPathSegment::argument(
                            "destination",
                            ArgumentSpec::new(ParserKind::Entity)
                                .with_suggestions("minecraft:ask_server"),
                        ),
                    ],
                ),
            ],
        });
        registry
    }

    #[test]
    fn new_command_suggestions_include_entities_for_first_tp_arg() {
        let (state, _temp_dir) = create_test_state();
        state
            .0
            .players
            .player_list
            .insert(Entity::PLACEHOLDER, (0, "Alex".to_string()));

        let suggestions = new_command_suggestions("/tp ", &registry(), &state, None).unwrap();
        let matches = suggestions
            .matches
            .iter()
            .map(|suggestion| suggestion.content.as_str())
            .collect::<Vec<_>>();

        assert_eq!(suggestions.start, 4);
        assert_eq!(suggestions.length, 0);
        assert!(matches.contains(&"@a"));
        assert!(matches.contains(&"Alex"));
    }

    #[test]
    fn new_command_suggestions_use_current_token_range() {
        let (state, _temp_dir) = create_test_state();
        state
            .0
            .players
            .player_list
            .insert(Entity::PLACEHOLDER, (0, "Alex".to_string()));

        let suggestions =
            new_command_suggestions("/tp Steve A", &registry(), &state, None).unwrap();
        let matches = suggestions
            .matches
            .iter()
            .map(|suggestion| suggestion.content.as_str())
            .collect::<Vec<_>>();

        assert_eq!(suggestions.start, 10);
        assert_eq!(suggestions.length, 1);
        assert_eq!(matches, vec!["Alex"]);
    }

    #[test]
    fn old_command_suggestions_do_not_handle_new_roots() {
        let (state, _temp_dir) = create_test_state();

        assert!(new_command_suggestions("/tp Unknown", &registry(), &state, None).is_some());
        assert!(new_command_suggestions("/time set ", &registry(), &state, None).is_none());
    }
}
