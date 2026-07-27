use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum BufferType {
    Out,
    Full,
    Flat,
    FlatCell,
    Interpolated,
}

#[derive(Copy, Clone, Eq, PartialEq)]
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
            BufferType::Out | BufferType::Full => 16 * 16 * 320,
            BufferType::Flat => 16 * 16,
            BufferType::FlatCell => 4 * 4,
            BufferType::Interpolated => 8 * (4 * 4 * (320 / 4))
        }
    }
}

impl BufferId {
    pub const OUT: BufferId = BufferId { ty: BufferType::Out, id: 0 };

    pub fn flat(id: u8) -> BufferId {
        BufferId { ty: BufferType::Flat, id }
    }

    pub fn flat_cell(id: u8) -> BufferId {
        BufferId { ty: BufferType::FlatCell, id }
    }

    pub fn interpolated(id: u8) -> BufferId {
        BufferId { ty: BufferType::Interpolated, id }
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
