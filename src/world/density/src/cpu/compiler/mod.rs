mod math;
mod visitor;

use crate::cpu::buffer::{BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::math::{
    compile_add, compile_div, compile_max, compile_min, compile_mul, compile_sub,
};
use crate::cpu::compiler::visitor::{AbsBufferVisitor, AbsNoiseVisitor, BufferOperationResult, BufferOperationVisitor, ClampBufferVisitor, ClampNoiseVisitor, FillBufferVisitor, FillConstantVisitor, FillNoiseVisitor, IntervalSelectVisitor, NegativeDecayBufferVisitor, NegativeDecayNoiseVisitor, PowBufferVisitor, PowNoiseVisitor, RangeChoiceVisitor, ShiftedNoiseVisitor, SplinePoint, SplineVisitor, SqueezeBufferVisitor, SqueezeNoiseVisitor, ValueOrBuffer, YClampedGradientVisitor};
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use crate::cpu::runtime::{NegativeDecayNoise, Operation};
use crate::{DensityFunction, DensityFunctionArgument, DensitySpline, ValueOrSpline};
use temper_core::random::{PositionalRandom, RandomSource};
use temper_data::noise::NoiseParameter;
use temper_noise::NormalNoise;

pub struct Compiler {
    full_buffers: Vec<bool>,
    interpolated_buffers: Vec<bool>,
    flat_buffers: Vec<bool>,
    flat_cell_buffers: Vec<bool>,

    ops: Vec<Box<dyn Operation>>,
}

#[derive(Debug)]
pub struct CompiledDensityFunction {
    pub(crate) ops: Vec<Box<dyn Operation>>,
    pub(crate) full_buffer_count: usize,
    pub(crate) interpolated_buffer_count: usize,
    pub(crate) flat_buffer_count: usize,
    pub(crate) flat_cell_buffer_count: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AnyBufferId {
    Full(BufferId<Full>),
    Interpolated(BufferId<Interpolated>),
    Flat(BufferId<Flat>),
    FlatCell(BufferId<FlatCell>),
}

enum ReturnValue {
    Buffer(AnyBufferId),
    Constant(f32),
    Noise(NoiseAccessor),
}

impl AnyBufferId {
    pub fn visit(self, visitor: impl BufferOperationVisitor) -> BufferOperationResult {
        match self {
            Self::Full(full) => visitor.visit_full(full),
            Self::Interpolated(interpolated) => visitor.visit_interpolated(interpolated),
            Self::Flat(flat) => visitor.visit_flat(flat),
            Self::FlatCell(flat) => visitor.visit_flat_cell(flat),
        }
    }

    pub fn idx(&self) -> usize {
        match self {
            Self::Full(id) => id.idx(),
            Self::Interpolated(id) => id.idx(),
            Self::Flat(id) => id.idx(),
            Self::FlatCell(id) => id.idx(),
        }
    }

    pub fn level(&self) -> usize {
        match self {
            Self::Full(_) => Full::LEVEL,
            Self::Interpolated(_) => Interpolated::LEVEL,
            Self::Flat(_) => Flat::LEVEL,
            Self::FlatCell(_) => FlatCell::LEVEL,
        }
    }

    pub fn copy_inner_type(&self, new_idx: usize) -> AnyBufferId {
        match self {
            Self::Full(_) => AnyBufferId::Full(BufferId::<Full>::new(new_idx)),
            Self::Interpolated(_) => {
                AnyBufferId::Interpolated(BufferId::<Interpolated>::new(new_idx))
            }
            Self::Flat(_) => AnyBufferId::Flat(BufferId::<Flat>::new(new_idx)),
            Self::FlatCell(_) => AnyBufferId::FlatCell(BufferId::<FlatCell>::new(new_idx)),
        }
    }
}

pub trait ToAnyBufferId: BufferType {
    fn convert_to_any(this: BufferId<Self>) -> AnyBufferId;
    fn try_downcast_to(any: AnyBufferId) -> Option<BufferId<Self>>;
}

impl ToAnyBufferId for Full {
    fn convert_to_any(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::Full(this)
    }

    fn try_downcast_to(any: AnyBufferId) -> Option<BufferId<Self>> {
        match any {
            AnyBufferId::Full(id) => Some(id),
            _ => None,
        }
    }
}

impl ToAnyBufferId for Interpolated {
    fn convert_to_any(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::Interpolated(this)
    }

    fn try_downcast_to(any: AnyBufferId) -> Option<BufferId<Self>> {
        match any {
            AnyBufferId::Interpolated(id) => Some(id),
            _ => None,
        }
    }
}

impl ToAnyBufferId for Flat {
    fn convert_to_any(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::Flat(this)
    }

    fn try_downcast_to(any: AnyBufferId) -> Option<BufferId<Self>> {
        match any {
            AnyBufferId::Flat(id) => Some(id),
            _ => None,
        }
    }
}

impl ToAnyBufferId for FlatCell {
    fn convert_to_any(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::FlatCell(this)
    }

    fn try_downcast_to(any: AnyBufferId) -> Option<BufferId<Self>> {
        match any {
            AnyBufferId::FlatCell(id) => Some(id),
            _ => None,
        }
    }
}

impl Compiler {
    pub fn compile<R: RandomSource>(
        rand: &mut R,
        func: DensityFunctionArgument,
    ) -> CompiledDensityFunction {
        let mut this = Compiler::new();

        let out = this.alloc_buffer(AnyBufferId::Full(BufferId::<Full>::new(0)));
        let actual = match func {
            DensityFunctionArgument::Function(func) => {
                compile(&mut this, &mut rand.fork_positional(), func.as_ref(), out)
            }
            DensityFunctionArgument::Constant(val) => {
                this.push_visitor(FillConstantVisitor::new(out, val as f32))
            }
            DensityFunctionArgument::External(_) => {
                panic!("should be linked before being compiled")
            }
        };

        if actual != out {
            this.push_visitor(FillBufferVisitor::new(out, actual));
        }

        CompiledDensityFunction {
            ops: this.ops,
            full_buffer_count: this.full_buffers.len(),
            interpolated_buffer_count: this.interpolated_buffers.len(),
            flat_buffer_count: this.flat_buffers.len(),
            flat_cell_buffer_count: this.flat_cell_buffers.len(),
        }
    }

    fn new() -> Compiler {
        Compiler {
            full_buffers: Vec::new(),
            interpolated_buffers: Vec::new(),
            flat_buffers: Vec::new(),
            flat_cell_buffers: Vec::new(),

            ops: Vec::new(),
        }
    }

    fn alloc_buffer(&mut self, buffer: AnyBufferId) -> AnyBufferId {
        let buffers = match buffer {
            AnyBufferId::Full(_) => &mut self.full_buffers,
            AnyBufferId::Interpolated(_) => &mut self.interpolated_buffers,
            AnyBufferId::Flat(_) => &mut self.flat_buffers,
            AnyBufferId::FlatCell(_) => &mut self.flat_cell_buffers,
        };

        for (i, used) in buffers.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return buffer.copy_inner_type(i);
            }
        }

        let id = buffers.len();
        buffers.push(true);
        buffer.copy_inner_type(id)
    }

    fn free_buffer(&mut self, buffer: AnyBufferId) {
        (match buffer {
            AnyBufferId::Full(_) => &mut self.full_buffers,
            AnyBufferId::Interpolated(_) => &mut self.interpolated_buffers,
            AnyBufferId::Flat(_) => &mut self.flat_buffers,
            AnyBufferId::FlatCell(_) => &mut self.flat_cell_buffers,
        })[buffer.idx()] = false;
    }

    fn push_visitor(&mut self, op: BufferOperationResult) -> AnyBufferId {
        self.ops.push(op.op);
        op.output_buf
    }
}

