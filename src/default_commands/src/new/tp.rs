use temper_command_infra::args::{EntityArg, PositionArg};
use temper_macros::Command;

#[derive(Command)]
#[command("tp")]
#[allow(dead_code)]
enum TpCommand {
    TpToPos {
        location: PositionArg,
    },
    TpToEntity {
        destination: EntityArg,
    },
    TpEntityToPos {
        target: EntityArg,
        location: PositionArg,
    },
    TpEntityToEntity {
        target: EntityArg,
        destination: EntityArg,
    },
}
