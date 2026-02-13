use ionic_inventories::slot::InventorySlot;
use ionic_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetEncode};

#[derive(NetEncode)]
#[packet(packet_id = "container_set_content", state = "play")]
pub struct SetContainerContent {
    pub window_id: VarInt,
    pub state_id: VarInt,
    pub slots: LengthPrefixedVec<InventorySlot>,
    pub carried_item: InventorySlot,
}
