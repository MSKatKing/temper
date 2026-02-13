use ionic_codec::net_types::network_position::NetworkPosition;
use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetEncode};

#[derive(NetEncode)]
#[packet(packet_id = "block_update", state = "play")]
pub struct BlockUpdate {
    pub location: NetworkPosition,
    pub block_state_id: VarInt,
}
