use temper_core::random::{PositionalRandom, RandomSource};
use crate::cpu::buffer::BufferId;
use crate::cpu::compiler::{compile, Compiler};
use crate::cpu::operation::{Operation, ValueSource};
use crate::DensityFunctionArgument;

pub fn compile_add<R: RandomSource, P: PositionalRandom<R>>(compiler: &mut Compiler, rand: &mut P, parent_buffer: BufferId, left: &DensityFunctionArgument, right: &DensityFunctionArgument) -> BufferId {
    let (left_value, left_buffer) = match ValueSource::try_from(left, rand) {
        Some(v) =>  (v, None),
        None => {
            let DensityFunctionArgument::Function(func) = left else {
                unreachable!()
            };

            let out = compile(compiler, rand, func, parent_buffer);

            (
                ValueSource::Buffer(out),
                Some(out)
            )
        }
    };

    let (right_value, right_buffer) = match ValueSource::try_from(right, rand) {
        Some(v) => (v, None),
        None => {
            let DensityFunctionArgument::Function(func) = right else {
                unreachable!()
            };

            let buffer = if let Some(left_buffer) = left_buffer && left_buffer == parent_buffer {
                compiler.alloc_buffer(parent_buffer.ty)
            } else {
                parent_buffer
            };

            let out = compile(compiler, rand, func, buffer);

            if out != buffer && buffer != parent_buffer {
                compiler.free_buffer(buffer);
            }

            (
                ValueSource::Buffer(out),
                Some(out)
            )
        }
    };

    if matches!(right_value, ValueSource::Constant(..)) && matches!(left_value, ValueSource::Constant(..)) {
        panic!("functions should be folded prior to being compiled")
    }

    match (left_buffer, right_buffer) {
        (Some(left_buffer), Some(right_buffer)) => {
            let out = left_buffer.max(right_buffer);

            compiler.push_op(Operation::AddBuffer {
                destination: out,
                source: if out == left_buffer {
                    right_value
                } else {
                    left_value
                }
            });

            out
        },
        (Some(buffer), None) => {
            compiler.push_op(Operation::AddBuffer {
                destination: buffer,
                source: right_value,
            });

            buffer
        },
        (None, Some(buffer)) => {
            compiler.push_op(Operation::AddBuffer {
                destination: buffer,
                source: left_value,
            });

            buffer
        },
        (None, None) => {
            compiler.push_op(Operation::ClearBuffer {
                destination: parent_buffer,
                source: left_value,
            });

            compiler.push_op(Operation::AddBuffer {
                destination: parent_buffer,
                source: right_value,
            });

            parent_buffer
        },
    }
}