mod math;

use crate::cpu::buffer::{BufferId, BufferType};
use crate::cpu::compiler::math::compile_add;
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use crate::cpu::operation::{Operation, Projection, ValueSource};
use crate::{DensityFunction, DensityFunctionArgument};
use std::collections::{HashMap, VecDeque};
use temper_data::noise::NoiseParameter;

pub struct Compiler {
    buffers: HashMap<BufferType, VecDeque<usize>>,
    next_buffer: HashMap<BufferType, usize>,
    ops: Vec<Operation>,
}

#[derive(Clone)]
pub struct CompiledDensityFunction {
    pub(crate) ops: Vec<Operation>,
    pub(crate) buffers: HashMap<BufferType, VecDeque<usize>>,
}

impl Compiler {
    pub fn compile(func: &DensityFunction) -> CompiledDensityFunction {
        let mut this = Compiler::new();
        
        let out = this.alloc_buffer(BufferType::Out);
        let actual = compile(&mut this, func, out);
        
        if actual != out {
            this.push_op(Operation::ClearBuffer {
                destination: out,
                source: ValueSource::Buffer(out, Projection::None)
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
            next_buffer: HashMap::with_capacity(5),
            ops: Vec::new(),
        }
    }

    fn alloc_buffer(&mut self, buffer_type: BufferType) -> BufferId {
        let idx = if let Some(buf) = {
            let buffers = self.buffers.entry(buffer_type).or_insert_with(VecDeque::new);
            buffers.pop_front()
        } {
            buf
        } else {
            let next_buffer = self.next_buffer.entry(buffer_type).or_insert(0);
            *next_buffer += 1;
            *next_buffer - 1
        };

        BufferId {
            ty: buffer_type,
            id: idx as _,
        }
    }

    fn free_buffer(&mut self, buffer: BufferId) {
        self.buffers.entry(buffer.ty).or_insert_with(VecDeque::new).push_back(buffer.id as _)
    }

    fn push_op(&mut self, op: Operation) {
        self.ops.push(op);
    }
}

fn buffer_size_of(func: &DensityFunction, parent_size: BufferType) -> BufferType {
    match func {
        DensityFunction::CacheOnce { .. } | DensityFunction::CacheAllInCell { .. } => BufferType::Full,
        DensityFunction::Cache2d { .. } => BufferType::Flat,
        DensityFunction::FlatCache { .. } => BufferType::FlatCell,
        DensityFunction::Interpolated { .. } => BufferType::Interpolated,
        _ => parent_size
    }
}

fn compile(compiler: &mut Compiler, func: &DensityFunction, parent_buffer: BufferId) -> BufferId {
    match func {
        DensityFunction::Add { left, right } => compile_add(compiler, parent_buffer, left, right),
        DensityFunction::Shift { noise } => {
            let noise_split = noise.split(":").collect::<Vec<_>>();
            let noise = if noise_split.len() == 2 {
                noise.clone()
            } else {
                format!("minecraft:{}", noise_split[0])
            };

            compiler.push_op(Operation::ClearBuffer {
                destination: parent_buffer,
                source: ValueSource::Noise(
                    NoiseAccessor::new(
                        NoiseParameter::get_by_name(noise.as_str()).unwrap_or_else(|| panic!("'{}' is not a valid noise parameter", noise)),
                        NoiseAccessType::Shift,
                    ),
                ),
            });

            parent_buffer
        },
        DensityFunction::YClampedGradient { from_y, to_y, from_value, to_value } => {
            compiler.push_op(Operation::YClampedGradient {
                destination: parent_buffer,
                y_range: (*from_y as i16)..=(*to_y as i16),
                value_range: (*from_value as f32)..=(*to_value as f32),
            });

            parent_buffer
        }
        _ => todo!(),
    }
}

impl TryFrom<&DensityFunctionArgument> for ValueSource {
    type Error = ();

    fn try_from(value: &DensityFunctionArgument) -> Result<Self, Self::Error> {
        match value {
            DensityFunctionArgument::Function(func) => match func.as_ref() {
                DensityFunction::Constant { value } => Ok(ValueSource::Constant(*value as _)),
                DensityFunction::Noise { noise, xz_scale, y_scale } => {
                    let param = NoiseParameter::get_by_name(noise.as_str()).ok_or(())?;

                    Ok(
                        ValueSource::Noise(
                            NoiseAccessor::new(
                                param,
                                NoiseAccessType::Basic {
                                    xz_scale: * xz_scale as f32,
                                    y_scale: *y_scale as f32
                                }
                            )
                        )
                    )
                },
                DensityFunction::Shift { noise } => {
                    let param = NoiseParameter::get_by_name(noise.as_str()).ok_or(())?;

                    Ok(
                        ValueSource::Noise(
                            NoiseAccessor::new(
                                param,
                                NoiseAccessType::Shift,
                            ),
                        ),
                    )
                }
                _ => Err(())
            }
            DensityFunctionArgument::Constant(value) => Ok(ValueSource::Constant(*value as _)),
            DensityFunctionArgument::External(_) => panic!("functions should be linked prior to being compiled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DensityFunctionArgument;

    #[test]
    fn test_compile_shift() {
        let mut compiler = Compiler::new();

        let func = DensityFunction::Shift {
            noise: "minecraft:aquifer_barrier".to_string(),
        };

        let parent = BufferId { ty: BufferType::Out, id: 0 };
        let out = compile(&mut compiler, &func, parent);

        assert_eq!(parent, out);
        assert_eq!(compiler.ops.len(), 1);
        assert!(compiler.ops.last().is_some());

        let Some(Operation::ClearBuffer { destination, source }) = compiler.ops.last() else {
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
