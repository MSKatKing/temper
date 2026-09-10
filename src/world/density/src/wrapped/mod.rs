pub mod binary;
pub mod conditional;
pub mod mapped;
pub mod marker;
pub mod noise;
pub mod spline;
pub mod unary;

pub use marker::*;

use crate::BoxedDensityFunction;
use crate::mapped::Gradient;
use crate::wrapped::binary::BinaryDensityFunction;
use crate::wrapped::conditional::ConditionalDensityFunction;
use crate::wrapped::mapped::Lerp;
use crate::wrapped::noise::NoiseDensityFunction;
use crate::wrapped::spline::FlattenedSpline;
use crate::wrapped::unary::UnaryDensityFunction;
use temper_core::pos::BlockPos;

pub struct WrappedDensityFunction<'a> {
    functions: Box<[FlattenedDensityFunction<'a>]>,
    pos: BlockPos,
}

pub enum FlattenedDensityFunction<'a> {
    Constant(f64),
    Unary {
        op: UnaryDensityFunction,
        arg: usize,
    },
    Binary {
        op: BinaryDensityFunction,
        lhs: usize,
        rhs: usize,
    },
    Spline(FlattenedSpline<'a>),
    Marker {
        op: MarkerDensityFunction,
        arg: usize,
    },
    Gradient(&'a Gradient),
    Lerp(Lerp),
    Conditional {
        op: ConditionalDensityFunction<'a>,
        input: usize,
    },
    Noise(NoiseDensityFunction<'a>),
}

impl WrappedDensityFunction<'_> {
    pub fn wrap(func: &BoxedDensityFunction) -> WrappedDensityFunction<'_> {
        let mut ops = Vec::new();
        func.wrap(&mut ops);

        WrappedDensityFunction {
            functions: ops.into_boxed_slice(),
            pos: BlockPos::of(i32::MAX, i32::MAX, i32::MAX),
        }
    }

    pub fn execute(&mut self, pos: BlockPos) -> f64 {
        self.pos = pos;
        self.execute_inner(0)
    }

    fn execute_inner(&self, idx: usize) -> f64 {
        let func = unsafe {
            ((&raw const self.functions[idx]) as *mut FlattenedDensityFunction).as_mut_unchecked()
        };
        func.execute(self)
    }
}

impl FlattenedDensityFunction<'_> {
    fn execute(&mut self, func: &WrappedDensityFunction) -> f64 {
        match self {
            FlattenedDensityFunction::Constant(val) => *val,
            FlattenedDensityFunction::Unary { op, arg } => op.execute(func.execute_inner(*arg)),
            FlattenedDensityFunction::Binary { op, lhs, rhs } => {
                op.execute(func.execute_inner(*lhs), func.execute_inner(*rhs))
            }
            FlattenedDensityFunction::Spline(spline) => spline.sample(func),
            FlattenedDensityFunction::Marker { op, arg } => op.execute(*arg, func),
            FlattenedDensityFunction::Gradient(gradient) => gradient.sample(func.pos),
            FlattenedDensityFunction::Lerp(lerp) => lerp.compute(func),
            FlattenedDensityFunction::Conditional { op, input } => {
                op.compute(func.execute_inner(*input), func)
            }
            FlattenedDensityFunction::Noise(noise) => noise.sample(func),
        }
    }
}

pub fn push_op<'a>(
    ops: &mut Vec<FlattenedDensityFunction<'a>>,
    f: impl FnOnce(&mut Vec<FlattenedDensityFunction<'a>>) -> FlattenedDensityFunction<'a>,
) -> usize {
    let idx = ops.len();
    // SAFETY: we replace the data before returning
    ops.push(unsafe { std::mem::zeroed() });
    let data = f(ops);
    ops[idx] = data;
    idx
}
