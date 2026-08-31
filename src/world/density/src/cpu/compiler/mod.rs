use crate::cpu::buffer::{BufferId, BufferType};
use crate::DensityFunction;
use std::collections::{HashMap, VecDeque};
use temper_data::noise::NoiseParameter;
use crate::cpu::operation::{NoiseAccessType, Operation, Projection, ValueSource};

pub struct Compiler {
    buffers: HashMap<BufferType, VecDeque<usize>>,
    next_buffer: HashMap<BufferType, usize>,
    ops: Vec<Operation>,
}

impl Compiler {
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
        DensityFunction::Add { left, right } => {
            match (left.constant(), right.constant()) {
                (Some(_), Some(_)) => panic!("functions should be folded prior to being compiled"),
                (Some(val), None) | (None, Some(val)) => {
                    let func = left.function().or(right.function()).unwrap();
                    compile(compiler, func, parent_buffer);

                    compiler.push_op(Operation::AddBuffer {
                        destination: parent_buffer,
                        source: ValueSource::Constant(val as _),
                    });

                    parent_buffer
                },
                (None, None) => {
                    let left = left.function().unwrap();
                    let right = right.function().unwrap();

                    compile(compiler, left, parent_buffer);

                    let buffer = compiler.alloc_buffer(parent_buffer.ty);
                    compile(compiler, right, buffer);
                    compiler.free_buffer(buffer);

                    compiler.push_op(Operation::AddBuffer {
                        destination: parent_buffer,
                        source: ValueSource::Buffer(buffer, Projection::None)
                    });

                    parent_buffer
                }
            }
        },
        DensityFunction::Shift { noise } => {
            let noise_split = noise.split(":").collect::<Vec<_>>();
            let noise = if noise_split.len() == 2 {
                noise.clone()
            } else {
                format!("minecraft:{}", noise_split[0])
            };

            compiler.push_op(Operation::FillNoiseBuffer {
                destination: parent_buffer,
                noise: NoiseParameter::get_by_name(noise.as_str()).unwrap_or_else(|| panic!("'{}' is not a valid noise parameter", noise)),
                access_type: NoiseAccessType::Shift,
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

#[cfg(test)]
mod tests {
    use crate::DensityFunctionArgument;
    use super::*;

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

        let Some(Operation::FillNoiseBuffer { destination, noise, access_type }) = compiler.ops.last() else {
            panic!("last operation was not a fill noise buffer operation")
        };

        assert_eq!(Some(*noise), NoiseParameter::get_by_name("minecraft:aquifer_barrier"));
        assert_eq!(*destination, parent);
        assert_eq!(*access_type, NoiseAccessType::Shift);
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
