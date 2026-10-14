use bevy_ecs::prelude::Component;
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use temper_core::dimension::Dimension;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Component, Default, Serialize, Deserialize)]
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

impl From<CurrentDimension> for Dimension {
    fn from(value: CurrentDimension) -> Self {
        value.0
    }
}
