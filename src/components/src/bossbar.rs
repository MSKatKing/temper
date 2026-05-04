use bevy_ecs::component::Component;
use uuid::Uuid;

#[derive(Component, Debug, Clone, Copy)]
pub struct BossbarOwner(Uuid);

impl BossbarOwner {
    pub fn new(id: Uuid) -> Self {
        BossbarOwner(id)
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}
