use crate::wrapped::FlattenedDensityFunction;
use std::fmt::Debug;
use temper_core::pos::BlockPos;

pub mod compile;
mod conditional;
pub mod json;
mod mapped;
mod marker;
mod math;
mod noise;
mod spline;
pub mod wrapped;

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
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize;
}

impl DensityFunction for f64 {
    fn wrap(&self, ops: &mut Vec<FlattenedDensityFunction>) -> usize {
        ops.push(FlattenedDensityFunction::Constant(*self));
        ops.len() - 1
    }
}
