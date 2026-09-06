use temper_core::pos::BlockPos;
use crate::{DensityFunction, DensityFunctionContext};

#[derive(Debug)]
pub struct CacheAllInCell(pub Box<dyn DensityFunction>);

#[derive(Debug)]
pub struct CacheOnce(pub Box<dyn DensityFunction>, pub usize);

#[derive(Debug)]
pub struct Cache2d(pub Box<dyn DensityFunction>, pub usize);

#[derive(Debug)]
pub struct FlatCache(pub Box<dyn DensityFunction>, pub usize);

#[derive(Debug)]
pub struct Interpolated(pub Box<dyn DensityFunction>);

impl DensityFunction for CacheAllInCell {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        self.0.compute(ctx)
    }
}

impl DensityFunction for CacheOnce {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let pos = ctx.block_pos();
        get_or_fill(ctx, *pos, self.1, self.0.as_ref())
    }
}

impl DensityFunction for Cache2d {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let pos = ctx.block_pos();
        let pos = BlockPos::of(pos.pos.x, 0, pos.pos.z);
        get_or_fill(ctx, pos, self.1, self.0.as_ref())
    }
}

impl DensityFunction for FlatCache {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let pos = ctx.block_pos();
        let pos = BlockPos::of(pos.pos.x >> 2, 0, pos.pos.z >> 2);
        get_or_fill(ctx, pos, self.1, self.0.as_ref())
    }
}

impl DensityFunction for Interpolated {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        self.0.compute(ctx)
    }
}

fn get_or_fill(ctx: &mut DensityFunctionContext, pos: BlockPos, idx: usize, func: &dyn DensityFunction) -> f64 {
    let storage = ctx.cache_storage(idx);

    if storage.last_pos != pos {
        let val = func.compute(ctx);

        let storage = ctx.cache_storage_mut(idx);
        storage.last_pos = pos;
        storage.last_value = val;

        val
    } else {
        storage.last_value
    }
}
