use crate::{
    CommandContext, Suggestion,
    arg::{CommandArgument, ParserResult, utils::parser_error},
};

use super::PrimitiveArgument;

pub enum BossbarSetOptions {
    Color(BossbarCommandColor),
    Name(String),
    Players(Vec<String>),
    Style((String, String)),
    Value(f32),
    Max(f32),
}

pub enum BossbarCommandColor {
    Blue,
    Green,
    Pink,
    Purple,
    Red,
    White,
    Yellow,
}

impl CommandArgument for BossbarSetOptions {
    fn parse(ctx: &mut CommandContext) -> ParserResult<Self> {
        let str = ctx.input.read_string();

        let value = match &*str.to_lowercase() {
            "color" => {
                let color = match &*ctx.input.read_string().to_lowercase() {
                    "blue" => BossbarCommandColor::Blue,
                    "green" => BossbarCommandColor::Green,
                    "pink" => BossbarCommandColor::Pink,
                    "purple" => BossbarCommandColor::Purple,
                    "red" => BossbarCommandColor::Red,
                    "yellow" => BossbarCommandColor::Yellow,
                    _ => BossbarCommandColor::White,
                };
                BossbarSetOptions::Color(color)
            }
            "name" => BossbarSetOptions::Name(ctx.input.read_string()),
            "players" => {
                let players = ctx.input.read_string();
                BossbarSetOptions::Players(vec![players])
            }
            "style" => {
                let style = ctx.input.read_string();
                let divider = ctx.input.read_string();
                BossbarSetOptions::Style((style, divider))
            }
            "value" => {
                let v = ctx
                    .input
                    .read_string()
                    .parse::<f32>()
                    .map_err(|_| parser_error("invalid float for value"))?;
                BossbarSetOptions::Value(v)
            }
            "max" => {
                let v = ctx
                    .input
                    .read_string()
                    .parse::<f32>()
                    .map_err(|_| parser_error("invalid float for max"))?;
                BossbarSetOptions::Max(v)
            }
            _ => return Err(parser_error(&format!("invalid option: {str}"))),
        };

        Ok(value)
    }

    fn primitive() -> PrimitiveArgument {
        PrimitiveArgument::greedy()
    }

    fn suggest(ctx: &mut CommandContext) -> Vec<Suggestion> {
        let str = ctx.input.read_string();

        let players: Vec<String> = ctx
            .state
            .players
            .player_list
            .iter()
            .map(|e| e.value().1.clone())
            .collect();
        let mut player_refs: Vec<&str> = players.iter().map(|s| s.as_str()).collect();
        player_refs.append(&mut vec!["@e", "@a", "@r"]);

        let suggestions: &[&str] = match str.to_lowercase().as_str() {
            "color" => &["blue", "green", "pink", "purple", "red", "white", "yellow"],
            "style" => &[
                "notched_6",
                "notched_10",
                "notched_12",
                "notched_20",
                "progress",
            ],
            "players" => &player_refs,
            _ => &["color", "name", "players", "style", "value", "max"],
        };

        suggestions.iter().map(Suggestion::of).collect()
    }
}
