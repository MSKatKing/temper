use std::ops::Range;
use crate::cpu::buffer::{BufferId, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::AnyBufferId;
use crate::cpu::compiler::visitor::{BufferOperationResult, BufferOperationVisitor, VisitorBufferType};
use crate::cpu::runtime::RangeChoice;
use crate::impl_visitor_base;

impl_visitor_base!(RangeChoiceVisitor, when_in_range: AnyBufferId, when_out_of_range: AnyBufferId, range: Range<f32>);

impl BufferOperationVisitor for RangeChoiceVisitor {
    fn visit_any<T: VisitorBufferType + 'static>(self, id: BufferId<T>) -> Option<BufferOperationResult> {
        None
    }

    fn visit_full(self, dst: BufferId<Full>) -> BufferOperationResult {
        match self.when_in_range {
            AnyBufferId::Full(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            },
            AnyBufferId::Interpolated(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            },
            AnyBufferId::Flat(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            },
            AnyBufferId::FlatCell(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            }
        }
    }

    fn visit_interpolated(self, dst: BufferId<Interpolated>) -> BufferOperationResult {
        match self.when_in_range {
            AnyBufferId::Full(_) => panic!("cannot write to higher buffer"),
            AnyBufferId::Interpolated(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(_) => panic!("cannot write to higher buffer"),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            },
            AnyBufferId::Flat(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(_) => panic!("cannot write to higher buffer"),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            },
            AnyBufferId::FlatCell(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(_) => panic!("cannot write to higher buffer"),
                AnyBufferId::Interpolated(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            }
        }
    }

    fn visit_flat(self, dst: BufferId<Flat>) -> BufferOperationResult {
        match self.when_in_range {
            AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) => panic!("cannot write to higher buffer"),
            AnyBufferId::Flat(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) => panic!("cannot write to higher buffer"),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            },
            AnyBufferId::FlatCell(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) => panic!("cannot write to higher buffer"),
                AnyBufferId::Flat(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            }
        }
    }

    fn visit_flat_cell(self, dst: BufferId<FlatCell>) -> BufferOperationResult {
        match self.when_in_range {
            AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) | AnyBufferId::Flat(_) => panic!("cannot write to higher buffer"),
            AnyBufferId::FlatCell(when_in_range) => match self.when_out_of_range {
                AnyBufferId::Full(_) | AnyBufferId::Interpolated(_) | AnyBufferId::Flat(_) => panic!("cannot write to higher buffer"),
                AnyBufferId::FlatCell(when_out_of_range) => BufferOperationResult::new(
                    RangeChoice {
                        dst,
                        when_in_range,
                        when_out_of_range,
                        range: self.range,
                    },
                    dst,
                ),
            }
        }
    }
}