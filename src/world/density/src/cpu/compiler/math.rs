use crate::DensityFunctionArgument;
use crate::cpu::compiler::visitor::{
    AddBufferVisitor, AddConstantVisitor, AddNoiseVisitor, FillConstantVisitor, FillNoiseVisitor,
    MulBufferVisitor, MulConstantVisitor, MulNoiseVisitor,
};
use crate::cpu::compiler::{AnyBufferId, Compiler, ReturnValue, compile_arg};
use temper_core::random::{PositionalRandom, RandomSource};

pub fn compile_add<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    left: &DensityFunctionArgument,
    right: &DensityFunctionArgument,
    parent_buffer: AnyBufferId,
) -> AnyBufferId {
    let left_source = compile_arg(compiler, rand, left, parent_buffer);

    let right_buffer = if let ReturnValue::Buffer(left) = &left_source
        && *left == parent_buffer
    {
        compiler.alloc_buffer(parent_buffer)
    } else {
        parent_buffer
    };
    let right_source = compile_arg(compiler, rand, right, right_buffer);

    if let ReturnValue::Buffer(left) = &left_source
        && *left != parent_buffer
    {
        compiler.free_buffer(*left);
    }

    if let ReturnValue::Buffer(right) = &right_source
        && *right != parent_buffer
    {
        compiler.free_buffer(*right);
    }

    match (left_source, right_source) {
        (ReturnValue::Constant(left), ReturnValue::Constant(right)) => {
            compiler.push_visitor(FillConstantVisitor::new(parent_buffer, left + right))
        }
        (ReturnValue::Constant(val), ReturnValue::Noise(noise))
        | (ReturnValue::Noise(noise), ReturnValue::Constant(val)) => {
            let noise = compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise));

            compiler.push_visitor(AddConstantVisitor::new(noise, val))
        }
        (ReturnValue::Noise(noise_a), ReturnValue::Noise(noise_b)) => {
            let noise_a = compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise_a));

            compiler.push_visitor(AddNoiseVisitor::new(noise_a, noise_b))
        }
        (ReturnValue::Noise(noise), ReturnValue::Buffer(buffer))
        | (ReturnValue::Buffer(buffer), ReturnValue::Noise(noise)) => {
            compiler.push_visitor(AddNoiseVisitor::new(buffer, noise))
        }
        (ReturnValue::Constant(val), ReturnValue::Buffer(buffer))
        | (ReturnValue::Buffer(buffer), ReturnValue::Constant(val)) => {
            compiler.push_visitor(AddConstantVisitor::new(buffer, val))
        }
        (ReturnValue::Buffer(left), ReturnValue::Buffer(right)) => {
            compiler.push_visitor(AddBufferVisitor::new(left, right))
        }
    }
}

pub fn compile_mul<R: RandomSource, P: PositionalRandom<R>>(
    compiler: &mut Compiler,
    rand: &mut P,
    left: &DensityFunctionArgument,
    right: &DensityFunctionArgument,
    parent_buffer: AnyBufferId,
) -> AnyBufferId {
    let left_source = compile_arg(compiler, rand, left, parent_buffer);

    let right_buffer = if let ReturnValue::Buffer(left) = &left_source
        && *left == parent_buffer
    {
        compiler.alloc_buffer(parent_buffer)
    } else {
        parent_buffer
    };
    let right_source = compile_arg(compiler, rand, right, right_buffer);

    if let ReturnValue::Buffer(left) = &left_source
        && *left != parent_buffer
    {
        compiler.free_buffer(*left);
    }

    if let ReturnValue::Buffer(right) = &right_source
        && *right != parent_buffer
    {
        compiler.free_buffer(*right);
    }

    match (left_source, right_source) {
        (ReturnValue::Constant(left), ReturnValue::Constant(right)) => {
            compiler.push_visitor(FillConstantVisitor::new(parent_buffer, left * right))
        }
        (ReturnValue::Constant(val), ReturnValue::Noise(noise))
        | (ReturnValue::Noise(noise), ReturnValue::Constant(val)) => {
            let noise = compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise));

            compiler.push_visitor(MulConstantVisitor::new(noise, val))
        }
        (ReturnValue::Noise(noise_a), ReturnValue::Noise(noise_b)) => {
            let noise_a = compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise_a));

            compiler.push_visitor(MulNoiseVisitor::new(noise_a, noise_b))
        }
        (ReturnValue::Noise(noise), ReturnValue::Buffer(buffer))
        | (ReturnValue::Buffer(buffer), ReturnValue::Noise(noise)) => {
            compiler.push_visitor(MulNoiseVisitor::new(buffer, noise))
        }
        (ReturnValue::Constant(val), ReturnValue::Buffer(buffer))
        | (ReturnValue::Buffer(buffer), ReturnValue::Constant(val)) => {
            compiler.push_visitor(MulConstantVisitor::new(buffer, val))
        }
        (ReturnValue::Buffer(left), ReturnValue::Buffer(right)) => {
            compiler.push_visitor(MulBufferVisitor::new(left, right))
        }
    }
}
