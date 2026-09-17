use bevy_ecs::prelude::{Query, ResMut};
use bevy_ecs::system::SystemParamItem;
use tracing::warn;
use temper_command_infra::{CommandHandler, CommandResult, CommandSource};
use temper_macros::Command;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::game_event::GameEventPacket;
use temper_resources::weather::WeatherStatus;

#[derive(Command)]
#[command("weather")]
pub enum WeatherCommand {
    #[literal("clear")]
    Clear,
    #[literal("rain")]
    Rain,
    #[literal("thunder")]
    Thunder,
}

impl CommandHandler for WeatherCommand {
    type SystemParam<'w, 's> = (
        ResMut<'w, WeatherStatus>,
        Query<'w, 's, &'static StreamWriter>,
    );

    fn handle(self, _source: CommandSource, params: &mut SystemParamItem<'_, '_, Self::SystemParam<'_, '_>>) -> CommandResult {
        match self {
            Self::Clear => params.0.clear(),
            Self::Rain => params.0.rain(),
            Self::Thunder => params.0.thunder(),
        }

        let rain_packet = GameEventPacket::new(
            GameEventPacket::RAIN_LEVEL_CHANGE,
            params.0.rain_amount()
        );

        let thunder_packet = GameEventPacket::new(
            GameEventPacket::THUNDER_LEVEL_CHANGE,
            params.0.thunder_amount()
        );

        for writer in params.1.iter() {
            writer.send_packet_ref(&rain_packet).unwrap_or_else(|_| {
                warn!("Failed to send weather update packet to player.")
            });

            writer.send_packet_ref(&thunder_packet).unwrap_or_else(|_| {
                warn!("Failed to send weather update packet to player.")
            })
        }

        Ok(())
    }
}
