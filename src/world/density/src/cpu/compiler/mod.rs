mod generic;

use crate::cpu::buffer::{BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::generic::{BufferAddVisitor, BufferFillVisitor, BufferOperationResult, BufferOperationVisitor, ConstantAddVisitor, ConstantFillVisitor, NoiseAddVisitor, NoiseFillVisitor, YClampedGradientVisitor};
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use crate::cpu::runtime::Operation;
use crate::{DensityFunction, DensityFunctionArgument};
use temper_core::random::{PositionalRandom, RandomSource};
use temper_data::noise::NoiseParameter;

pub struct Compiler {
    full_buffers: Vec<bool>,
    interpolated_buffers: Vec<bool>,
    flat_buffers: Vec<bool>,
    flat_cell_buffers: Vec<bool>,

    ops: Vec<Box<dyn Operation>>,
    constants: Vec<Box<dyn Operation>>,
}

pub struct CompiledDensityFunction {
    pub(crate) ops: Vec<Box<dyn Operation>>,
    pub(crate) full_buffer_count: usize,
    pub(crate) interpolated_buffer_count: usize,
    pub(crate) flat_buffer_count: usize,
    pub(crate) flat_cell_buffer_count: usize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
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

    pub fn copy_inner_type(&self, new_idx: usize) -> AnyBufferId {
        match self {
            Self::Full(_) => AnyBufferId::Full(BufferId::<Full>::new(new_idx)),
            Self::Interpolated(_) => AnyBufferId::Interpolated(BufferId::<Interpolated>::new(new_idx)),
            Self::Flat(_) => AnyBufferId::Flat(BufferId::<Flat>::new(new_idx)),
            Self::FlatCell(_) => AnyBufferId::FlatCell(BufferId::<FlatCell>::new(new_idx)),
        }
    }
}

trait ToAnyBufferId: BufferType {
    fn to_any_buffer_id(this: BufferId<Self>) -> AnyBufferId;
}

impl ToAnyBufferId for Full {
    fn to_any_buffer_id(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::Full(this)
    }
}

impl ToAnyBufferId for Interpolated {
    fn to_any_buffer_id(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::Interpolated(this)
    }
}

impl ToAnyBufferId for Flat {
    fn to_any_buffer_id(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::Flat(this)
    }
}

impl ToAnyBufferId for FlatCell {
    fn to_any_buffer_id(this: BufferId<Self>) -> AnyBufferId {
        AnyBufferId::FlatCell(this)
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
                compile(&mut this, &mut rand.fork_positional(), func.as_ref(), out.into())
            }
            DensityFunctionArgument::Constant(val) => {
                this.push_op(out.visit(ConstantFillVisitor {
                    src: val as f32,
                }))
            }
            DensityFunctionArgument::External(_) => {
                panic!("should be linked before being compiled")
            }
        };

        if actual != out {
            this.push_op(out.visit(BufferFillVisitor {
                src: actual,
            }));
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
            constants: Vec::new(),
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

    fn push_op(&mut self, op: BufferOperationResult) -> AnyBufferId {
        self.ops.push(op.op);
        op.output_buf
    }

    fn push_constant_op(&mut self, op: impl Operation + 'static) {
        self.constants.push(Box::new(op));
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
        DensityFunctionArgument::Function(func) => {
            match func.as_ref() {
                DensityFunction::Noise {
                    noise,
                    xz_scale,
                    y_scale
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
                        }
                    ))
                },
                func => {
                    ReturnValue::Buffer(compile(
                        compiler,
                        rand,
                        func,
                        parent_buffer,
                    ))
                }
            }
        }

    }
}

fn compile_add<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    left: &DensityFunctionArgument,
    right: &DensityFunctionArgument,
    parent_buffer: AnyBufferId,
) -> AnyBufferId {
    let left_source = compile_arg(compiler, rand, left, parent_buffer);

    let right_buffer = if let ReturnValue::Buffer(left) = &left_source && *left == parent_buffer {
        compiler.alloc_buffer(parent_buffer)
    } else {
        parent_buffer
    };
    let right_source = compile_arg(compiler, rand, right, right_buffer);

    if let ReturnValue::Buffer(left) = &left_source && *left != parent_buffer {
        compiler.free_buffer(*left);
    }

    if let ReturnValue::Buffer(right) = &right_source && *right != parent_buffer {
        compiler.free_buffer(*right);
    }

    match (left_source, right_source) {
        (ReturnValue::Constant(left), ReturnValue::Constant(right)) => {
            let left = compiler.push_op(parent_buffer.visit(ConstantFillVisitor {
                src: left
            }));

            compiler.push_op(left.visit(ConstantAddVisitor {
                src: right,
            }))
        }
        (ReturnValue::Constant(val), ReturnValue::Noise(noise))
            | (ReturnValue::Noise(noise), ReturnValue::Constant(val)) => {
            let noise = compiler.push_op(parent_buffer.visit(NoiseFillVisitor {
                src: noise,
            }));

            compiler.push_op(noise.visit(ConstantAddVisitor {
                src: val,
            }))
        }
        (ReturnValue::Noise(noise_a), ReturnValue::Noise(noise_b)) => {
            let noise_a = compiler.push_op(parent_buffer.visit(NoiseFillVisitor {
                src: noise_a,
            }));

            compiler.push_op(noise_a.visit(NoiseAddVisitor {
                src: noise_b,
            }))
        }
        (ReturnValue::Noise(noise), ReturnValue::Buffer(buffer))
            | (ReturnValue::Buffer(buffer), ReturnValue::Noise(noise)) => {
            compiler.push_op(buffer.visit(NoiseAddVisitor {
                src: noise,
            }))
        }
        (ReturnValue::Constant(val), ReturnValue::Buffer(buffer))
            | (ReturnValue::Buffer(buffer), ReturnValue::Constant(val)) => {
            compiler.push_op(buffer.visit(ConstantAddVisitor {
                src: val,
            }))
        }
        (ReturnValue::Buffer(left), ReturnValue::Buffer(right)) => {
            compiler.push_op(left.visit(BufferAddVisitor {
                src: right,
            }))
        }
    }
}

fn compile<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    func: &DensityFunction,
    parent_buffer: AnyBufferId,
) -> AnyBufferId {
    match func {
        DensityFunction::Add {
            left,
            right,
        } => compile_add(compiler, rand, left, right, parent_buffer),
        DensityFunction::YClampedGradient {
            from_y, to_y,
            from_value, to_value,
        } => {
            compiler.push_op(parent_buffer.visit(YClampedGradientVisitor {
                y_range: (*from_y as i16)..=(*to_y as i16),
                value_range: (*from_value as f32)..=(*to_value as f32),
            }))
        }
        _ => todo!("{:?}", func),
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
