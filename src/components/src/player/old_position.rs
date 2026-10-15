use crate::player::position::Position;
use bevy_ecs::prelude::Component;
use bevy_math::DVec3;
use std::ops::{Deref, DerefMut};
use temper_core::dimension::Dimension;

#[derive(Component, Debug, Clone, Copy)]
pub struct OldPosition(DVec3, Dimension);

impl OldPosition {
    pub fn new(x: f64, y: f64, z: f64, dimension: Dimension) -> Self {
        OldPosition(DVec3::new(x, y, z), dimension)
    }
}

impl From<Position> for OldPosition {
    fn from(value: Position) -> Self {
        OldPosition(value.coords, value.dimension)
    }
}

impl From<OldPosition> for Position {
    fn from(value: OldPosition) -> Self {
        Position::new(value.x, value.y, value.z, value.1)
    }
}

impl Deref for OldPosition {
    type Target = DVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OldPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
