use super::encode_text_component;
use serde::{Deserialize, Serialize};
use std::io::Write;
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::id_or_inline::IdOr;
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_codec::net_types::var_int::VarInt;
use temper_macros::{NetDecode, NetEncode};
use temper_text::TextComponent;
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct Trim {
    pub material: IdOr<TrimMaterial>,
    pub pattern: IdOr<TrimPattern>,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetDecode, TypeHash)]
pub struct TrimMaterial {
    pub suffix: String,
    pub overrides: LengthPrefixedVec<TrimMaterialOverride>,
    pub description: TextComponent,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetEncode, NetDecode, TypeHash)]
pub struct TrimMaterialOverride {
    pub armor_material_type: String,
    pub overridden_asset_name: String,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetDecode, TypeHash)]
pub struct TrimPattern {
    pub asset_name: String,
    pub template_item: VarInt,
    pub description: TextComponent,
    pub decal: bool,
}

impl NetEncode for TrimMaterial {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.suffix.encode(writer, &NetEncodeOpts::None)?;
        self.overrides.encode(writer, &NetEncodeOpts::None)?;
        encode_text_component(&self.description, writer, &NetEncodeOpts::None)
    }
}

impl NetEncode for TrimPattern {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.asset_name.encode(writer, &NetEncodeOpts::None)?;
        self.template_item.encode(writer, &NetEncodeOpts::None)?;
        encode_text_component(&self.description, writer, &NetEncodeOpts::None)?;
        self.decal.encode(writer, &NetEncodeOpts::None)
    }
}
