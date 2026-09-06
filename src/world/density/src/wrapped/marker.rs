use temper_core::pos::BlockPos;
use crate::DensityFunctionContext;
use crate::wrapped::WrappedDensityFunction;

#[derive(Debug)]
pub struct CacheAllInCell<'a>(pub(crate) Box<dyn WrappedDensityFunction + 'a>);

#[derive(Debug)]
pub struct CacheOnce<'a>(pub(crate) Box<dyn WrappedDensityFunction + 'a>);

#[derive(Debug)]
pub struct Cache2d<'a> {
    pub(crate) inner: Box<dyn WrappedDensityFunction + 'a>,
    pub(crate) last_pos: BlockPos,
    pub(crate) last_value: f64,
}

#[derive(Debug)]
pub struct FlatCache<'a> {
    pub(crate) inner: Box<dyn WrappedDensityFunction + 'a>,
    pub(crate) last_pos: BlockPos,
    pub(crate) last_value: f64,
}

#[derive(Debug)]
pub struct Interpolated<'a> {
    pub(crate) inner: Box<dyn WrappedDensityFunction + 'a>,
    pub(crate) last_pos: BlockPos,
    pub(crate) data: [f64; 8],
}

impl WrappedDensityFunction for CacheAllInCell<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.0.compute(ctx)
    }
}

impl WrappedDensityFunction for CacheOnce<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.0.compute(ctx)
    }
}

impl WrappedDensityFunction for Cache2d<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let pos = ctx.block_pos();
        let pos = BlockPos::of(pos.pos.x, 0, pos.pos.z);
        
        if pos != self.last_pos {
            self.last_pos = pos;
            self.last_value = self.inner.compute(ctx);
        }
        
        self.last_value
    }
}

impl WrappedDensityFunction for FlatCache<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let pos = ctx.block_pos();
        let pos = BlockPos::of(pos.pos.x >> 2, 0, pos.pos.z >> 2);
        
        if pos != self.last_pos {
            self.last_pos = pos;
            self.last_value = self.inner.compute(ctx);
        }
        
        self.last_value
    }
}

impl WrappedDensityFunction for Interpolated<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.inner.compute(ctx)
    }
}
