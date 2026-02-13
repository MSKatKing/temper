use ionic_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use ionic_macros::{packet, NetDecode};

#[derive(NetDecode)]
#[packet(packet_id = "key", state = "login")]
pub struct EncryptionResponse {
    pub shared_secret: LengthPrefixedVec<u8>,
    pub verify_token: LengthPrefixedVec<u8>,
}
