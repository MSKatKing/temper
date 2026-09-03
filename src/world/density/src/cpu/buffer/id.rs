use crate::cpu::buffer::ty::{BufferType, Full};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BufferId<Type: BufferType> {
    idx: usize,
    __type: PhantomData<Type>,
}

impl<Type: BufferType> Clone for BufferId<Type> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            __type: PhantomData,
        }
    }
}

impl<Type: BufferType> PartialEq for BufferId<Type> {
    fn eq(&self, other: &Self) -> bool {
        self.idx.eq(&other.idx)
    }
}

impl<Type: BufferType> Eq for BufferId<Type> {}

impl<Type: BufferType> Copy for BufferId<Type> {}

impl<Type: BufferType> BufferId<Type> {
    pub const OUT: BufferId<Full> = BufferId::new(0);

    pub const fn new(idx: usize) -> Self {
        Self {
            idx,
            __type: PhantomData,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}
