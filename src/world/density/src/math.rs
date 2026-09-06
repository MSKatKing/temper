use std::ops::Neg;
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};

macro_rules! math_function {
    (binary $name:ident, $fun:expr) => {
        #[derive(Debug)]
        pub struct $name {
            pub left: BoxedDensityFunction,
            pub right: BoxedDensityFunction,
        }

        impl DensityFunction for $name {
            fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
                $fun(
                    self.left.compute(ctx),
                    self.right.compute(ctx),
                )
            }
        }
    };
    (unary $name:ident, $fun:expr) => {
        #[derive(Debug)]
        pub struct $name(pub BoxedDensityFunction);

        impl DensityFunction for $name {
            fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
                $fun(self.0.compute(ctx))
            }
        }
    };
    (custom $name:ident, $fun:expr, $($field:ident: $ty:ty),* $(,)?) => {
        #[derive(Debug)]
        pub struct $name {
            pub inner: BoxedDensityFunction,
            $(
            pub $field: $ty,
            )*
        }

        impl DensityFunction for $name {
            fn compute(&self, ctx: &mut DensityFunctionContext) -> f64 {
                $fun(self.inner.compute(ctx), $(self.$field),*)
            }
        }
    }
}

math_function!(binary Add, <f64 as std::ops::Add>::add);
math_function!(binary Sub, <f64 as std::ops::Sub>::sub);
math_function!(binary Mul, <f64 as std::ops::Mul>::mul);
math_function!(binary Div, <f64 as std::ops::Div>::div);
math_function!(binary Min, f64::min);
math_function!(binary Max, f64::max);
math_function!(unary Abs, f64::abs);
math_function!(unary Ceil, f64::ceil);
math_function!(unary Floor, f64::floor);
math_function!(unary Square, square);
math_function!(unary Cube, cube);
math_function!(unary Negate, f64::neg);
math_function!(unary Round, f64::round);
math_function!(unary Sign, f64::signum);
math_function!(unary Sqrt, f64::sqrt);
math_function!(unary Truncate, f64::trunc);
math_function!(unary Reciprocal, f64::recip);
math_function!(unary Log, f64::ln);
math_function!(unary Squeeze, squeeze);
math_function!(unary HalfNegative, half_negative);
math_function!(unary QuarterNegative, quarter_negative);
math_function!(custom Clamp, f64::clamp, min: f64, max: f64);

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
