mod marker;

pub use marker::*;

use crate::DensityFunctionContext;
use std::fmt::Debug;

pub trait WrappedDensityFunction: Debug {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64;
}