fn compile_arg<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    argument: &DensityFunctionArgument,
    parent_buffer: AnyBufferId,
) -> ReturnValue {
    match argument {
        DensityFunctionArgument::External(name) => panic!("unlinked function! {name}"),
        DensityFunctionArgument::Constant(val) => ReturnValue::Constant(*val as f32),
        DensityFunctionArgument::Function(func) => match func.as_ref() {
            DensityFunction::Noise {
                noise,
                xz_scale,
                y_scale,
            } => {
                let params = NoiseParameter::get_by_name(noise.as_str())
                    .unwrap_or_else(|| panic!("unknown noise parameter: {}", noise.as_str()));

                ReturnValue::Noise(NoiseAccessor::new(
                    params,
                    rand,
                    noise.as_str(),
                    NoiseAccessType::Basic {
                        xz_scale: *xz_scale as f32,
                        y_scale: *y_scale as f32,
                    },
                ))
            }
            DensityFunction::Shift { noise } => {
                let params = NoiseParameter::get_by_name(noise.as_str())
                    .unwrap_or_else(|| panic!("unknown noise parameter: {}", noise.as_str()));

                ReturnValue::Noise(NoiseAccessor::new(
                    params,
                    rand,
                    noise.as_str(),
                    NoiseAccessType::Shift,
                ))
            }
            DensityFunction::ShiftA { noise } => {
                let params = NoiseParameter::get_by_name(noise.as_str())
                    .unwrap_or_else(|| panic!("unknown noise parameter: {}", noise.as_str()));

                ReturnValue::Noise(NoiseAccessor::new(
                    params,
                    rand,
                    noise.as_str(),
                    NoiseAccessType::ShiftA,
                ))
            }
            DensityFunction::ShiftB { noise } => {
                let params = NoiseParameter::get_by_name(noise.as_str())
                    .unwrap_or_else(|| panic!("unknown noise parameter: {}", noise.as_str()));

                ReturnValue::Noise(NoiseAccessor::new(
                    params,
                    rand,
                    noise.as_str(),
                    NoiseAccessType::ShiftB,
                ))
            }
            DensityFunction::OldBlendedNoise { xz_scale, xz_factor, y_scale, y_factor, smear_scale_multiplier } => {
                ReturnValue::Noise(NoiseAccessor::new_blended(
                    *xz_scale,
                    *y_scale,
                    *xz_factor,
                    *y_factor,
                    *smear_scale_multiplier,
                ))
            }
            DensityFunction::CacheAllInCell { input } => {
                compile_arg(compiler, rand, input, parent_buffer)
            }
            DensityFunction::CacheOnce { input } => {
                compile_arg(compiler, rand, input, parent_buffer)
            } // TODO: make this compile a global instead
            func => ReturnValue::Buffer(compile(compiler, rand, func, parent_buffer)),
        },
    }
}

