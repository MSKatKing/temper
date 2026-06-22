use bevy_ecs::prelude::{Query, Res, ResMut};
use temper_commands::Sender;
use temper_commands::arg::primitive::int::Integer;
use temper_components::player::time::LastSentTimeUpdate;
use temper_macros::command;
use temper_resources::time::WorldTime;
use temper_text::TextComponent;

use temper_commands::{
    CommandContext, Suggestion,
    arg::{CommandArgument, ParserResult, primitive::PrimitiveArgument, utils::parser_error},
};

#[derive(Debug, Clone, Copy)]
struct TimeSetArg(u32);

fn parse_time_value(s: &str) -> Result<u32, &'static str> {
    if s.is_empty() {
        return Err("empty time value");
    }

    match s.to_lowercase().as_str() {
        "day" => return Ok(1000),
        "noon" => return Ok(6000),
        "night" => return Ok(13000),
        "midnight" => return Ok(18000),
        "dawn" => return Ok(0),
        "dusk" => return Ok(12000),
        _ => {}
    }

    let last_char = s.chars().last().unwrap();
    if last_char == 's' || last_char == 't'
    /* || last_char == 'd' */
    {
        let number_part = &s[..s.len() - 1];
        let val = number_part.parse::<u32>().map_err(|_| "invalid number")?;
        match last_char {
            /* 'd' => Ok(val * 24000), */
            's' => Ok(val * 20),
            't' => Ok(val),
            _ => unreachable!(),
        }
    } else {
        s.parse::<u32>().map_err(|_| "invalid time format")
    }
}

impl CommandArgument for TimeSetArg {
    fn parse(ctx: &mut CommandContext) -> ParserResult<Self> {
        let s = ctx.input.read_string();
        match parse_time_value(&s) {
            Ok(ticks) => Ok(TimeSetArg(ticks)),
            Err(_) => Err(parser_error(&format!("invalid time value: {}", s))),
        }
    }

    fn primitive() -> PrimitiveArgument {
        PrimitiveArgument::word()
    }

    fn suggest(ctx: &mut CommandContext) -> Vec<Suggestion> {
        ctx.input.skip_whitespace(u32::MAX, false);
        if !ctx.input.has_remaining_input() {
            return vec![
                Suggestion::of("day"),
                Suggestion::of("noon"),
                Suggestion::of("night"),
                Suggestion::of("midnight"),
            ];
        }

        let input = ctx.input.read_string();

        let presets = ["day", "noon", "night", "midnight", "dawn", "dusk"];
        let matching_presets: Vec<Suggestion> = presets
            .into_iter()
            .filter(|preset| preset.starts_with(&input.to_lowercase()))
            .map(Suggestion::of)
            .collect();

        if !matching_presets.is_empty() {
            return matching_presets;
        }

        if !input.is_empty() && input.chars().all(|c| c.is_ascii_digit()) {
            return vec![
                // Suggestion::of(format!("{}d", input)),
                Suggestion::of(format!("{}s", input)),
                Suggestion::of(format!("{}t", input)),
            ];
        }

        vec![]
    }
}

#[command("time set")]
fn time_set(
    #[sender] sender: Sender,
    #[arg] time: TimeSetArg,
    args: (ResMut<WorldTime>, Query<&mut LastSentTimeUpdate>),
) {
    let (mut world_time, mut query) = args;

    let ticks = time.0;
    let ticks_u16 = (ticks % 24000) as u16;
    world_time.set_time(ticks_u16);

    sender.send_message(
        TextComponent::from(format!(
            "Set the world time to {} ticks",
            world_time.current_time()
        )),
        false,
    );

    for mut last_sent in query.iter_mut() {
        last_sent.send_next_tick();
    }
}

type TimeInteger = Integer<0, 24000>;

#[command("time add")]
fn time_add(
    #[sender] sender: Sender,
    #[arg] time: TimeInteger,
    args: (ResMut<WorldTime>, Query<&mut LastSentTimeUpdate>),
) {
    let (mut world_time, mut query) = args;

    let new_time = world_time.current_time() + *time as u16;
    world_time.set_time(new_time);

    sender.send_message(
        TextComponent::from(format!("Advanced the world time by {} ticks", *time)),
        false,
    );

    for mut last_sent in query.iter_mut() {
        last_sent.send_next_tick();
    }
}

#[command("time query")]
fn time_query(#[sender] sender: Sender, world_time: Res<WorldTime>) {
    sender.send_message(
        TextComponent::from(format!(
            "The current world time is: {}",
            world_time.current_time()
        )),
        false,
    );
}
