use std::ops::Neg;
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};
use crate::wrapped::WrappedDensityFunction;

macro_rules! math_function {
    (binary $name:ident, $wrapped_name:ident, $fun:expr) => {
        #[derive(Debug)]
        pub struct $name {
            pub left: BoxedDensityFunction,
            pub right: BoxedDensityFunction,
        }
        
        #[derive(Debug)]
        pub struct $wrapped_name<'a> {
            left: Box<dyn WrappedDensityFunction + 'a>,
            right: Box<dyn WrappedDensityFunction + 'a>,
        }

        impl DensityFunction for $name {
            fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
                Box::new($wrapped_name {
                    left: self.left.wrap(),
                    right: self.right.wrap(),
                })
            }
        }
        
        impl WrappedDensityFunction for $wrapped_name<'_> {
            fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
                $fun(
                    self.left.compute(ctx),
                    self.right.compute(ctx),
                )
            }
        }
    };
    (unary $name:ident, $wrapped_name:ident, $fun:expr) => {
        #[derive(Debug)]
        pub struct $name(pub BoxedDensityFunction);
        
        #[derive(Debug)]
        pub struct $wrapped_name<'a>(Box<dyn WrappedDensityFunction + 'a>);

        impl DensityFunction for $name {
            fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
                Box::new($wrapped_name(self.0.wrap()))
            }
        }
        
        impl WrappedDensityFunction for $wrapped_name<'_> {
            fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
                $fun(self.0.compute(ctx))
            }
        }
    };
    (custom $name:ident, $wrapped_name:ident, $fun:expr, $($field:ident: $ty:ty),* $(,)?) => {
        #[derive(Debug)]
        pub struct $name {
            pub inner: BoxedDensityFunction,
            $(
            pub $field: $ty,
            )*
        }
        
        #[derive(Debug)]
        pub struct $wrapped_name<'a> {
            inner: Box<dyn WrappedDensityFunction + 'a>,
            original: &'a $name,
        }

        impl DensityFunction for $name {
            fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
                Box::new($wrapped_name {
                    inner: self.inner.wrap(),
                    original: &self,
                })
            }
        }
        
        impl WrappedDensityFunction for $wrapped_name<'_> {
            fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
                $fun(
                    self.inner.compute(ctx),
                    $(self.original.$field),*
                )
            }
        }
    }
}

math_function!(binary Add, WrappedAdd, <f64 as std::ops::Add>::add);
math_function!(binary Sub, WrappedSub, <f64 as std::ops::Sub>::sub);
math_function!(binary Mul, WrappedMul, <f64 as std::ops::Mul>::mul);
math_function!(binary Div, WrappedDiv, <f64 as std::ops::Div>::div);
math_function!(binary Min, WrappedMin, f64::min);
math_function!(binary Max, WrappedMax, f64::max);
math_function!(unary Abs, WrappedAbs, f64::abs);
math_function!(unary Ceil, WrappedCeil, f64::ceil);
math_function!(unary Floor, WrappedFloor, f64::floor);
math_function!(unary Square, WrappedSquare, square);
math_function!(unary Cube, WrappedCube, cube);
math_function!(unary Negate, WrappedNegate, f64::neg);
math_function!(unary Round, WrappedRound, f64::round);
math_function!(unary Sign, WrappedSign, f64::signum);
math_function!(unary Sqrt, WrappedSqrt, f64::sqrt);
math_function!(unary Truncate, WrappedTruncate, f64::trunc);
math_function!(unary Reciprocal, WrappedReciprocal, f64::recip);
math_function!(unary Log, WrappedLog, f64::ln);
math_function!(unary Squeeze, WrappedSqueeze, squeeze);
math_function!(unary HalfNegative, WrappedHalfNegative, half_negative);
math_function!(unary QuarterNegative, WrappedQuarterNegative, quarter_negative);
math_function!(custom Clamp, WrappedClamp, f64::clamp, min: f64, max: f64);

#[inline(always)]
fn square(x: f64) -> f64 {
    x * x
}

#[inline(always)]
fn cube(x: f64) -> f64 {
    x * x * x
}

#[inline(always)]
fn squeeze(x: f64) -> f64 {
    x / 2.0 - x * x * x / 24.0
}

#[inline(always)]
fn half_negative(x: f64) -> f64 {
    if x.is_sign_negative() {
        x / 2.0
    } else {
        x
    }
}

#[inline(always)]
fn quarter_negative(x: f64) -> f64 {
    if x.is_sign_negative() {
        x / 4.0
    } else {
        x
    }
}
