use ionic_macros::{packet, NetEncode};
use ionic_text::TextComponent;

#[derive(NetEncode, Debug, Clone)]
#[packet(packet_id = "system_chat", state = "play")]
pub struct SystemMessagePacket {
    pub message: ionic_nbt::NBT<TextComponent>,
    pub overlay: bool,
}
