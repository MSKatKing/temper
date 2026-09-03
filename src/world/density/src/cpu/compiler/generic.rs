use crate::cpu::buffer::{BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::{AnyBufferId, ToAnyBufferId};
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::{
    BufferAdd, ConstantAdd, FillConstant, FillNoise, NoiseAdd, Operation, YClampedGradient,
};
use crate::cpu::workspace::WorkspaceStorable;
use std::ops::RangeInclusive;

pub struct BufferOperationResult {
    pub op: Box<dyn Operation>,
    pub output_buf: AnyBufferId,
}

impl BufferOperationResult {
    pub fn new<O: Operation + 'static, T: ToAnyBufferId>(op: O, output_buf: BufferId<T>) -> Self {
        Self {
            op: Box::new(op),
            output_buf: T::to_any_buffer_id(output_buf),
        }
    }
}

pub trait BufferOperationVisitor: Sized {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        id: BufferId<T>,
    ) -> Option<BufferOperationResult>;

    fn visit_full(self, id: BufferId<Full>) -> BufferOperationResult {
        self.visit_any(id).unwrap()
    }

    fn visit_interpolated(self, id: BufferId<Interpolated>) -> BufferOperationResult {
        self.visit_any(id).unwrap()
    }

    fn visit_flat(self, id: BufferId<Flat>) -> BufferOperationResult {
        self.visit_any(id).unwrap()
    }

    fn visit_flat_cell(self, id: BufferId<FlatCell>) -> BufferOperationResult {
        self.visit_any(id).unwrap()
    }
}

pub struct BufferAddVisitor {
    pub src: AnyBufferId,
}

pub struct ConstantAddVisitor {
    pub src: f32,
}

pub struct NoiseAddVisitor {
    pub src: NoiseAccessor,
}

pub struct ConstantFillVisitor {
    pub src: f32,
}

pub struct NoiseFillVisitor {
    pub src: NoiseAccessor,
}

#[allow(dead_code)]
pub struct BufferFillVisitor {
    pub src: AnyBufferId,
}

pub struct YClampedGradientVisitor {
    pub y_range: RangeInclusive<i16>,
    pub value_range: RangeInclusive<f32>,
}

impl BufferOperationVisitor for BufferAddVisitor {
    fn visit_any<T: BufferType>(self, _: BufferId<T>) -> Option<BufferOperationResult> {
        None // we cant know precisely what T is so we have to override the specific functions
    }

    fn visit_full(self, id: BufferId<Full>) -> BufferOperationResult {
        match self.src {
            AnyBufferId::Full(full_id) => BufferOperationResult::new(
                BufferAdd::<Full, Full> {
                    dst: id,
                    src: full_id,
                },
                id,
            ),
            AnyBufferId::Interpolated(interpolated_id) => BufferOperationResult::new(
                BufferAdd::<Full, Interpolated> {
                    dst: id,
                    src: interpolated_id,
                },
                id,
            ),
            AnyBufferId::Flat(flat_id) => BufferOperationResult::new(
                BufferAdd::<Full, Flat> {
                    dst: id,
                    src: flat_id,
                },
                id,
            ),
            AnyBufferId::FlatCell(flat_cell_id) => BufferOperationResult::new(
                BufferAdd::<Full, FlatCell> {
                    dst: id,
                    src: flat_cell_id,
                },
                id,
            ),
        }
    }

    fn visit_interpolated(self, id: BufferId<Interpolated>) -> BufferOperationResult {
        match self.src {
            AnyBufferId::Full(full_id) => BufferOperationResult::new(
                BufferAdd::<Full, Interpolated> {
                    dst: full_id,
                    src: id,
                },
                full_id,
            ),
            AnyBufferId::Interpolated(interpolated_id) => BufferOperationResult::new(
                BufferAdd::<Interpolated, Interpolated> {
                    dst: id,
                    src: interpolated_id,
                },
                id,
            ),
            AnyBufferId::Flat(flat_id) => BufferOperationResult::new(
                BufferAdd::<Interpolated, Flat> {
                    dst: id,
                    src: flat_id,
                },
                id,
            ),
            AnyBufferId::FlatCell(flat_cell_id) => BufferOperationResult::new(
                BufferAdd::<Interpolated, FlatCell> {
                    dst: id,
                    src: flat_cell_id,
                },
                id,
            ),
        }
    }

