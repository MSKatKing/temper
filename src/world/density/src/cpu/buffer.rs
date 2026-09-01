use std::ops::{Deref, DerefMut};
use temper_core::pos::ChunkBlockPos;
use crate::cpu::unpack_buffer_coord;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum BufferType {
    Out,
    Full,
    Flat,
    FlatCell,
    Interpolated,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct BufferId {
    pub ty: BufferType,
    pub id: u8,
}

pub struct Buffer {
    pub ty: BufferType,
    pub data: Box<[f32]>,
}

impl BufferType {
    pub fn size(&self) -> usize {
        match self {
            BufferType::Out | BufferType::Full => 16 * 16 * 384,
            BufferType::Flat => 16 * 16,
            BufferType::FlatCell => 4 * 4,
            BufferType::Interpolated => 8 * (4 * 4 * (384 / 4)),
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
            data: vec![0.0; ty.size()].into_boxed_slice(),
            ty,
        }
    }
    
    pub fn pos_iter(&mut self) -> impl Iterator<Item = (ChunkBlockPos, &mut f32)> + '_ {
        self.data
            .iter_mut()
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
