use std::ops::Range;
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};

#[derive(Debug)]
pub struct IntervalSelect {
    pub input: BoxedDensityFunction,
    pub thresholds: Vec<f64>,
    pub functions: Vec<BoxedDensityFunction>,
}

#[derive(Debug)]
pub struct RangeChoice {
    pub input: BoxedDensityFunction,
    pub range: Range<f64>,
    pub when_in_range: BoxedDensityFunction,
    pub when_out_range: BoxedDensityFunction,
}

impl DensityFunction for IntervalSelect {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let input = self.input.compute(ctx);
        
        let mut idx = 0;
        while input > self.thresholds[idx] { idx += 1; }
        
        self.functions[idx].compute(ctx)
    }
}

impl DensityFunction for RangeChoice {
    fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
        let input = self.input.compute(ctx);
        
        if self.range.contains(&input) {
            self.when_in_range.compute(ctx)
        } else {
            self.when_out_range.compute(ctx)
        }
    }
}
