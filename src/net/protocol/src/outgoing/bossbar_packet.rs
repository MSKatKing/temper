use std::io::Write;
use thiserror::__private18::Var;
use temper_codec::encode::errors::NetEncodeError;
use temper_codec::encode::{NetEncode, NetEncodeOpts};
use temper_macros::{NetEncode, packet, Discriminant};
use tokio::io::AsyncWrite;
use temper_codec::net_types::var_int::VarInt;
use temper_text::TextComponent;

#[derive(NetEncode, Discriminant)]
pub enum BossbarAction {
    Add,
    Remove,
    UpdateHealth,
    UpdateTitle,
    UpdateStyle,
    UpdateFlags,
}

#[packet(packet_id = "boss_event", state = "play")]
pub struct BossbarPacket {
    pub uuid: u128,
    pub action: BossbarAction,

    pub title: TextComponent,
    pub health: f32,
    pub color: VarInt,
    pub division: VarInt,
    pub flags: u8,
}

impl BossbarPacket {
    pub fn add_bossbar(uuid: u128, title: TextComponent, health: f32, color: VarInt, division: VarInt, flags: u8) -> BossbarPacket {
        BossbarPacket {
            uuid,
            action: BossbarAction::Add,
            title,
            health,
            color,
            division,
            flags,
        }
    }

    pub fn remove_bossbar(uuid: u128) -> BossbarPacket {
        BossbarPacket {
            uuid,
            action: BossbarAction::Remove,
            title: Default::default(),
            health: 0.0,
            color: Default::default(),
            division: Default::default(),
            flags: 0,
        }
    }

    pub fn update_health(uuid: u128, health: f32) -> BossbarPacket {
        BossbarPacket {
            uuid,
            action: BossbarAction::UpdateHealth,
            title: Default::default(),
            health,
            color: Default::default(),
            division: Default::default(),
            flags: 0,
        }
    }

    pub fn update_title(uuid: u128, title: TextComponent) -> BossbarPacket {
        BossbarPacket {
            uuid,
            action: BossbarAction::UpdateTitle,
            title,
            health: 0.0,
            color: Default::default(),
            division: Default::default(),
            flags: 0,
        }
    }

    pub fn update_style(uuid: u128, color: VarInt, division: VarInt) -> BossbarPacket {
        BossbarPacket {
            uuid,
            action: BossbarAction::UpdateStyle,
            title: Default::default(),
            health: 0.0,
            color,
            division,
            flags: 0,
        }
    }

    pub fn update_flags(uuid: u128, flags: u8) -> BossbarPacket {
        BossbarPacket {
            uuid,
            action: BossbarAction::UpdateFlags,
            title: Default::default(),
            health: 0.0,
            color: Default::default(),
            division: Default::default(),
            flags,
        }
    }
}

impl NetEncode for BossbarPacket {
    fn encode<W: Write>(&self, writer: &mut W, opts: &NetEncodeOpts) -> Result<(), NetEncodeError> {
        todo!()
    }

    async fn encode_async<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        opts: &NetEncodeOpts,
    ) -> Result<(), NetEncodeError> {
        todo!()
    }
}
