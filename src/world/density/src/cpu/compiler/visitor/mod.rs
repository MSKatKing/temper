mod math;

use crate::cpu::buffer::{BufferId, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::{AnyBufferId, ToAnyBufferId};
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::{FillBuffer, FillConstant, FillNoise, Operation};
use crate::cpu::workspace::WorkspaceStorable;

pub use math::*;

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

#[macro_export]
macro_rules! impl_visitor_base {
    ($name:ident, $($field:ident: $field_ty:ty),+ $(,)?) => {
        pub struct $name {
            $($field: $field_ty),+
        }

        impl $name {
            #[allow(clippy::new_ret_no_self)]
            pub fn new(dst: $crate::cpu::compiler::AnyBufferId, $($field: $field_ty),+) -> $crate::cpu::compiler::visitor::BufferOperationResult {
                dst.visit($name { $($field),+ })
            }
        }
    };
}

#[macro_export]
macro_rules! impl_commutative_visitor {
    ($visitor:ty, $operation:ident, $src_field:ident) => {
        impl $crate::cpu::compiler::visitor::BufferOperationVisitor for $visitor {
            fn visit_any<
                T: $crate::cpu::workspace::WorkspaceStorable
                    + $crate::cpu::compiler::ToAnyBufferId
                    + 'static,
            >(
                self,
                _id: $crate::cpu::buffer::BufferId<T>,
            ) -> Option<$crate::cpu::compiler::visitor::BufferOperationResult> {
                None // we need to define different behavior based on dst and src types
            }

            fn visit_full(
                self,
                id: $crate::cpu::buffer::BufferId<$crate::cpu::buffer::Full>,
            ) -> $crate::cpu::compiler::visitor::BufferOperationResult {
                match self.$src_field {
                    AnyBufferId::Full(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Full, $crate::cpu::buffer::Full> {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                    AnyBufferId::Interpolated(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Full,
                                $crate::cpu::buffer::Interpolated,
                            > {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                    AnyBufferId::Flat(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Full, $crate::cpu::buffer::Flat> {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                    AnyBufferId::FlatCell(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Full, $crate::cpu::buffer::FlatCell> {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                }
            }

            fn visit_interpolated(
                self,
                id: $crate::cpu::buffer::BufferId<$crate::cpu::buffer::Interpolated>,
            ) -> $crate::cpu::compiler::visitor::BufferOperationResult {
                match self.$src_field {
                    AnyBufferId::Full(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Full,
                                $crate::cpu::buffer::Interpolated,
                            > {
                                dst: other,
                                src: id,
                            },
                            other,
                        )
                    }
                    AnyBufferId::Interpolated(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Interpolated,
                                $crate::cpu::buffer::Interpolated,
                            > {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                    AnyBufferId::Flat(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Interpolated,
                                $crate::cpu::buffer::Flat,
                            > {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                    AnyBufferId::FlatCell(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Interpolated,
                                $crate::cpu::buffer::FlatCell,
                            > {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                }
            }

            fn visit_flat(
                self,
                id: $crate::cpu::buffer::BufferId<$crate::cpu::buffer::Flat>,
            ) -> $crate::cpu::compiler::visitor::BufferOperationResult {
                match self.$src_field {
                    AnyBufferId::Full(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Full, $crate::cpu::buffer::Flat> {
                                dst: other,
                                src: id,
                            },
                            other,
                        )
                    }
                    AnyBufferId::Interpolated(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Interpolated,
                                $crate::cpu::buffer::Flat,
                            > {
                                dst: other,
                                src: id,
                            },
                            other,
                        )
                    }
                    AnyBufferId::Flat(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Flat, $crate::cpu::buffer::Flat> {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                    AnyBufferId::FlatCell(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Flat, $crate::cpu::buffer::FlatCell> {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                }
            }

            fn visit_flat_cell(
                self,
                id: $crate::cpu::buffer::BufferId<$crate::cpu::buffer::FlatCell>,
            ) -> $crate::cpu::compiler::visitor::BufferOperationResult {
                match self.$src_field {
                    AnyBufferId::Full(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Full, $crate::cpu::buffer::FlatCell> {
                                dst: other,
                                src: id,
                            },
                            other,
                        )
                    }
                    AnyBufferId::Interpolated(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::Interpolated,
                                $crate::cpu::buffer::FlatCell,
                            > {
                                dst: other,
                                src: id,
                            },
                            other,
                        )
                    }
                    AnyBufferId::Flat(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<$crate::cpu::buffer::Flat, $crate::cpu::buffer::FlatCell> {
                                dst: other,
                                src: id,
                            },
                            other,
                        )
                    }
                    AnyBufferId::FlatCell(other) => {
                        $crate::cpu::compiler::visitor::BufferOperationResult::new(
                            $operation::<
                                $crate::cpu::buffer::FlatCell,
                                $crate::cpu::buffer::FlatCell,
                            > {
                                dst: id,
                                src: other,
                            },
                            id,
                        )
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_direct_visitor {
    ($visitor:ty, $operation:ident, $dst_field:ident, $($field:ident: $self_field:ident),*) => {
        impl $crate::cpu::compiler::visitor::BufferOperationVisitor for $visitor {
            fn visit_any<T: $crate::cpu::workspace::WorkspaceStorable + $crate::cpu::compiler::ToAnyBufferId + 'static>(self, id: $crate::cpu::buffer::BufferId<T>) -> Option<$crate::cpu::compiler::visitor::BufferOperationResult> {
                Some($crate::cpu::compiler::visitor::BufferOperationResult::new($operation::<T> {
                    $dst_field: id,
                    $(
                        $field: self.$self_field
                    ),*
                }, id))
            }
        }
    };
}

impl_visitor_base!(FillBufferVisitor, other: AnyBufferId);
impl_visitor_base!(FillConstantVisitor, other: f32);
impl_visitor_base!(FillNoiseVisitor, other: NoiseAccessor);

impl_commutative_visitor!(FillBufferVisitor, FillBuffer, other);
impl_direct_visitor!(FillConstantVisitor, FillConstant, dst, src: other);
impl_direct_visitor!(FillNoiseVisitor, FillNoise, dst, noise: other);
