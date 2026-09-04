use crate::DensityFunctionArgument;
use crate::cpu::compiler::visitor::{
    AddBufferVisitor, AddConstantVisitor, AddNoiseVisitor, DivBufferVisitor, DivConstantVisitor,
    DivNoiseVisitor, FillConstantVisitor, FillNoiseVisitor, MaxBufferVisitor, MaxConstantVisitor,
    MaxNoiseVisitor, MinBufferVisitor, MinConstantVisitor, MinNoiseVisitor, MulBufferVisitor,
    MulConstantVisitor, MulNoiseVisitor, SubBufferVisitor, SubConstantVisitor, SubNoiseVisitor,
};
use crate::cpu::compiler::{AnyBufferId, Compiler, ReturnValue, compile_arg};
use temper_core::random::{PositionalRandom, RandomSource};

macro_rules! compile_commutative {
    ($name:ident, $buffer:ty, $constant:ty, $noise:ty) => {
        pub fn $name<R: RandomSource, P: PositionalRandom<R>>(
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

                    compiler.push_visitor(<$constant>::new(noise, val))
                }
                (ReturnValue::Noise(noise_a), ReturnValue::Noise(noise_b)) => {
                    let noise_a =
                        compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise_a));

                    compiler.push_visitor(<$noise>::new(noise_a, noise_b))
                }
                (ReturnValue::Noise(noise), ReturnValue::Buffer(buffer))
                | (ReturnValue::Buffer(buffer), ReturnValue::Noise(noise)) => {
                    compiler.push_visitor(<$noise>::new(buffer, noise))
                }
                (ReturnValue::Constant(val), ReturnValue::Buffer(buffer))
                | (ReturnValue::Buffer(buffer), ReturnValue::Constant(val)) => {
                    compiler.push_visitor(<$constant>::new(buffer, val))
                }
                (ReturnValue::Buffer(left), ReturnValue::Buffer(right)) => {
                    compiler.push_visitor(<$buffer>::new(left, right))
                }
            }
        }
    };
}

macro_rules! compile_non_commutative {
    ($name:ident, $buffer:ty, $constant:ty, $noise:ty) => {
        pub fn $name<R: RandomSource, P: PositionalRandom<R>>(
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
                    compiler.push_visitor(FillConstantVisitor::new(parent_buffer, left / right))
                }
                (ReturnValue::Constant(val), ReturnValue::Noise(noise)) => {
                    let val = compiler.push_visitor(FillConstantVisitor::new(parent_buffer, val));

                    compiler.push_visitor(<$noise>::new(val, noise))
                }
                (ReturnValue::Noise(noise), ReturnValue::Constant(val)) => {
                    let noise = compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise));

                    compiler.push_visitor(<$constant>::new(noise, val))
                }
                (ReturnValue::Noise(noise_a), ReturnValue::Noise(noise_b)) => {
                    let noise_a =
                        compiler.push_visitor(FillNoiseVisitor::new(parent_buffer, noise_a));

                    compiler.push_visitor(<$noise>::new(noise_a, noise_b))
                }
                (ReturnValue::Noise(noise), ReturnValue::Buffer(buffer)) => {
                    let noise = compiler.push_visitor(<$noise>::new(parent_buffer, noise));

                    compiler.push_visitor(<$buffer>::new(noise, buffer))
                }
                (ReturnValue::Buffer(buffer), ReturnValue::Noise(noise)) => {
                    compiler.push_visitor(<$noise>::new(buffer, noise))
                }
                (ReturnValue::Constant(val), ReturnValue::Buffer(buffer)) => {
                    let val = compiler.push_visitor(<$constant>::new(parent_buffer, val));

                    compiler.push_visitor(<$buffer>::new(val, buffer))
                }
                (ReturnValue::Buffer(buffer), ReturnValue::Constant(val)) => {
                    compiler.push_visitor(<$constant>::new(buffer, val))
                }
                (ReturnValue::Buffer(left), ReturnValue::Buffer(right)) => {
                    compiler.push_visitor(<$buffer>::new(left, right))
                }
            }
        }
    };
}

compile_commutative!(
    compile_add,
    AddBufferVisitor,
    AddConstantVisitor,
    AddNoiseVisitor
);
compile_commutative!(
    compile_mul,
    MulBufferVisitor,
    MulConstantVisitor,
    MulNoiseVisitor
);
compile_commutative!(
    compile_min,
    MinBufferVisitor,
    MinConstantVisitor,
    MinNoiseVisitor
);
compile_commutative!(
    compile_max,
    MaxBufferVisitor,
    MaxConstantVisitor,
    MaxNoiseVisitor
);
compile_non_commutative!(
    compile_sub,
    SubBufferVisitor,
    SubConstantVisitor,
    SubNoiseVisitor
);
compile_non_commutative!(
    compile_div,
    DivBufferVisitor,
    DivConstantVisitor,
    DivNoiseVisitor
);
