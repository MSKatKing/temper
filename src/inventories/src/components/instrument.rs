use super::{SoundEvent, encode_text_component};
use serde::{Deserialize, Serialize};
use std::io::Write;
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::id_or_inline::IdOr;
use temper_macros::NetDecode;
use temper_text::TextComponent;
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetDecode, TypeHash)]
pub struct Instrument {
    pub sound_event: IdOr<SoundEvent>,
    pub use_duration: f32,
    pub range: f32,
    pub description: TextComponent,
}

impl NetEncode for Instrument {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.sound_event.encode(writer, &NetEncodeOpts::None)?;
        self.use_duration.encode(writer, &NetEncodeOpts::None)?;
        self.range.encode(writer, &NetEncodeOpts::None)?;
        encode_text_component(&self.description, writer, &NetEncodeOpts::None)
    }
}
