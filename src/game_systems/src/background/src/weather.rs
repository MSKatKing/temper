use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Query};
use tracing::warn;
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_components::player::time::LastSentTimeUpdate;
use temper_net_runtime::connection::StreamWriter;
use temper_protocol::outgoing::game_event::GameEventPacket;
use temper_protocol::outgoing::update_time::{Clock, UpdateTimePacket};
use temper_resources::weather::WeatherStatus;

pub fn tick_weather_cycle(
    mut weather: ResMut<WeatherStatus>,
    players: Query<(Entity, &StreamWriter)>,
) {
    if weather.tick() {
        let rain_packet = GameEventPacket::new(
            GameEventPacket::RAIN_LEVEL_CHANGE,
            weather.rain_amount(),
        );
        
        let thunder_packet = GameEventPacket::new(
            GameEventPacket::THUNDER_LEVEL_CHANGE,
            weather.thunder_amount(),
        );
        
        for (eid, writer) in players.iter() {
            writer.send_packet_ref(&rain_packet).unwrap_or_else(|_| {
                warn!("Failed to send weather update to player {}", eid);
            });
            
            writer.send_packet_ref(&thunder_packet).unwrap_or_else(|_| {
                warn!("Failed to send weather update to player {}", eid);
            })
        }
    }
}