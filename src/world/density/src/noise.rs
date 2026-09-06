use bevy_math::DVec3;
use temper_noise::{BlendedNoise, NormalNoise};
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};
use crate::wrapped::WrappedDensityFunction;

#[derive(Debug)]
pub struct Noise {
    pub noise: NormalNoise,
    pub xz_scale: f64,
    pub y_scale: f64,
    pub shift_x: Option<BoxedDensityFunction>,
    pub shift_y: Option<BoxedDensityFunction>,
    pub shift_z: Option<BoxedDensityFunction>,
}

#[derive(Debug)]
pub struct WrappedNoise<'a> {
    noise: &'a NormalNoise,
    xz_scale: f64,
    y_scale: f64,
    shift_x: Option<Box<dyn WrappedDensityFunction + 'a>>,
    shift_y: Option<Box<dyn WrappedDensityFunction + 'a>>,
    shift_z: Option<Box<dyn WrappedDensityFunction + 'a>>,
}

#[derive(Debug)]
pub struct OldBlendedNoise(pub BlendedNoise);

#[derive(Debug)]
pub struct Shift(pub NormalNoise);

#[derive(Debug)]
pub struct ShiftA(pub NormalNoise);

#[derive(Debug)]
pub struct ShiftB(pub NormalNoise);

impl DensityFunction for Noise {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(WrappedNoise {
            noise: &self.noise,
            xz_scale: self.xz_scale,
            y_scale: self.y_scale,
            shift_x: self.shift_x.as_ref().map(|v| v.wrap()),
            shift_y: self.shift_y.as_ref().map(|v| v.wrap()),
            shift_z: self.shift_z.as_ref().map(|v| v.wrap()),
        })
    }
}

impl WrappedDensityFunction for WrappedNoise<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let shift_x = self.shift_x.as_mut().map(|v| v.compute(ctx)).unwrap_or_default();
        let shift_y = self.shift_y.as_mut().map(|v| v.compute(ctx)).unwrap_or_default();
        let shift_z = self.shift_z.as_mut().map(|v| v.compute(ctx)).unwrap_or_default();

        let pos = ctx.block_pos();
        let pos = DVec3::new(
            pos.pos.x as f64 * self.xz_scale + shift_x,
            pos.pos.y as f64 * self.y_scale + shift_y,
            pos.pos.z as f64 * self.xz_scale + shift_z,
        );
        self.noise.noise(pos)
    }
}

impl DensityFunction for OldBlendedNoise {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(self)
    }
}

impl WrappedDensityFunction for &'_ OldBlendedNoise {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.0.noise(ctx.block_pos().pos.as_dvec3())
    }
}

impl DensityFunction for Shift {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(self)
    }
}

impl WrappedDensityFunction for &'_ Shift {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.0.noise(ctx.block_pos().pos.as_dvec3() * 0.25) * 4.0
    }
}

impl DensityFunction for ShiftA {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(self)
    }
}

impl WrappedDensityFunction for &'_ ShiftA {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let pos = DVec3::new(
            ctx.block_pos().pos.x as f64,
            0.0,
            ctx.block_pos().pos.z as f64,
        );
        self.0.noise(pos * 0.25) * 4.0
    }
}

impl DensityFunction for ShiftB {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(self)
    }
}

impl WrappedDensityFunction for &'_ ShiftB {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let pos = DVec3::new(
            ctx.block_pos().pos.z as f64,
            ctx.block_pos().pos.x as f64,
            0.0,
        );
        self.0.noise(pos * 0.25) * 4.0
    }
}
