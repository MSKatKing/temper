use super::DyeColor;
use serde::{Deserialize, Serialize};
use temper_codec::net_types::id_or_inline::IdOr;
use temper_macros::{NetDecode, NetEncode};
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BannerPatternLayer {
    pub pattern_type: IdOr<BannerPattern>,
    pub color: DyeColor,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct BannerPattern {
    pub asset_id: String,
    pub translation_key: String,
}
