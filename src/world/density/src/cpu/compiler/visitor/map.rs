use crate::{impl_direct_visitor, impl_visitor_base};
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::{AbsBuffer, AbsNoise, ClampBuffer, ClampNoise, NegativeDecayBuffer, NegativeDecayNoise, PowBuffer, PowNoise, SqueezeBuffer, SqueezeNoise};

impl_visitor_base!(SqueezeBufferVisitor);
impl_visitor_base!(SqueezeNoiseVisitor, noise: NoiseAccessor);

impl_visitor_base!(AbsBufferVisitor);
impl_visitor_base!(AbsNoiseVisitor, noise: NoiseAccessor);

impl_visitor_base!(ClampBufferVisitor, min: f32, max: f32);
impl_visitor_base!(ClampNoiseVisitor, noise: NoiseAccessor, min: f32, max: f32);

impl_visitor_base!(PowBufferVisitor, amt: i32);
impl_visitor_base!(PowNoiseVisitor, noise: NoiseAccessor, amt: i32);

impl_visitor_base!(NegativeDecayBufferVisitor, amt: f32);
impl_visitor_base!(NegativeDecayNoiseVisitor, noise: NoiseAccessor, amt: f32);

impl_direct_visitor!(SqueezeBufferVisitor, SqueezeBuffer, dst,);
impl_direct_visitor!(SqueezeNoiseVisitor, SqueezeNoise, dst, noise: noise);

impl_direct_visitor!(AbsBufferVisitor, AbsBuffer, dst,);
impl_direct_visitor!(AbsNoiseVisitor, AbsNoise, dst, noise: noise);

impl_direct_visitor!(ClampBufferVisitor, ClampBuffer, dst, min: min, max: max);
impl_direct_visitor!(ClampNoiseVisitor, ClampNoise, dst, noise: noise, min: min, max: max);

impl_direct_visitor!(PowBufferVisitor, PowBuffer, dst, amt: amt);
impl_direct_visitor!(PowNoiseVisitor, PowNoise, dst, noise: noise, amt: amt);

impl_direct_visitor!(NegativeDecayBufferVisitor, NegativeDecayBuffer, dst, amt: amt);
impl_direct_visitor!(NegativeDecayNoiseVisitor, NegativeDecayNoise, dst, noise: noise, amt: amt);
