use crate::bossbar::{BossBarData, BossbarColor, BossbarDividers, BossbarFlags};
use temper_text::TextComponent;

#[derive(Clone)]
pub enum UpdateBBKind {
    Add {
        data: BossBarData,
    },
    Remove,
    UpdateHealth {
        new_health: f32,
    },
    UpdateTitle {
        title: TextComponent,
    },
    UpdateStyle {
        color: BossbarColor,
        dividers: BossbarDividers,
    },
    UpdateFlags {
        flags: BossbarFlags,
    },
}
