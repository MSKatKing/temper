mod math;
mod visitor;

use crate::cpu::buffer::{BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::math::{compile_add, compile_mul};
use crate::cpu::compiler::visitor::{
    BufferOperationResult, BufferOperationVisitor, FillBufferVisitor, FillConstantVisitor,
    FillNoiseVisitor, YClampedGradientVisitor,
};
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
}

#[derive(Debug)]
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
            Self::Interpolated(_) => {
                AnyBufferId::Interpolated(BufferId::<Interpolated>::new(new_idx))
            }
            Self::Flat(_) => AnyBufferId::Flat(BufferId::<Flat>::new(new_idx)),
            Self::FlatCell(_) => AnyBufferId::FlatCell(BufferId::<FlatCell>::new(new_idx)),
        }
    }
}

pub trait ToAnyBufferId: BufferType {
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
            let buffer = compiler.alloc_buffer(AnyBufferId::Flat(BufferId::<Flat>::new(0)));
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
        _ => todo!("{:?}", func),
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
