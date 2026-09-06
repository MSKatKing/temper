use std::collections::HashMap;
use temper_core::pos::BlockPos;
use temper_core::random::{PositionalRandom, RandomSource};
use temper_noise::{BlendedNoise, NormalNoise};
use temper_noise::params::NoiseParameter;
use crate::{BoxedDensityFunction, Constant, DensityFunctionContext};
use crate::conditional::{IntervalSelect, RangeChoice};
use crate::json::{DensityFunction, DensityFunctionArgument};
use crate::mapped::{Axis, Gradient, Tiling};
use crate::marker::{Cache2d, CacheAllInCell, CacheOnce, FlatCache, Interpolated};
use crate::math::{Abs, Add, Clamp, Cube, Div, HalfNegative, Max, Min, Mul, Negate, QuarterNegative, Reciprocal, Round, Square, Squeeze, Sub, Truncate};
use crate::noise::{Noise, OldBlendedNoise, Shift, ShiftA, ShiftB};

pub struct CompiledDensityFunction {
    root: BoxedDensityFunction,
    pub(super) num_ctx: usize,
}

pub struct Compiler<'a> {
    num_ctx: usize,
    externals: &'a HashMap<String, DensityFunctionArgument>,
}

impl CompiledDensityFunction {
    pub fn execute(&self, pos: BlockPos) -> f64 {
        let mut ctx = DensityFunctionContext::new(pos, self);
        self.root.compute(&mut ctx)
    }
}

impl Compiler<'_> {
    pub fn compile<R: RandomSource, P: PositionalRandom<R>>(rand: &mut P, externals: &HashMap<String, DensityFunctionArgument>, func: DensityFunctionArgument) -> CompiledDensityFunction {
        let mut this = Compiler {
            num_ctx: 0,
            externals,
        };

        let compiled = compile_arg(&mut this, rand, &func);

        CompiledDensityFunction {
            root: compiled,
            num_ctx: this.num_ctx,
        }
    }

    fn get_next_ctx(&mut self) -> usize {
        self.num_ctx += 1;
        self.num_ctx - 1
    }
}

fn compile_arg<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    arg: &DensityFunctionArgument,
) -> BoxedDensityFunction {
    match arg {
        DensityFunctionArgument::Constant(val) => Box::new(Constant(*val)),
        DensityFunctionArgument::Function(val) => compile(compiler, rand, val.as_ref()),
        DensityFunctionArgument::External(val) => compile_arg(
            compiler,
            rand,
            compiler.externals.get(val).expect("missing"),
        )
    }
}

