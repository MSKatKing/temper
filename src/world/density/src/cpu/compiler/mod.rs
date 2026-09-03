mod math;

use crate::cpu::buffer::{BufferApplyTo, BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::math::{compile_max, compile_min, compile_mul, compile_sub};
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use crate::cpu::runtime::{BufferAdd, ConstantAdd, FillBuffer, FillConstant, Operation};
use crate::{DensityFunction, DensityFunctionArgument};
use temper_core::random::{PositionalRandom, RandomSource};
use temper_data::noise::NoiseParameter;
use crate::cpu::workspace::{GetDstSrc, WorkspaceStorable};

trait CompilerStorable: BufferType {
    fn storage(compiler: &Compiler) -> &Vec<bool>;
    fn storage_mut(compiler: &mut Compiler) -> &mut Vec<bool>;
}

impl CompilerStorable for Full {
    fn storage(compiler: &Compiler) -> &Vec<bool> {
        &compiler.full_buffers
    }

    fn storage_mut(compiler: &mut Compiler) -> &mut Vec<bool> {
        &mut compiler.full_buffers
    }
}

impl CompilerStorable for Interpolated {
    fn storage(compiler: &Compiler) -> &Vec<bool> {
        &compiler.interpolated_buffers
    }

    fn storage_mut(compiler: &mut Compiler) -> &mut Vec<bool> {
        &mut compiler.interpolated_buffers
    }
}

impl CompilerStorable for Flat {
    fn storage(compiler: &Compiler) -> &Vec<bool> {
        &compiler.flat_buffers
    }

    fn storage_mut(compiler: &mut Compiler) -> &mut Vec<bool> {
        &mut compiler.flat_buffers
    }
}

impl CompilerStorable for FlatCell {
    fn storage(compiler: &Compiler) -> &Vec<bool> {
        &compiler.flat_cell_buffers
    }

    fn storage_mut(compiler: &mut Compiler) -> &mut Vec<bool> {
        &mut compiler.flat_cell_buffers
    }
}

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

#[derive(Copy, Clone)]
pub enum AnyBufferId {
    Full(BufferId<Full>),
    Interpolated(BufferId<Interpolated>),
    Flat(BufferId<Flat>),
    FlatCell(BufferId<FlatCell>),
}

impl From<BufferId<Full>> for AnyBufferId {
    fn from(value: BufferId<Full>) -> Self {
        Self::Full(value)
    }
}

impl From<BufferId<Interpolated>> for AnyBufferId {
    fn from(value: BufferId<Interpolated>) -> Self {
        Self::Interpolated(value)
    }
}

impl From<BufferId<Flat>> for AnyBufferId {
    fn from(value: BufferId<Flat>) -> Self {
        Self::Flat(value)
    }
}

impl From<BufferId<FlatCell>> for AnyBufferId {
    fn from(value: BufferId<FlatCell>) -> Self {
        Self::FlatCell(value)
    }
}

impl Compiler {
    pub fn compile<R: RandomSource>(
        rand: &mut R,
        func: DensityFunctionArgument,
    ) -> CompiledDensityFunction {
        let mut this = Compiler::new();

        let out = this.alloc_buffer::<Full>();
        let actual = match func {
            DensityFunctionArgument::Function(func) => {
                compile(&mut this, &mut rand.fork_positional(), func.as_ref(), out.into())
            }
            DensityFunctionArgument::Constant(val) => {
                this.push_op(FillConstant {
                    dst: out,
                    src: val as f32,
                });

                out
            }
            DensityFunctionArgument::External(_) => {
                panic!("should be linked before being compiled")
            }
        };

        if let AnyBufferId::Full(actual) = actual {
            if actual == out {
                return CompiledDensityFunction {
                    ops: this.ops,
                    full_buffer_count: this.full_buffers.len(),
                    interpolated_buffer_count: this.interpolated_buffers.len(),
                    flat_buffer_count: this.flat_buffers.len(),
                    flat_cell_buffer_count: this.flat_cell_buffers.len(),
                };
            }
        }

        this.push_op(FillBuffer {
            dst: out,
            src: actual,
        });

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

    fn alloc_buffer<T: CompilerStorable>(&mut self) -> BufferId<T> {
        let buffers = T::storage_mut(self);

        for (i, used) in buffers.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return BufferId::<T>::new(i);
            }
        }

        let id = buffers.len();
        buffers.push(true);
        BufferId::<T>::new(id)
    }

    fn free_buffer<T: CompilerStorable>(&mut self, buffer: BufferId<T>) {
        T::storage_mut(self)[buffer.idx()] = false;
    }

    fn push_op(&mut self, op: impl Operation + 'static) {
        self.ops.push(Box::new(op));
    }

    fn push_constant_op(&mut self, op: impl Operation + 'static) {
        self.constants.push(Box::new(op));
    }
}

