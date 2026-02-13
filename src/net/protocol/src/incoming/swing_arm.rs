use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetDecode};

#[derive(NetDecode)]
#[packet(packet_id = "swing", state = "play")]
pub struct SwingArmPacket {
    pub hand: VarInt,
}
