use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetDecode};

#[derive(NetDecode)]
#[packet(packet_id = "accept_teleportation", state = "play")]
pub struct ConfirmPlayerTeleport {
    pub teleport_id: VarInt,
}
