use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetEncode};

#[derive(NetEncode)]
#[packet(packet_id = "login_compression", state = "login")]
pub struct SetCompressionPacket {
    pub threshold: VarInt,
}
