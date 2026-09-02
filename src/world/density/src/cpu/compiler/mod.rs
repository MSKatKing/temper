mod math;

use crate::cpu::buffer::{BufferId, BufferType};
use crate::cpu::compiler::math::{compile_add, compile_max, compile_min, compile_mul, compile_sub};
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use crate::cpu::operation::{Operation, ValueSource};
use crate::{DensityFunction, DensityFunctionArgument};
use std::collections::{HashMap, VecDeque};
use temper_core::random::{PositionalRandom, RandomSource};
use temper_data::noise::NoiseParameter;

pub struct Compiler {
    buffers: HashMap<BufferType, VecDeque<Option<usize>>>,
    ops: Vec<Operation>,
}

#[derive(Clone, Debug)]
pub struct CompiledDensityFunction {
    pub(crate) ops: Vec<Operation>,
    pub(crate) buffers: HashMap<BufferType, VecDeque<Option<usize>>>,
}

impl Compiler {
    pub fn compile<R: RandomSource>(
        rand: &mut R,
        func: DensityFunctionArgument,
    ) -> CompiledDensityFunction {
        let mut this = Compiler::new();

        let out = this.alloc_buffer(BufferType::Out);
        let actual = match func {
            DensityFunctionArgument::Function(func) => {
                compile(&mut this, &mut rand.fork_positional(), func.as_ref(), out)
            }
            DensityFunctionArgument::Constant(val) => {
                this.push_op(Operation::ClearBuffer {
                    destination: out,
                    source: ValueSource::Constant(val as f32),
                });

                out
            }
            DensityFunctionArgument::External(_) => {
                panic!("should be linked before being compiled")
            }
        };

        if actual != out {
            this.push_op(Operation::ClearBuffer {
                destination: out,
                source: ValueSource::Buffer(actual),
            });
        }

        CompiledDensityFunction {
            ops: this.ops,
            buffers: this.buffers,
        }
    }

    fn new() -> Compiler {
        Compiler {
            buffers: HashMap::with_capacity(5),
            ops: Vec::new(),
        }
    }

    fn alloc_buffer(&mut self, buffer_type: BufferType) -> BufferId {
        let buffers = self.buffers.entry(buffer_type).or_default();

        for (i, val) in buffers.iter_mut().enumerate() {
            if val.is_some() {
                continue;
            }

            *val = Some(i);
            return BufferId {
                ty: buffer_type,
                id: i as u8,
            };
        }

        let i = buffers.len();
        buffers.push_back(Some(i));
        BufferId {
            ty: buffer_type,
            id: i as u8,
        }
    }

    fn free_buffer(&mut self, buffer: BufferId) {
        let v = self
            .buffers
            .entry(buffer.ty)
            .or_default()
            .iter_mut()
            .find(|val| val.is_some() && val.unwrap() == buffer.id as usize);

        if let Some(i) = v {
            *i = None;
        } else {
            panic!("{:?} never existed", buffer)
        }
    }

    fn push_op(&mut self, op: Operation) {
        self.ops.push(op);
    }
}

fn compile<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    func: &DensityFunction,
    parent_buffer: BufferId,
) -> BufferId {
    macro_rules! marker_compile {
        ($buffer_type:ident, $input:expr) => {{
            let buffer = if parent_buffer.ty == BufferType::$buffer_type {
                parent_buffer
            } else {
                compiler.alloc_buffer(BufferType::$buffer_type)
            };

            match $input {
                DensityFunctionArgument::Constant(val) => {
                    compiler.push_op(Operation::ClearBuffer {
                        destination: buffer,
                        source: ValueSource::Constant(*val as f32),
                    });

                    buffer
                }
                DensityFunctionArgument::Function(func) => {
                    let actual = compile(compiler, rand, func, buffer);

                    if actual != buffer {
                        if actual.ty > buffer.ty {
                            panic!("internal function overrode buffer size");
                        }

                        if buffer != parent_buffer {
                            compiler.free_buffer(buffer);
                        }
                    }

                    actual
                }
                DensityFunctionArgument::External(_) => {
                    panic!("functions should be linked before being compiled")
                }
            }
        }};
    }

    match func {
        DensityFunction::Add { left, right } => {
            compile_add(compiler, rand, parent_buffer, left, right)
        }
        DensityFunction::Sub { left, right } => {
            compile_sub(compiler, rand, parent_buffer, left, right)
        }
        DensityFunction::Mul { left, right } => {
            compile_mul(compiler, rand, parent_buffer, left, right)
        }
        DensityFunction::Min { left, right } => {
            compile_min(compiler, rand, parent_buffer, left, right)
        }
        DensityFunction::Max { left, right } => {
            compile_max(compiler, rand, parent_buffer, left, right)
        }
        DensityFunction::Shift { noise } => {
            let noise_split = noise.split(":").collect::<Vec<_>>();
            let noise = if noise_split.len() == 2 {
                noise.clone()
            } else {
                format!("minecraft:{}", noise_split[0])
            };

            compiler.push_op(Operation::ClearBuffer {
                destination: parent_buffer,
                source: ValueSource::Noise(NoiseAccessor::new(
                    NoiseParameter::get_by_name(noise.as_str())
                        .unwrap_or_else(|| panic!("'{}' is not a valid noise parameter", noise)),
                    rand,
                    noise.as_str(),
                    NoiseAccessType::Shift,
                )),
            });

            parent_buffer
        }
        DensityFunction::Noise {
            noise,
            xz_scale,
            y_scale,
        } => {
            let noise_split = noise.split(":").collect::<Vec<_>>();
            let noise = if noise_split.len() == 2 {
                noise.clone()
            } else {
                format!("minecraft:{}", noise_split[0])
            };

            compiler.push_op(Operation::ClearBuffer {
                destination: parent_buffer,
                source: ValueSource::Noise(NoiseAccessor::new(
                    NoiseParameter::get_by_name(noise.as_str())
                        .unwrap_or_else(|| panic!("'{}' is not a valid noise parameter", noise)),
                    rand,
                    noise.as_str(),
                    NoiseAccessType::Basic {
                        xz_scale: *xz_scale as f32,
                        y_scale: *y_scale as f32,
                    },
                )),
            });

            parent_buffer
        }
        DensityFunction::Interpolated { input } => marker_compile!(Interpolated, input),
        DensityFunction::Cache2d { input } => marker_compile!(Flat, input),
        DensityFunction::FlatCache { input } => marker_compile!(FlatCell, input),
        DensityFunction::YClampedGradient {
            from_y,
            to_y,
            from_value,
            to_value,
        } => {
            compiler.push_op(Operation::YClampedGradient {
                destination: parent_buffer,
                y_range: (*from_y as i16)..=(*to_y as i16),
                value_range: (*from_value as f32)..=(*to_value as f32),
            });

            parent_buffer
        }
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
