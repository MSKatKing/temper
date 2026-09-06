use bevy_math::DVec3;
use temper_noise::{BlendedNoise, NormalNoise};
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};

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
pub struct OldBlendedNoise(pub BlendedNoise);

#[derive(Debug)]
pub struct Shift(pub NormalNoise);

#[derive(Debug)]
pub struct ShiftA(pub NormalNoise);

#[derive(Debug)]
pub struct ShiftB(pub NormalNoise);

impl DensityFunction for Noise {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let shift_x = self.shift_x.as_ref().map(|v| v.compute(ctx)).unwrap_or_default();
        let shift_y = self.shift_y.as_ref().map(|v| v.compute(ctx)).unwrap_or_default();
        let shift_z = self.shift_z.as_ref().map(|v| v.compute(ctx)).unwrap_or_default();
        
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
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        self.0.noise(ctx.block_pos().pos.as_dvec3())
    }
}

impl DensityFunction for Shift {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        self.0.noise(ctx.block_pos().pos.as_dvec3() * 0.25) * 4.0
    }
}

impl DensityFunction for ShiftA {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let pos = DVec3::new(
            ctx.block_pos().pos.x as f64,
            0.0,
            ctx.block_pos().pos.z as f64,
        );
        self.0.noise(pos * 0.25) * 4.0
    }
}

impl DensityFunction for ShiftB {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let pos = DVec3::new(
            ctx.block_pos().pos.z as f64,
            ctx.block_pos().pos.x as f64,
            0.0,
        );
        self.0.noise(pos * 0.25) * 4.0
    }
}
