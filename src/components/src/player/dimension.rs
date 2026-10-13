use std::ops::{Deref, DerefMut};
use bevy_ecs::prelude::Component;
use temper_core::dimension::Dimension;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Component)]
pub struct CurrentDimension(pub Dimension);

impl Deref for CurrentDimension {
    type Target = Dimension;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurrentDimension {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Dimension> for CurrentDimension {
    fn from(value: Dimension) -> Self {
        Self(value)
    }
}
