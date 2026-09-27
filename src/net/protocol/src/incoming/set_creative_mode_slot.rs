use std::io::Read;
use temper_codec::decode::errors::NetDecodeError;
use temper_codec::decode::{NetDecode, NetDecodeOpts};
use temper_inventories::slot::InventorySlot;
use temper_macros::packet;

#[packet(packet_id = "set_creative_mode_slot", state = "play")]
pub struct SetCreativeModeSlot {
    pub slot_index: i16,
    pub slot: InventorySlot,
}

impl NetDecode for SetCreativeModeSlot {
    fn decode<R: Read>(reader: &mut R, opts: &NetDecodeOpts) -> Result<Self, NetDecodeError> {
        Ok(Self {
            slot_index: i16::decode(reader, opts)?,
            slot: InventorySlot::decode_creative_mode_slot(reader, opts)?,
        })
    }
}
