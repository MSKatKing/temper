use crate::cpu::buffer::BufferId;
use crate::cpu::compiler::AnyBufferId;
use crate::cpu::compiler::visitor::{BufferOperationResult, BufferOperationVisitor, VisitorBufferType};
use crate::cpu::runtime::{Spline, SplineOrValue};
use crate::impl_visitor_base;

pub enum ValueOrBuffer {
    Value(f32),
    Buffer(AnyBufferId)
}

pub struct SplinePoint {
    pub location: f32,
    pub derivative: f32,
    pub value: ValueOrBuffer,
}

impl_visitor_base!(SplineVisitor, points: Vec<SplinePoint>);

impl BufferOperationVisitor for SplineVisitor {
    fn visit_any<T: VisitorBufferType + 'static>(self, id: BufferId<T>) -> Option<BufferOperationResult> {
        Some(BufferOperationResult::new(
            Spline {
                dst: id,
                points: convert(self.points),
            },
            id,
        ))
    }
}

fn convert<T: VisitorBufferType>(points: Vec<SplinePoint>) -> Vec<crate::cpu::runtime::SplinePoint<T>> {
    let mut out = Vec::with_capacity(points.len());
    for point in points {
        out.push(crate::cpu::runtime::SplinePoint {
            location: point.location,
            derivative: point.derivative,
            value: match point.value {
                ValueOrBuffer::Value(v) => SplineOrValue::Value(v),
                ValueOrBuffer::Buffer(b) => SplineOrValue::Spline(T::try_downcast_to(b).unwrap()),
            }
        })
    }
    out
}
