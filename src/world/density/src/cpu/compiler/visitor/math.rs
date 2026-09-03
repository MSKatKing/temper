use crate::cpu::compiler::AnyBufferId;
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::{
    BufferAdd, BufferMul, ConstantAdd, ConstantMul, NoiseAdd, NoiseMul, YClampedGradient,
};
use crate::{impl_commutative_visitor, impl_direct_visitor, impl_visitor_base};
use std::ops::RangeInclusive;

impl_visitor_base!(AddBufferVisitor, other: AnyBufferId);
impl_visitor_base!(AddConstantVisitor, other: f32);
impl_visitor_base!(AddNoiseVisitor, other: NoiseAccessor);

impl_visitor_base!(MulBufferVisitor, other: AnyBufferId);
impl_visitor_base!(MulConstantVisitor, other: f32);
impl_visitor_base!(MulNoiseVisitor, other: NoiseAccessor);

// impl_visitor_base!(MinBufferVisitor, other: AnyBufferId);
// impl_visitor_base!(MinConstantVisitor, other: f32);
// impl_visitor_base!(MinNoiseVisitor, other: NoiseAccessor);
//
// impl_visitor_base!(MaxBufferVisitor, other: AnyBufferId);
// impl_visitor_base!(MaxConstantVisitor, other: f32);
// impl_visitor_base!(MaxNoiseVisitor, other: NoiseAccessor);
//
// impl_visitor_base!(SubBufferVisitor, other: AnyBufferId);
// impl_visitor_base!(SubConstantVisitor, other: f32);
// impl_visitor_base!(SubNoiseVisitor, other: NoiseAccessor);
//
// impl_visitor_base!(DivBufferVisitor, other: AnyBufferId);
// impl_visitor_base!(DivConstantVisitor, other: f32);
// impl_visitor_base!(DivNoiseVisitor, other: NoiseAccessor);

impl_visitor_base!(YClampedGradientVisitor, y_range: RangeInclusive<i16>, value_range: RangeInclusive<f32>);

impl_commutative_visitor!(AddBufferVisitor, BufferAdd, other);
impl_direct_visitor!(AddConstantVisitor, ConstantAdd, dst, src: other);
impl_direct_visitor!(AddNoiseVisitor, NoiseAdd, dst, src: other);

impl_commutative_visitor!(MulBufferVisitor, BufferMul, other);
impl_direct_visitor!(MulConstantVisitor, ConstantMul, dst, src: other);
impl_direct_visitor!(MulNoiseVisitor, NoiseMul, dst, src: other);

// impl_commutative_visitor!(MinBufferVisitor, BufferMin, other);
// impl_commutative_visitor!(MaxBufferVisitor, BufferMax, other);

impl_direct_visitor!(YClampedGradientVisitor, YClampedGradient, dst, y_range: y_range, value_range: value_range);
