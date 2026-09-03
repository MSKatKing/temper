use std::alloc::Layout;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use temper_core::pos::ChunkBlockPos;

mod ty;
mod op;
mod id;

pub use id::BufferId;
pub use ty::*;
pub use op::*;

#[derive(Debug)]
pub struct Buffer<Type: BufferType> {
    data: &'static mut [f32],
    layout: Layout,
    __type: PhantomData<Type>,
}

impl<Type: BufferType> Deref for Buffer<Type> {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<Type: BufferType> DerefMut for Buffer<Type> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<Type: BufferType> Drop for Buffer<Type> {
    fn drop(&mut self) {
        // SAFETY: self.data was allocated on the heap and self.layout was the Layout it was
        // allocated with
        unsafe {
            std::alloc::dealloc(self.data.as_mut_ptr().cast(), self.layout);
        }
    }
}

impl<Type: BufferType> Buffer<Type> {
    pub fn new() -> Self {
        debug_assert_ne!(Type::SIZE, 0);

        // create layout for allocation
        let layout = Layout::from_size_align(Type::SIZE * size_of::<f32>(), 32)
            .expect("Failed to create buffer layout");

        let data = unsafe {
            // layout size is guaranteed to be >0
            let ptr = std::alloc::alloc_zeroed(layout);

            // ensure that ptr is valid, otherwise quit
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            // ptr returned from alloc is guaranteed to be valid
            std::slice::from_raw_parts_mut(ptr.cast::<f32>(), Type::SIZE)
        };

        Self {
            data,
            layout,
            __type: PhantomData,
        }
    }

    pub fn pos_iter(&self) -> impl Iterator<Item = (ChunkBlockPos, &f32)> + '_ {
        self
            .iter()
            .enumerate()
            .map(|(idx, val)|
                (Type::unpack_coord(idx), val)
            )
    }

    pub fn pos_iter_mut(&mut self) -> impl Iterator<Item = (ChunkBlockPos, &mut f32)> + '_ {
        self
            .iter_mut()
            .enumerate()
            .map(|(idx, val)|
                (Type::unpack_coord(idx), val)
            )
    }

    pub fn apply_to<Other: BufferType, Op: BufferOperation>(&self, other: &mut Buffer<Other>)
    where
        Type: BufferApplyTo<Other> {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: buffer contents are always aligned to 32 bytes and if we're here then avx2 is
            // supported by the system
            unsafe {
                Type::apply_to_simd::<Op>(self, other)
            }
        } else {
            Type::apply_to::<Op>(self, other)
        }
    }

    pub fn apply_to_self<Op: BufferOperation>(&mut self) {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: buffer contents are always aligned to 32 bytes and if we're here then avx2 is
            // supported by the system
            unsafe {
                Type::apply_to_self_simd::<Op>(self)
            }
        } else {
            Type::apply_to_self::<Op>(self)
        }
    }
}