fn compile<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    func: &DensityFunction,
    parent_buffer: AnyBufferId,
) -> AnyBufferId {
    match func {
        DensityFunction::Add { left, right } => {
            compile_add(compiler, rand, left, right, parent_buffer)
        }
        DensityFunction::Mul { left, right } => {
            compile_mul(compiler, rand, left, right, parent_buffer)
        }
        DensityFunction::Min { left, right } => {
            compile_min(compiler, rand, left, right, parent_buffer)
        }
        DensityFunction::Max { left, right } => {
            compile_max(compiler, rand, left, right, parent_buffer)
        }
        DensityFunction::Sub { left, right } => {
            compile_sub(compiler, rand, left, right, parent_buffer)
        }
        DensityFunction::Div { left, right } => {
            compile_div(compiler, rand, left, right, parent_buffer)
        }
        DensityFunction::YClampedGradient {
            from_y,
            to_y,
            from_value,
            to_value,
        } => compiler.push_visitor(YClampedGradientVisitor::new(
            parent_buffer,
            (*from_y as i16)..=(*to_y as i16),
            (*from_value as f32)..=(*to_value as f32),
        )),
        DensityFunction::Cache2d { input } => {
            let buffer = if parent_buffer.level() >= Flat::LEVEL {
                parent_buffer
            } else {
                compiler.alloc_buffer(AnyBufferId::Flat(BufferId::<Flat>::new(0)))
            };

            let input_source = compile_arg(compiler, rand, input, buffer);

            match input_source {
                ReturnValue::Constant(val) => {
                    compiler.push_visitor(FillConstantVisitor::new(buffer, val))
                }
                ReturnValue::Noise(noise) => {
                    compiler.push_visitor(FillNoiseVisitor::new(buffer, noise))
                }
                ReturnValue::Buffer(actual) => {
                    if actual != buffer {
                        compiler.free_buffer(buffer);
                    }

                    actual
                }
            }
        }
        DensityFunction::Interpolated { input } => {
            let buffer = if parent_buffer.level() >= Interpolated::LEVEL {
                parent_buffer
            } else {
                compiler.alloc_buffer(AnyBufferId::Interpolated(BufferId::<Interpolated>::new(0)))
            };

            let input_source = compile_arg(compiler, rand, input, buffer);

            match input_source {
                ReturnValue::Constant(val) => {
                    compiler.push_visitor(FillConstantVisitor::new(buffer, val))
                }
                ReturnValue::Noise(noise) => {
                    compiler.push_visitor(FillNoiseVisitor::new(buffer, noise))
                }
                ReturnValue::Buffer(actual) => {
                    if actual != buffer {
                        compiler.free_buffer(buffer);
                    }

                    actual
                }
            }
        }
        DensityFunction::FlatCache { input } => {
            let buffer = if parent_buffer.level() >= FlatCell::LEVEL {
                parent_buffer
            } else {
                compiler.alloc_buffer(AnyBufferId::FlatCell(BufferId::<FlatCell>::new(0)))
            };

            let input_source = compile_arg(compiler, rand, input, buffer);

            match input_source {
                ReturnValue::Constant(val) => {
                    compiler.push_visitor(FillConstantVisitor::new(buffer, val))
                }
                ReturnValue::Noise(noise) => {
                    compiler.push_visitor(FillNoiseVisitor::new(buffer, noise))
                }
                ReturnValue::Buffer(actual) => {
                    if actual != buffer {
                        compiler.free_buffer(buffer);
                    }

                    actual
                }
            }
        }
        DensityFunction::ShiftedNoise {
            noise,
            xz_scale,
            y_scale,
            shift_x,
            shift_y,
            shift_z,
        } => {
            let x = compiler.alloc_buffer(parent_buffer);
            let actual_x = compile_arg(compiler, rand, shift_x, x);

            let y = compiler.alloc_buffer(parent_buffer);
            let actual_y = compile_arg(compiler, rand, shift_y, y);

            let z = compiler.alloc_buffer(parent_buffer);
            let actual_z = compile_arg(compiler, rand, shift_z, z);

            let actual_x = match actual_x {
                ReturnValue::Constant(val) => {
                    compiler.push_visitor(FillConstantVisitor::new(x, val))
                }
                ReturnValue::Noise(val) => compiler.push_visitor(FillNoiseVisitor::new(x, val)),
                ReturnValue::Buffer(buf) => {
                    if buf.level() > parent_buffer.level() {
                        compiler.push_visitor(FillBufferVisitor::new(x, buf))
                    } else {
                        x
                    }
                }
            };

            let actual_y = match actual_y {
                ReturnValue::Constant(val) => {
                    compiler.push_visitor(FillConstantVisitor::new(y, val))
                }
                ReturnValue::Noise(val) => compiler.push_visitor(FillNoiseVisitor::new(y, val)),
                ReturnValue::Buffer(buf) => {
                    if buf.level() > parent_buffer.level() {
                        compiler.push_visitor(FillBufferVisitor::new(y, buf))
                    } else {
                        y
                    }
                }
            };

            let actual_z = match actual_z {
                ReturnValue::Constant(val) => {
                    compiler.push_visitor(FillConstantVisitor::new(z, val))
                }
                ReturnValue::Noise(val) => compiler.push_visitor(FillNoiseVisitor::new(z, val)),
                ReturnValue::Buffer(buf) => {
                    if buf.level() > parent_buffer.level() {
                        compiler.push_visitor(FillBufferVisitor::new(z, buf))
                    } else {
                        z
                    }
                }
            };

            let params = NoiseParameter::get_by_name(noise.as_str())
                .unwrap_or_else(|| panic!("Unknown noise parameter: {}", noise.as_str()));

            let noise = NormalNoise::new(
                &mut rand.spawn_from_hash(noise.as_str()),
                params.first_octave,
                params.amplitudes,
            );

            compiler.push_visitor(ShiftedNoiseVisitor::new(
                parent_buffer,
                noise,
                *xz_scale as f32,
                *y_scale as f32,
                actual_x,
                actual_y,
                actual_z,
            ))
        }
        DensityFunction::OldBlendedNoise { xz_scale, xz_factor, y_scale, y_factor, smear_scale_multiplier } => {
            compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, NoiseAccessor::new_blended(*xz_scale, *y_scale, *xz_factor, *y_factor, *smear_scale_multiplier)))
        }
        DensityFunction::Squeeze { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(
                    parent_buffer,
                    val / 2.0 - val.powi(3) / 24.0
                )),
                ReturnValue::Noise(val) => compiler.push_visitor(SqueezeNoiseVisitor::new(
                    parent_buffer,
                    val,
                )),
                ReturnValue::Buffer(buf) => compiler.push_visitor(SqueezeBufferVisitor::new(
                    buf,
                )),
            }
        }
        DensityFunction::RangeChoice { input, when_in_range, when_out_of_range, min_inclusive, max_exclusive } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            let dst = match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val)),
                ReturnValue::Noise(noise) => compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise)),
                ReturnValue::Buffer(buf) => {
                    buf
                }
            };

            let in_range_buf = compiler.alloc_buffer(dst);
            let when_in_range = compile_arg(compiler, rand, when_in_range, in_range_buf);

            let out_of_range_buf = compiler.alloc_buffer(dst);
            let when_out_of_range = compile_arg(compiler, rand, when_out_of_range, out_of_range_buf);

            let when_in_range = match when_in_range {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(in_range_buf, val)),
                ReturnValue::Noise(noise) => compiler.push_visitor(FillNoiseVisitor::new(in_range_buf, noise)),
                ReturnValue::Buffer(buf) => {
                    if buf != in_range_buf {
                        compiler.free_buffer(in_range_buf)
                    }

                    buf
                }
            };

            let when_out_of_range = match when_out_of_range {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(out_of_range_buf, val)),
                ReturnValue::Noise(noise) => compiler.push_visitor(FillNoiseVisitor::new(out_of_range_buf, noise)),
                ReturnValue::Buffer(buf) => {
                    if buf != out_of_range_buf {
                        compiler.free_buffer(out_of_range_buf)
                    }

                    buf
                }
            };

            compiler.free_buffer(when_in_range);
            compiler.free_buffer(when_out_of_range);

            compiler.push_visitor(RangeChoiceVisitor::new(dst, when_in_range, when_out_of_range, (*min_inclusive as f32)..(*max_exclusive as f32)))
        }
        DensityFunction::Abs { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val.abs())),
                ReturnValue::Noise(noise) => compiler.push_visitor(AbsNoiseVisitor::new(parent_buffer, noise)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(AbsBufferVisitor::new(buf)),
            }
        }
        DensityFunction::Clamp { input, min, max } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val.clamp(*min as f32, *max as f32))),
                ReturnValue::Noise(noise) => compiler.push_visitor(ClampNoiseVisitor::new(parent_buffer, noise, *min as f32, *max as f32)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(ClampBufferVisitor::new(buf, *min as f32, *max as f32)),
            }
        }
        DensityFunction::Square { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val.powi(2))),
                ReturnValue::Noise(noise) => compiler.push_visitor(PowNoiseVisitor::new(parent_buffer, noise, 2)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(PowBufferVisitor::new(buf, 2)),
            }
        }
        DensityFunction::Cube { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val.powi(3))),
                ReturnValue::Noise(noise) => compiler.push_visitor(PowNoiseVisitor::new(parent_buffer, noise, 3)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(PowBufferVisitor::new(buf, 3)),
            }
        }
        DensityFunction::Invert { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val.powi(-1))),
                ReturnValue::Noise(noise) => compiler.push_visitor(PowNoiseVisitor::new(parent_buffer, noise, -1)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(PowBufferVisitor::new(buf, -1)),
            }
        }
        DensityFunction::IntervalSelect { input, thresholds, functions } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            let buf = match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val)),
                ReturnValue::Noise(noise) => compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise)),
                ReturnValue::Buffer(buf) => buf,
            };

            let mut buffers = Vec::with_capacity(functions.len());
            for function in functions {
                let buf = compiler.alloc_buffer(buf);
                let actual = compile_arg(compiler, rand, function, buf);

                let actual = match actual {
                    ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(buf, val)),
                    ReturnValue::Noise(noise) => compiler.push_visitor(FillNoiseVisitor::new(buf, noise)),
                    ReturnValue::Buffer(actual) => {
                        let actual = if actual.level() != buf.level() {
                            compiler.free_buffer(actual);
                            compiler.push_visitor(FillBufferVisitor::new(buf, actual))
                        } else {
                            actual
                        };

                        if actual != buf {
                            compiler.free_buffer(buf)
                        }

                        actual
                    },
                };

                buffers.push(actual);
            }

            buffers.iter().for_each(|buf| compiler.free_buffer(*buf));

            let thresholds = thresholds.iter().map(|v| *v as f32).collect::<Vec<_>>();

            compiler.push_visitor(IntervalSelectVisitor::new(buf, thresholds, buffers))
        },
        DensityFunction::Spline { spline } => compile_spline(compiler, rand, spline, parent_buffer),
        DensityFunction::HalfNegative { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, if val.is_sign_negative() { val / 2.0 } else { val })),
                ReturnValue::Noise(noise) => compiler.push_visitor(NegativeDecayNoiseVisitor::new(parent_buffer, noise, 2.0)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(NegativeDecayBufferVisitor::new(buf, 2.0)),
            }
        }
        DensityFunction::QuarterNegative { input } => {
            let input = compile_arg(compiler, rand, input, parent_buffer);

            match input {
                ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, if val.is_sign_negative() { val / 4.0 } else { val })),
                ReturnValue::Noise(noise) => compiler.push_visitor(NegativeDecayNoiseVisitor::new(parent_buffer, noise, 4.0)),
                ReturnValue::Buffer(buf) => compiler.push_visitor(NegativeDecayBufferVisitor::new(buf, 4.0)),
            }
        }
        _ => todo!("{:?}", func),
    }
}

