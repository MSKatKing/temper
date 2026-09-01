use temper_core::random::{PositionalRandom, RandomSource};
use crate::cpu::buffer::BufferId;
use crate::cpu::compiler::{compile, Compiler};
use crate::cpu::operation::{Operation, ValueSource};
use crate::DensityFunctionArgument;

fn compile_math_inner<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    parent_buffer: BufferId,
    left: &DensityFunctionArgument,
    right: &DensityFunctionArgument,
) -> ((ValueSource, Option<BufferId>), (ValueSource, Option<BufferId>)) {
    let (left_value, left_buffer) = ValueSource::try_from(left, rand)
        .map(|v| (v, None))
        .unwrap_or_else(|| {
            let func = left.function().unwrap();
            let out = compile(compiler, rand, func, parent_buffer);

            (
                ValueSource::Buffer(out),
                Some(out),
            )
        });

    let (right_value, right_buffer) = ValueSource::try_from(right, rand)
        .map(|v| (v, None))
        .unwrap_or_else(|| {
            let func = right.function().unwrap();

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
                Some(out),
            )
        });

    (
        (left_value, left_buffer),
        (right_value, right_buffer),
    )
}

macro_rules! implement_commutative {
    ($name:ident, $op:ident) => {
        pub fn $name<R: RandomSource, P: PositionalRandom<R>>(
            compiler: &mut Compiler,
            rand: &mut P,
            parent_buffer: BufferId,
            left: &DensityFunctionArgument,
            right: &DensityFunctionArgument,
        ) -> BufferId {
            let (
                (left_value, left_buffer),
                (right_value, right_buffer),
            ) = compile_math_inner(compiler, rand, parent_buffer, left, right);
            
            match (left_buffer, right_buffer) {
                (Some(left_buffer), Some(right_buffer)) => {
                    let out = left_buffer.max(right_buffer);
                    
                    compiler.push_op(Operation::$op {
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
                    compiler.push_op(Operation::$op {
                        destination: buffer,
                        source: right_value,
                    });
                    
                    buffer
                },
                (None, Some(buffer)) => {
                    compiler.push_op(Operation::$op {
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
                    
                    compiler.push_op(Operation::$op {
                        destination: parent_buffer,
                        source: right_value,
                    });
                    
                    parent_buffer
                }
            }
        }
    };
}

implement_commutative!(compile_add, AddBuffer);
implement_commutative!(compile_mul, MulBuffer);
implement_commutative!(compile_min, MinBuffer);
implement_commutative!(compile_max, MaxBuffer);

pub fn compile_sub<R: RandomSource, P: PositionalRandom<R>>(compiler: &mut Compiler, rand: &mut P, parent_buffer: BufferId, left: &DensityFunctionArgument, right: &DensityFunctionArgument) -> BufferId {
    let (
        (left_value, left_buffer),
        (right_value, right_buffer),
    ) = compile_math_inner(compiler, rand, parent_buffer, left, right);

    match (left_buffer, right_buffer) {
        (Some(left_buffer), Some(right_buffer)) => {
            assert!(left_buffer.ty >= right_buffer.ty, "lhs size must be larger than or equal to rhs size");

            compiler.push_op(Operation::SubBuffer {
                destination: left_buffer,
                source: right_value,
            });

            left_buffer
        },
        (Some(buffer), None) => {
            compiler.push_op(Operation::SubBuffer {
                destination: buffer,
                source: right_value,
            });

            buffer
        },
        (None, Some(buffer)) => {
            let left = if buffer == parent_buffer {
                compiler.alloc_buffer(parent_buffer.ty)
            } else {
                parent_buffer
            };

            compiler.push_op(Operation::ClearBuffer {
                destination: left,
                source: left_value,
            });

            compiler.push_op(Operation::SubBuffer {
                destination: left,
                source: right_value,
            });

            compiler.free_buffer(buffer);

            left
        },
        (None, None) => {
            compiler.push_op(Operation::ClearBuffer {
                destination: parent_buffer,
                source: left_value,
            });

            compiler.push_op(Operation::SubBuffer {
                destination: parent_buffer,
                source: right_value,
            });

            parent_buffer
        },
    }
}
