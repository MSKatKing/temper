use std::alloc::Layout;
use std::num::NonZeroUsize;
use crate::cpu::unpack_buffer_coord;
use std::ops::{Deref, DerefMut};
use temper_core::pos::ChunkBlockPos;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum BufferType {
    FlatCell,
    Flat,
    Interpolated,
    Full,
    Out,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct BufferId {
    pub ty: BufferType,
    pub id: u8,
}

pub struct Buffer {
    pub ty: BufferType,
    data: &'static mut [f32],
}

impl BufferType {
    pub fn size(&self) -> NonZeroUsize {
        match self {
            BufferType::Out | BufferType::Full => NonZeroUsize::new(16 * 16 * 384).expect("non-zero"),
            BufferType::Flat => NonZeroUsize::new(16 * 16).expect("non-zero"),
            BufferType::FlatCell => NonZeroUsize::new(4 * 4).expect("non-zero"),
            BufferType::Interpolated => NonZeroUsize::new(8 * (4 * 4 * (384 / 4))).expect("non-zero"),
        }
    }
}

impl BufferId {
    pub const OUT: BufferId = BufferId {
        ty: BufferType::Out,
        id: 0,
    };

    pub fn flat(id: u8) -> BufferId {
        BufferId {
            ty: BufferType::Flat,
            id,
        }
    }

    pub fn flat_cell(id: u8) -> BufferId {
        BufferId {
            ty: BufferType::FlatCell,
            id,
        }
    }

    pub fn interpolated(id: u8) -> BufferId {
        BufferId {
            ty: BufferType::Interpolated,
            id,
        }
    }
}

impl Buffer {
    pub fn new(ty: BufferType) -> Self {
        Self {
            // on modern hardware, _mm256_load_ps and _mm256_loadu_ps basically have no difference,
            // but _mm256_load_ps is faster on older hardware. it requires that the data being
            // loaded is aligned to 32 bytes, so this block ensures that all buffer's internal data
            // is aligned to 32 bytes on the heap.
            data: unsafe {
                let layout = Layout::from_size_align(
                    size_of::<f32>() * ty.size().get(),
                    32
                ).expect("error creating buffer layout");

                // SAFETY: the layout size is not zero (see BufferType::size)
                let alloc = std::alloc::alloc_zeroed(layout);

                if alloc.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }

                // SAFETY: the pointer is non-null and guaranteed to be valid by alloc_zeroed
                std::slice::from_raw_parts_mut(alloc.cast(), ty.size().get())
            },
            ty,
        }
    }

    pub fn pos_iter_mut(&mut self) -> impl Iterator<Item = (ChunkBlockPos, &mut f32)> + '_ {
        self.data
            .iter_mut()
            .enumerate()
            .map(|(i, v)| (unpack_buffer_coord(i as u32, self.ty), v))
    }

    pub fn pos_iter(&self) -> impl Iterator<Item = (ChunkBlockPos, &f32)> + '_ {
        self.data
            .iter()
            .enumerate()
            .map(|(i, v)| (unpack_buffer_coord(i as u32, self.ty), v))
    }
}

impl Deref for Buffer {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // ensure data (which is allocated on the heap) is properly freed
        //
        // SAFETY: self.data is guaranteed to be allocated on the heap (see Buffer::new) and layout
        // is the same
        unsafe {
            let layout = Layout::from_size_align(
                size_of::<f32>() * self.data.len(),
                32,
            ).expect("error creating buffer layout");

            std::alloc::dealloc(self.data.as_mut_ptr().cast(), layout);
        }
    }
}
