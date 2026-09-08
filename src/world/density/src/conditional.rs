use crate::wrapped::conditional::ConditionalDensityFunction;
use crate::wrapped::{push_op, FlattenedDensityFunction};
use crate::{BoxedDensityFunction, DensityFunction};
use std::ops::Range;

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
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        push_op(ops, |ops| {
            FlattenedDensityFunction::Conditional {
                op: ConditionalDensityFunction::IntervalSelect {
                    thresholds: &self.thresholds,
                    functions: self.functions.iter().map(|v| v.wrap(ops)).collect(),
                },
                input: self.input.wrap(ops),
            }
        })
    }
}

impl DensityFunction for RangeChoice {
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        push_op(ops, |ops| {
            FlattenedDensityFunction::Conditional {
                op: ConditionalDensityFunction::RangeChoice {
                    range: &self.range,
                    when_in_range: self.when_in_range.wrap(ops),
                    when_out_range: self.when_out_range.wrap(ops),
                },
                input: self.input.wrap(ops),
            }
        })
    }
}
