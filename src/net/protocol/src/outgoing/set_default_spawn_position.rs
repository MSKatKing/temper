use bevy_math::DVec3;
use temper_codec::net_types::network_position::NetworkPosition;
use temper_components::player::position::Position;
use temper_macros::{NetEncode, packet};

#[derive(NetEncode)]
#[packet(packet_id = "set_default_spawn_position", state = "play")]
pub struct SetDefaultSpawnPositionPacket {
    pub dimension: String,
    pub spawn_position: NetworkPosition,
    pub yaw: f32,
    pub pitch: f32,
}

// Spawn in chunk (1, 1) at y=100 to ensure spawning above ground, since for some reason the terrain
// gen can't create land at (0, 0)
pub const DEFAULT_SPAWN_POSITION: Position = Position {
    coords: DVec3 {
        x: 16.0,
        y: 100.0,
        z: 16.0,
    },
};

impl Default for SetDefaultSpawnPositionPacket {
    fn default() -> Self {
        Self::new()
    }
}

impl SetDefaultSpawnPositionPacket {
    pub fn new() -> Self {
        Self {
            dimension: "minecraft:overworld".to_string(),
            spawn_position: DEFAULT_SPAWN_POSITION.into(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}
