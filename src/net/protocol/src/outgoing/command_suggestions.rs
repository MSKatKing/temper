use ionic_nbt::NBT;
use ionic_text::TextComponent;
use ionic_codec::net_types::{
    length_prefixed_vec::LengthPrefixedVec, prefixed_optional::PrefixedOptional, var_int::VarInt,
};
use ionic_macros::{packet, NetEncode};

#[derive(NetEncode)]
#[packet(packet_id = "command_suggestions", state = "play")]
pub struct CommandSuggestionsPacket {
    pub transaction_id: VarInt,
    pub start: VarInt,
    pub length: VarInt,
    pub matches: LengthPrefixedVec<Match>,
}

#[derive(NetEncode)]
pub struct Match {
    pub content: String,
    pub tooltip: PrefixedOptional<NBT<TextComponent>>,
}
