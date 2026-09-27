use serde::{Deserialize, Serialize};
use temper_macros::{NetDecode, NetEncode};
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct PaintingVariant {
    pub width: i32,
    pub height: i32,
    pub asset_id: String,
}
