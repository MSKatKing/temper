mod marker;

pub use marker::*;

use std::fmt::Debug;
use crate::DensityFunctionContext;

pub trait WrappedDensityFunction: Debug {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64;
}
