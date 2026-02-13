use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetDecode};

#[derive(NetDecode, Debug, Clone)]
#[packet(packet_id = "command_suggestion", state = "play")]
pub struct CommandSuggestionRequest {
    pub transaction_id: VarInt,
    pub input: String,
}
