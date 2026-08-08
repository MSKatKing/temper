use crate::cpu::buffer::{BufferId, BufferType};
use crate::DensityFunction;
use std::collections::{HashMap, VecDeque};
use crate::cpu::operation::{Operation, Projection, ValueSource};

pub struct Compiler {
    buffers: HashMap<BufferType, VecDeque<usize>>,
    next_buffer: HashMap<BufferType, usize>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            buffers: HashMap::with_capacity(5),
            next_buffer: HashMap::with_capacity(5),
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
        todo!()
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
                }
            }
        }
    }
}
