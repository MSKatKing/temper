use crate::wrapped::WrappedDensityFunction;
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};
use std::ops::Range;

#[derive(Debug)]
pub struct IntervalSelect {
    pub input: BoxedDensityFunction,
    pub thresholds: Vec<f64>,
    pub functions: Vec<BoxedDensityFunction>,
}

#[derive(Debug)]
pub struct WrappedIntervalSelect<'a> {
    input: Box<dyn WrappedDensityFunction + 'a>,
    thresholds: &'a [f64],
    functions: Vec<Box<dyn WrappedDensityFunction + 'a>>,
}

#[derive(Debug)]
pub struct RangeChoice {
    pub input: BoxedDensityFunction,
    pub range: Range<f64>,
    pub when_in_range: BoxedDensityFunction,
    pub when_out_range: BoxedDensityFunction,
}

#[derive(Debug)]
pub struct WrappedRangeChoice<'a> {
    input: Box<dyn WrappedDensityFunction + 'a>,
    range: &'a Range<f64>,
    when_in_range: Box<dyn WrappedDensityFunction + 'a>,
    when_out_range: Box<dyn WrappedDensityFunction + 'a>,
}

impl DensityFunction for IntervalSelect {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(WrappedIntervalSelect {
            input: self.input.wrap(),
            thresholds: &self.thresholds,
            functions: self.functions.iter().map(|v| v.wrap()).collect(),
        })
    }
}

impl WrappedDensityFunction for WrappedIntervalSelect<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let input = self.input.compute(ctx);

        for (i, threshold) in self.thresholds.iter().enumerate() {
            if input < *threshold {
                return self.functions[i].compute(ctx);
            }
        }

        self.functions.last_mut().unwrap().compute(ctx)
    }
}

impl DensityFunction for RangeChoice {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(WrappedRangeChoice {
            input: self.input.wrap(),
            range: &self.range,
            when_in_range: self.when_in_range.wrap(),
            when_out_range: self.when_out_range.wrap(),
        })
    }
}

impl WrappedDensityFunction for WrappedRangeChoice<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        let input = self.input.compute(ctx);

        if self.range.contains(&input) {
            self.when_in_range.compute(ctx)
        } else {
            self.when_out_range.compute(ctx)
        }
    }
}