fn compile_spline<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    spline: &DensitySpline,
    parent_buffer: AnyBufferId,
) -> AnyBufferId {
    let input = compile_arg(compiler, rand, &spline.coordinate, parent_buffer);

    let input = match input {
        ReturnValue::Constant(val) => compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val)),
        ReturnValue::Noise(noise) => compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise)),
        ReturnValue::Buffer(buf) => buf,
    };

    let mut points = Vec::with_capacity(spline.points.len());
    for point in spline.points.iter() {
        let buf = compiler.alloc_buffer(input);

        let value = match &point.value {
            ValueOrSpline::Value(v) => {
                compiler.free_buffer(buf);
                ValueOrBuffer::Value(*v as f32)
            }
            ValueOrSpline::Spline(spline) => {
                let actual = compile_spline(compiler, rand, spline, buf);
                if actual != buf {
                    compiler.push_visitor(FillBufferVisitor::new(buf, actual));
                }

                ValueOrBuffer::Buffer(buf)
            }
        };

        points.push(SplinePoint {
            location: point.location as f32,
            derivative: point.derivative as f32,
            value,
        })
    }

    points.iter().for_each(|buf| {
        if let ValueOrBuffer::Buffer(buf) = &buf.value {
            compiler.free_buffer(*buf);
        }
    });

    points.sort_by(|point_a, point_b| point_a.location.total_cmp(&point_b.location));

    compiler.push_visitor(SplineVisitor::new(input, points))
}

#[cfg(test)]
mod tests {
    // TODO
}
