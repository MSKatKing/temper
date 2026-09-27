use super::{SoundEvent, encode_text_component};
use serde::{Deserialize, Serialize};
use std::io::Write;
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_codec::net_types::id_or_inline::IdOr;
use temper_codec::net_types::var_int::VarInt;
use temper_macros::NetDecode;
use temper_text::TextComponent;
use type_hash::TypeHash;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, NetDecode, TypeHash)]
pub struct JukeboxSong {
    pub sound_event: IdOr<SoundEvent>,
    pub description: TextComponent,
    pub duration: f32,
    pub output: VarInt,
}

impl NetEncode for JukeboxSong {
    fn encode<W: Write>(
        &self,
        writer: &mut W,
        _opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        self.sound_event.encode(writer, &NetEncodeOpts::None)?;
        encode_text_component(&self.description, writer, &NetEncodeOpts::None)?;
        self.duration.encode(writer, &NetEncodeOpts::None)?;
        self.output.encode(writer, &NetEncodeOpts::None)
    }
}
