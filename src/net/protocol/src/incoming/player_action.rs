use ionic_codec::net_types::network_position::NetworkPosition;
use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetDecode};

#[derive(NetDecode)]
#[packet(packet_id = "player_action", state = "play")]
pub struct PlayerAction {
    pub status: VarInt,
    pub location: NetworkPosition,
    pub face: u8,
    pub sequence: VarInt,
}
