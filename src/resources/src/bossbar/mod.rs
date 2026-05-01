use bevy_ecs::prelude::Resource;
use std::collections::HashMap;
use temper_text::TextComponent;
use uuid::Uuid;

mod bossbar_data;
mod update_kinds;

pub use bossbar_data::*;
pub use update_kinds::*;

#[derive(Resource)]
pub struct BossBarResource {
    pub update_queue: crossbeam_queue::SegQueue<(Uuid, UpdateBBKind)>,
    pub boss_bars: HashMap<Uuid, BossBarData>,
}

impl BossBarResource {
    pub fn add(self, data: BossBarData) -> Uuid {
        let uuid = Uuid::new_v4();
        self.update_queue.push((uuid, UpdateBBKind::Add { data }));
        uuid
    }

    pub fn remove(self, uuid: Uuid) {
        self.update_queue.push((uuid, UpdateBBKind::Remove));
    }

    pub fn update_health(self, uuid: Uuid, new_health: f32) {
        self.update_queue
            .push((uuid, UpdateBBKind::UpdateHealth { new_health }));
    }

    pub fn update_title(self, uuid: Uuid, title: TextComponent) {
        self.update_queue
            .push((uuid, UpdateBBKind::UpdateTitle { title }));
    }

    pub fn update_style(self, uuid: Uuid, color: BossbarColor, dividers: BossbarDividers) {
        self.update_queue
            .push((uuid, UpdateBBKind::UpdateStyle { color, dividers }));
    }

    pub fn update_flags(self, uuid: Uuid, flags: BossbarFlags) {
        self.update_queue
            .push((uuid, UpdateBBKind::UpdateFlags { flags }));
    }
}