    fn visit_flat(self, id: BufferId<Flat>) -> BufferOperationResult {
        match self.src {
            AnyBufferId::Full(full_id) => BufferOperationResult::new(
                BufferAdd::<Full, Flat> {
                    dst: full_id,
                    src: id,
                },
                full_id,
            ),
            AnyBufferId::Interpolated(interpolated_id) => BufferOperationResult::new(
                BufferAdd::<Interpolated, Flat> {
                    dst: interpolated_id,
                    src: id,
                },
                interpolated_id,
            ),
            AnyBufferId::Flat(flat_id) => BufferOperationResult::new(
                BufferAdd::<Flat, Flat> {
                    dst: id,
                    src: flat_id,
                },
                id,
            ),
            AnyBufferId::FlatCell(flat_cell_id) => BufferOperationResult::new(
                BufferAdd::<Flat, FlatCell> {
                    dst: id,
                    src: flat_cell_id,
                },
                id,
            ),
        }
    }

    fn visit_flat_cell(self, id: BufferId<FlatCell>) -> BufferOperationResult {
        match self.src {
            AnyBufferId::Full(full_id) => BufferOperationResult::new(
                BufferAdd::<Full, FlatCell> {
                    dst: full_id,
                    src: id,
                },
                full_id,
            ),
            AnyBufferId::Interpolated(interpolated_id) => BufferOperationResult::new(
                BufferAdd::<Interpolated, FlatCell> {
                    dst: interpolated_id,
                    src: id,
                },
                interpolated_id,
            ),
            AnyBufferId::Flat(flat_id) => BufferOperationResult::new(
                BufferAdd::<Flat, FlatCell> {
                    dst: flat_id,
                    src: id,
                },
                flat_id,
            ),
            AnyBufferId::FlatCell(flat_cell_id) => BufferOperationResult::new(
                BufferAdd::<FlatCell, FlatCell> {
                    dst: id,
                    src: flat_cell_id,
                },
                id,
            ),
        }
    }
}

impl BufferOperationVisitor for ConstantAddVisitor {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        id: BufferId<T>,
    ) -> Option<BufferOperationResult> {
        Some(BufferOperationResult::new(
            ConstantAdd::<T> {
                dst: id,
                src: self.src,
            },
            id,
        ))
    }
}

impl BufferOperationVisitor for NoiseAddVisitor {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        id: BufferId<T>,
    ) -> Option<BufferOperationResult> {
        Some(BufferOperationResult::new(
            NoiseAdd::<T> {
                dst: id,
                src: self.src,
            },
            id,
        ))
    }
}

impl BufferOperationVisitor for ConstantFillVisitor {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        id: BufferId<T>,
    ) -> Option<BufferOperationResult> {
        Some(BufferOperationResult::new(
            FillConstant::<T> {
                dst: id,
                src: self.src,
            },
            id,
        ))
    }
}

impl BufferOperationVisitor for NoiseFillVisitor {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        id: BufferId<T>,
    ) -> Option<BufferOperationResult> {
        Some(BufferOperationResult::new(
            FillNoise::<T> {
                dst: id,
                noise: self.src,
            },
            id,
        ))
    }
}

impl BufferOperationVisitor for BufferFillVisitor {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        _id: BufferId<T>,
    ) -> Option<BufferOperationResult> {
        None // we cant know precisely what T is so we have to override the specific functions
    }
}

impl BufferOperationVisitor for YClampedGradientVisitor {
    fn visit_any<T: WorkspaceStorable + ToAnyBufferId + 'static>(
        self,
        id: BufferId<T>,
    ) -> Option<BufferOperationResult> {
        Some(BufferOperationResult::new(
            YClampedGradient::<T> {
                dst: id,
                y_range: self.y_range,
                value_range: self.value_range,
            },
            id,
        ))
    }
}
