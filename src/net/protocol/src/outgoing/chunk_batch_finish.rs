use ionic_codec::net_types::var_int::VarInt;
use ionic_macros::{packet, NetEncode};

#[derive(NetEncode)]
#[packet(packet_id = "chunk_batch_finished", state = "play")]
pub struct ChunkBatchFinish {
    pub batch_size: VarInt,
}
