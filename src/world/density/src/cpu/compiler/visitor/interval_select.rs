use crate::cpu::buffer::{BufferId, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::AnyBufferId;
use crate::cpu::compiler::visitor::{BufferOperationResult, BufferOperationVisitor, VisitorBufferType};
use crate::cpu::runtime::IntervalSelect;
use crate::impl_visitor_base;

impl_visitor_base!(IntervalSelectVisitor, thresholds: Vec<f32>, functions: Vec<AnyBufferId>);

impl BufferOperationVisitor for IntervalSelectVisitor {
    fn visit_any<T: VisitorBufferType + 'static>(self, id: BufferId<T>) -> Option<BufferOperationResult> {
        None
    }

    fn visit_full(self, dst: BufferId<Full>) -> BufferOperationResult {
        let first = &self.functions[0];
        match first {
            AnyBufferId::Full(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<Full>(self.functions),
                },
                dst,
            ),
            AnyBufferId::Interpolated(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<Interpolated>(self.functions),
                },
                dst,
            ),
            AnyBufferId::Flat(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<Flat>(self.functions),
                },
                dst,
            ),
            AnyBufferId::FlatCell(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<FlatCell>(self.functions),
                },
                dst,
            ),
        }
    }

    fn visit_interpolated(self, dst: BufferId<Interpolated>) -> BufferOperationResult {
        let first = &self.functions[0];
        match first {
            AnyBufferId::Full(_) => panic!("cannot write to lower buffer"),
            AnyBufferId::Interpolated(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<Interpolated>(self.functions),
                },
                dst,
            ),
            AnyBufferId::Flat(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<Flat>(self.functions),
                },
                dst,
            ),
            AnyBufferId::FlatCell(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<FlatCell>(self.functions),
                },
                dst,
            ),
        }
    }

    fn visit_flat(self, dst: BufferId<Flat>) -> BufferOperationResult {
        let first = &self.functions[0];
        match first {
            AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) => panic!("cannot write to lower buffer"),
            AnyBufferId::Flat(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<Flat>(self.functions),
                },
                dst,
            ),
            AnyBufferId::FlatCell(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<FlatCell>(self.functions),
                },
                dst,
            ),
        }
    }

    fn visit_flat_cell(self, dst: BufferId<FlatCell>) -> BufferOperationResult {
        let first = &self.functions[0];
        match first {
            AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) | AnyBufferId::Flat(_) => panic!("cannot write to lower buffer"),
            AnyBufferId::FlatCell(_) => BufferOperationResult::new(
                IntervalSelect {
                    dst,
                    thresholds: self.thresholds,
                    functions: enforce_all_are::<FlatCell>(self.functions),
                },
                dst,
            ),
        }
    }
}

fn enforce_all_are<T: VisitorBufferType>(t: Vec<AnyBufferId>) -> Vec<BufferId<T>> {
    let mut out = Vec::with_capacity(t.len());
    for id in t {
        out.push(T::try_downcast_to(id).unwrap_or_else(|| panic!("{id:?} does not match expected type")))
    }
    out
}
