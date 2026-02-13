use ionic_nbt::NBT;
use ionic_text::TextComponent;
use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetEncode};

#[derive(NetEncode)]
#[packet(packet_id = "open_screen", state = "play")]
pub struct OpenScreen {
    pub window_id: VarInt,
    pub window_type: VarInt,
    pub title: NBT<TextComponent>,
}
