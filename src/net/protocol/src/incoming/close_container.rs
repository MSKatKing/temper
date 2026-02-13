use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetDecode};

#[derive(NetDecode)]
#[packet(packet_id = "container_close", state = "play")]
pub struct CloseContainer {
    pub window_id: VarInt,
}