fn compile<R: RandomSource, P: PositionalRandom<R>, T: CompilerStorable + WorkspaceStorable, U: WorkspaceStorable + CompilerStorable + BufferApplyTo<T> + GetDstSrc<T>>(
    compiler: &mut Compiler,
    rand: &mut P,
    func: &DensityFunction,
    parent_buffer: BufferId<T>,
) -> BufferId<impl BufferApplyTo<T>> {
    match func {
        DensityFunction::Add {
            left: DensityFunctionArgument::Constant(val),
            right: DensityFunctionArgument::Function(func),
        } | DensityFunction::Add {
            left: DensityFunctionArgument::Function(func),
            right: DensityFunctionArgument::Constant(val),
        } => {
            let out = compile(compiler, rand, func, parent_buffer);

            compiler.push_op(ConstantAdd {
                dst: out,
                src: *val as f32,
            });

            out
        },
        DensityFunction::Add {
            left: DensityFunctionArgument::Function(left),
            right: DensityFunctionArgument::Function(right),
        } => {
            let left = compile(compiler, rand, left, parent_buffer);

            let right_buf = compiler.alloc_buffer::<T>();
            let right = compile(compiler, rand, right, right_buf);

            compiler.push_op(BufferAdd {
                dst: left,
                src: right,
            });

            left
        },
        DensityFunction::Add { .. } => panic!("should be folded and linked"),
        _ => todo!("{:?}", func),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DensityFunctionArgument;
    use temper_core::random::XoroshiroRandomSource;

    #[test]
    fn test_compile_shift() {
        let mut compiler = Compiler::new();
        let mut rand = XoroshiroRandomSource::new(0);

        let func = DensityFunction::Shift {
            noise: "minecraft:aquifer_barrier".to_string(),
        };

        let parent = BufferId {
            ty: BufferType::Out,
            id: 0,
        };
        let out = compile(&mut compiler, &mut rand.fork_positional(), &func, parent);

        assert_eq!(parent, out);
        assert_eq!(compiler.ops.len(), 1);
        assert!(compiler.ops.last().is_some());

        let Some(Operation::ClearBuffer {
            destination,
            source,
        }) = compiler.ops.last()
        else {
            panic!("last operation was not a clear buffer operation")
        };

        let ValueSource::Noise(accessor) = source else {
            panic!("clear buffer operation's source was not a noise source")
        };

        assert_eq!(*destination, parent);
        assert_eq!(accessor.access_type, NoiseAccessType::Shift);
    }

    #[test]
    fn test_compile_add() {
        let mut compiler = Compiler::new();
        let mut rand = XoroshiroRandomSource::new(0);

        let func = DensityFunction::Add {
            left: DensityFunctionArgument::Function(Box::new(DensityFunction::Shift {
                noise: "minecraft:aquifer_barrier".to_string(),
            })),
            right: DensityFunctionArgument::Function(Box::new(DensityFunction::Shift {
                noise: "aquifer_barrier".to_string(),
            })),
        };

        let parent = BufferId {
            ty: BufferType::Out,
            id: 0,
        };
        let out = compile(&mut compiler, &mut rand.fork_positional(), &func, parent);

        assert_eq!(parent, out);
        assert_eq!(compiler.ops.len(), 2);
        assert!(compiler.ops.last().is_some());
        assert!(matches!(
            compiler.ops.last().unwrap(),
            Operation::AddBuffer { .. }
        ));
    }

    #[test]
    #[should_panic = "functions should be folded prior to being compiled"]
    fn test_compile_add_no_fold() {
        let mut compiler = Compiler::new();
        let mut rand = XoroshiroRandomSource::new(0);

        let func = DensityFunction::Add {
            left: DensityFunctionArgument::Constant(1.0),
            right: DensityFunctionArgument::Constant(2.0),
        };

        let parent = compiler.alloc_buffer(BufferType::Out);

        // should panic because func should've been folded into a constant prior to compilation
        compile(&mut compiler, &mut rand.fork_positional(), &func, parent);
    }

    #[test]
    fn test_compile_y_clamped_gradient() {
        let mut compiler = Compiler::new();
        let mut rand = XoroshiroRandomSource::new(0);

        let from_y = -16;
        let to_y = 16;
        let from_value = -1.0;
        let to_value = 1.0;

        let func = DensityFunction::YClampedGradient {
            from_y,
            to_y,
            from_value,
            to_value,
        };

        let parent = compiler.alloc_buffer(BufferType::Out);
        let out = compile(&mut compiler, &mut rand.fork_positional(), &func, parent);

        assert_eq!(parent, out);
        assert_eq!(compiler.ops.len(), 1);
        assert!(compiler.ops.last().is_some());

        let Some(Operation::YClampedGradient {
            destination,
            y_range,
            value_range,
        }) = compiler.ops.last()
        else {
            panic!("last operation was not a y-clamped gradient")
        };

        assert_eq!(*destination, out);
        assert_eq!(*y_range.start(), from_y as i16);
        assert_eq!(*y_range.end(), to_y as i16);
        assert_eq!(*value_range.start(), from_value as f32);
        assert_eq!(*value_range.end(), to_value as f32);
    }

    #[test]
    fn test_compile_add() {
        let mut compiler = Compiler::new();

        let func = DensityFunction::Add {
            left: DensityFunctionArgument::Function(Box::new(DensityFunction::Shift { noise: "minecraft:aquifer_barrier".to_string() })),
            right: DensityFunctionArgument::Function(Box::new(DensityFunction::Shift { noise: "aquifer_barrier".to_string() })),
        };

        let parent = BufferId { ty: BufferType::Out, id: 0 };
        let out = compile(&mut compiler, &func, parent);

        assert_eq!(parent, out);
        assert_eq!(compiler.ops.len(), 3);
        assert!(compiler.ops.last().is_some());
        assert!(matches!(compiler.ops.last().unwrap(), Operation::AddBuffer { .. }));
    }

    #[test]
    #[should_panic = "functions should be folded prior to being compiled"]
    fn test_compile_add_no_fold() {
        let mut compiler = Compiler::new();

        let func = DensityFunction::Add {
            left: DensityFunctionArgument::Constant(1.0),
            right: DensityFunctionArgument::Constant(2.0),
        };

        let parent = compiler.alloc_buffer(BufferType::Out);

        // should panic because func should've been folded into a constant prior to compilation
        compile(&mut compiler, &func, parent);
    }

    #[test]
    fn test_compile_y_clamped_gradient() {
        let mut compiler = Compiler::new();

        let from_y = -16;
        let to_y = 16;
        let from_value = -1.0;
        let to_value = 1.0;

        let func = DensityFunction::YClampedGradient {
            from_y,
            to_y,
            from_value,
            to_value,
        };

        let parent = compiler.alloc_buffer(BufferType::Out);
        let out = compile(&mut compiler, &func, parent);

        assert_eq!(parent, out);
        assert_eq!(compiler.ops.len(), 1);
        assert!(compiler.ops.last().is_some());

        let Some(Operation::YClampedGradient { destination, y_range, value_range }) = compiler.ops.last() else {
            panic!("last operation was not a y-clamped gradient")
        };

        assert_eq!(*destination, out);
        assert_eq!(*y_range.start(), from_y as i16);
        assert_eq!(*y_range.end(), to_y as i16);
        assert_eq!(*value_range.start(), from_value as f32);
        assert_eq!(*value_range.end(), to_value as f32);
    }
}
