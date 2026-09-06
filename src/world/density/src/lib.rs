use crate::wrapped::WrappedDensityFunction;
use std::fmt::Debug;
use temper_core::pos::BlockPos;

pub mod compile;
mod conditional;
pub mod json;
mod mapped;
mod marker;
mod math;
mod noise;
pub mod wrapped;
mod spline;

pub type BoxedDensityFunction = Box<dyn DensityFunction>;

pub struct DensityFunctionContext {
    pub block_pos: BlockPos,
}

impl DensityFunctionContext {
    pub fn new(pos: BlockPos) -> Self {
        Self { block_pos: pos }
    }

    pub fn block_pos(&self) -> &BlockPos {
        &self.block_pos
    }
}

pub trait DensityFunction: Debug + Send + Sync {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_>;
}

#[derive(Debug)]
pub struct Constant(pub f64);

impl DensityFunction for Constant {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(self)
    }
}

impl WrappedDensityFunction for &'_ Constant {
    fn compute(&mut self, _: &DensityFunctionContext) -> f64 {
        self.0
    }
}
