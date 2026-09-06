use std::fmt::Debug;
use temper_core::pos::{BlockPos, ChunkBlockPos, ChunkPos};

mod json;
mod marker;
mod math;
mod noise;
mod mapped;
mod conditional;

pub type BoxedDensityFunction = Box<dyn DensityFunction>;

pub struct CacheStorage {
    last_pos: BlockPos,
    last_value: f64,
}

pub struct DensityFunctionContext {
    block_pos: BlockPos,
    cache_storage: Vec<CacheStorage>,
}

impl DensityFunctionContext {
    pub fn block_pos(&self) -> &BlockPos {
        &self.block_pos
    }
    
    pub fn cache_storage(&self, idx: usize) -> &CacheStorage {
        &self.cache_storage[idx]
    }
    
    pub fn cache_storage_mut(&mut self, idx: usize) -> &mut CacheStorage {
        &mut self.cache_storage[idx]
    }
}

pub trait DensityFunction: Debug {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64;
}

#[derive(Debug)]
pub struct Constant(pub f64);

impl DensityFunction for Constant {
    fn compute(&self, _: &mut DensityFunctionContext) -> f64 {
        self.0
    }
}