fn compile<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    func: &DensityFunction,
) -> BoxedDensityFunction {
    match func {
        DensityFunction::Cache2d { input } => {
            Box::new(Cache2d(compile_arg(compiler, rand, input), compiler.get_next_ctx()))
        },
        DensityFunction::CacheAllInCell { input } => {
            Box::new(CacheAllInCell(compile_arg(compiler, rand, input)))
        },
        DensityFunction::CacheOnce { input } => {
            Box::new(CacheOnce(compile_arg(compiler, rand, input), compiler.get_next_ctx()))
        },
        DensityFunction::FlatCache { input } => {
            Box::new(FlatCache(compile_arg(compiler, rand, input), compiler.get_next_ctx()))
        },
        DensityFunction::Interpolated { input } => {
            Box::new(Interpolated(compile_arg(compiler, rand, input)))
        },
        DensityFunction::Noise { noise, xz_scale, y_scale } => {
            Box::new(Noise {
                noise: NormalNoise::new(&mut rand.spawn_from_hash(noise.strip_prefix("minecraft:").unwrap_or(noise.as_str())), NoiseParameter::get_by_name(noise).expect("unknown noise")),
                xz_scale: *xz_scale,
                y_scale: *y_scale,
                shift_x: None,
                shift_y: None,
                shift_z: None,
            })
        },
        DensityFunction::OldBlendedNoise { xz_scale, y_scale, xz_factor, y_factor, smear_scale_multiplier } => {
            Box::new(OldBlendedNoise(BlendedNoise::new_unseeded(*xz_scale, *y_scale, *xz_factor, *y_factor, *smear_scale_multiplier)))
        },
        DensityFunction::Shift { noise } => {
            Box::new(Shift(NormalNoise::new(&mut rand.spawn_from_hash(noise.strip_prefix("minecraft:").unwrap_or(noise.as_str())), NoiseParameter::get_by_name(noise).expect("unknown noise"))))
        },
        DensityFunction::ShiftA { noise } => {
            Box::new(ShiftA(NormalNoise::new(&mut rand.spawn_from_hash(noise.strip_prefix("minecraft:").unwrap_or(noise.as_str())), NoiseParameter::get_by_name(noise).expect("unknown noise"))))
        },
        DensityFunction::ShiftB { noise } => {
            Box::new(ShiftB(NormalNoise::new(&mut rand.spawn_from_hash(noise.strip_prefix("minecraft:").unwrap_or(noise.as_str())), NoiseParameter::get_by_name(noise).expect("unknown noise"))))
        },
        DensityFunction::ShiftedNoise { noise, xz_scale, y_scale, shift_x, shift_y, shift_z } => {
            Box::new(Noise {
                noise: NormalNoise::new(&mut rand.spawn_from_hash(noise.strip_prefix("minecraft:").unwrap_or(noise.as_str())), NoiseParameter::get_by_name(noise).expect("unknown noise")),
                xz_scale: *xz_scale,
                y_scale: *y_scale,
                shift_x: Some(compile_arg(compiler, rand, shift_x)),
                shift_y: Some(compile_arg(compiler, rand, shift_y)),
                shift_z: Some(compile_arg(compiler, rand, shift_z)),
            })
        },
        DensityFunction::Abs { input } => {
            Box::new(Abs(compile_arg(compiler, rand, input)))
        },
        DensityFunction::Add { left, right } => {
            Box::new(Add {
                left: compile_arg(compiler, rand, left),
                right: compile_arg(compiler, rand, right),
            })
        },
        DensityFunction::Ceil { .. } => todo!(),
        DensityFunction::Clamp { input, min, max } => {
            Box::new(Clamp {
                inner: compile_arg(compiler, rand, input),
                min: *min,
                max: *max,
            })
        },
        DensityFunction::Constant { value } => Box::new(Constant(*value)),
        DensityFunction::Cube { input } => {
            Box::new(Cube(compile_arg(compiler, rand, input)))
        },
        DensityFunction::Div { left, right } => {
            Box::new(Div {
                left: compile_arg(compiler, rand, left),
                right: compile_arg(compiler, rand, right),
            })
        },
        DensityFunction::Floor { .. } => todo!(),
        DensityFunction::Invert { input } => Box::new(Reciprocal(compile_arg(compiler, rand, input))),
        DensityFunction::Mul { left, right } => Box::new(Mul {
            left: compile_arg(compiler, rand, left),
            right: compile_arg(compiler, rand, right),
        }),
        DensityFunction::Min { left, right } => Box::new(Min {
            left: compile_arg(compiler, rand, left),
            right: compile_arg(compiler, rand, right),
        }),
        DensityFunction::Max { left, right } => Box::new(Max {
            left: compile_arg(compiler, rand, left),
            right: compile_arg(compiler, rand, right),
        }),
        DensityFunction::Negate { input } => Box::new(Negate(compile_arg(compiler, rand, input))),
        DensityFunction::Round { .. } => todo!(),
        DensityFunction::Sub { left, right } => Box::new(Sub {
            left: compile_arg(compiler, rand, left),
            right: compile_arg(compiler, rand, right),
        }),
        DensityFunction::Square { input } => Box::new(Square(compile_arg(compiler, rand, input))),
        DensityFunction::Truncate { .. } => todo!(),
        DensityFunction::YClampedGradient { from_y, to_y, from_value, to_value } => Box::new(Gradient {
            axis: Axis::Y,
            tiling: Tiling::ClampToEdge,
            from_coord: *from_y,
            to_coord: *to_y,
            from_value: *from_value,
            to_value: *to_value,
        }),
        DensityFunction::Squeeze { input } => Box::new(Squeeze(compile_arg(compiler, rand, input))),
        DensityFunction::Spline { .. } => todo!(),
        DensityFunction::HalfNegative { input } => Box::new(HalfNegative(compile_arg(compiler, rand, input))),
        DensityFunction::QuarterNegative { input } => Box::new(QuarterNegative(compile_arg(compiler, rand, input))),
        DensityFunction::IntervalSelect { input, thresholds, functions } => Box::new(IntervalSelect {
            input: compile_arg(compiler, rand, input),
            thresholds: thresholds.clone(),
            functions: functions.iter().map(|v| compile_arg(compiler, rand, v)).collect(),
        }),
        DensityFunction::RangeChoice { input, min_inclusive, max_exclusive, when_in_range, when_out_of_range } => {
            Box::new(RangeChoice {
                input: compile_arg(compiler, rand, input),
                when_in_range: compile_arg(compiler, rand, when_in_range),
                when_out_range: compile_arg(compiler, rand, when_out_of_range),
                range: (*min_inclusive)..(*max_exclusive)
            })
        },
        DensityFunction::Beardifier => Box::new(Constant(0.0)),
        DensityFunction::BlendAlpha => Box::new(Constant(1.0)),
        DensityFunction::BlendOffset => Box::new(Constant(0.0)),
        DensityFunction::BlendDensity { input } => compile_arg(compiler, rand, input),
        _ => todo!("{:?}", func)
    }
}
