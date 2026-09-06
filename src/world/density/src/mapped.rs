use std::ops::{Div, Rem};
use temper_core::math::TemperMathExt;
use temper_core::pos::BlockPos;
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};
use crate::wrapped::WrappedDensityFunction;

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
pub enum Tiling {
    ClampToEdge,
    Repeat,
    MirroredRepeat,
}

#[derive(Debug)]
pub struct Gradient {
    pub axis: Axis,
    pub tiling: Tiling,
    pub from_coord: i32,
    pub to_coord: i32,
    pub from_value: f64,
    pub to_value: f64,
}

#[derive(Debug)]
pub struct Lerp {
    pub alpha: BoxedDensityFunction,
    pub first: BoxedDensityFunction,
    pub second: BoxedDensityFunction,
}

#[derive(Debug)]
pub struct WrappedLerp<'a> {
    alpha: Box<dyn WrappedDensityFunction + 'a>,
    first: Box<dyn WrappedDensityFunction + 'a>,
    second: Box<dyn WrappedDensityFunction + 'a>,
}

impl Axis {
    fn get_coord(&self, pos: &BlockPos) -> f64 {
        match self {
            Axis::X => pos.pos.x as f64,
            Axis::Y => pos.pos.y as f64,
            Axis::Z => pos.pos.z as f64,
        }
    }
}

impl DensityFunction for Gradient {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(self)
    }
}

impl WrappedDensityFunction for &'_ Gradient {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let coord = self.axis.get_coord(ctx.block_pos());
        let coord_range = self.to_coord as f64 - self.from_coord as f64;
        let coord_factor = (self.to_value - self.from_value) / coord_range;

        match self.tiling {
            Tiling::ClampToEdge => {
                let rel = coord.clamp(self.from_coord as f64, self.to_coord as f64) - self.from_coord as f64;
                self.from_value + rel * coord_factor
            }
            Tiling::MirroredRepeat => {
                let rel = coord - self.from_coord as f64;
                let tile_idx = rel.div(coord_range).floor();
                let local_coord = rel - tile_idx * coord_range;

                if (tile_idx as i32 & 1) == 0 {
                    self.from_value + local_coord * coord_factor
                } else {
                    self.from_value + (coord_range - local_coord) * coord_factor
                }
            }
            Tiling::Repeat => {
                let rel = coord - self.from_coord as f64;
                self.from_value + rel.rem(coord_range).floor() * coord_factor
            }
        }
    }
}

impl DensityFunction for Lerp {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(WrappedLerp {
            alpha: self.alpha.wrap(),
            first: self.first.wrap(),
            second: self.second.wrap(),
        })
    }
}

impl WrappedDensityFunction for WrappedLerp<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.alpha.compute(ctx).lerp(
            self.first.compute(ctx),
            self.second.compute(ctx),
        )
    }
}
