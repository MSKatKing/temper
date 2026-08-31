use crate::cpu::buffer::{Buffer, BufferId, BufferType};
use crate::cpu::compiler::CompiledDensityFunction;
use crate::cpu::operation::Operation;
use crate::cpu::runtime::execute_function;

pub struct Workspace<'func> {
    pub out: Buffer,
    pub full: Vec<Buffer>,
    pub flat: Vec<Buffer>,
    pub flat_cell: Vec<Buffer>,
    pub interpolated: Vec<Buffer>,
    pub operations: &'func [Operation],
}

impl Workspace<'_> {
    pub fn new(density_function: &CompiledDensityFunction) -> Workspace<'_> {
        fn instantiate_buffers(function: &CompiledDensityFunction, ty: BufferType) -> Vec<Buffer> {
            function.buffers
                .get(&ty)
                .map(|buffers| {
                    buffers
                        .iter()
                        .map(|_| Buffer::new(ty))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| Vec::with_capacity(0))
        }

        Workspace {
            out: Buffer::new(BufferType::Out),
            full: instantiate_buffers(density_function, BufferType::Full),
            interpolated: instantiate_buffers(density_function, BufferType::Interpolated),
            flat: instantiate_buffers(density_function, BufferType::Flat),
            flat_cell: instantiate_buffers(density_function, BufferType::FlatCell),
            operations: &density_function.ops,
        }
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<&Buffer> {
        match id.ty {
            BufferType::Out => Some(&self.out),
            BufferType::Full => self.full.get(id.id as usize),
            BufferType::Flat => self.flat.get(id.id as usize),
            BufferType::FlatCell => self.flat_cell.get(id.id as usize),
            BufferType::Interpolated => self.interpolated.get(id.id as usize),
        }
    }

    pub fn get_buffer_mut(&mut self, id: BufferId) -> Option<&mut Buffer> {
        match id.ty {
            BufferType::Out => Some(&mut self.out),
            BufferType::Full => self.full.get_mut(id.id as usize),
            BufferType::Flat => self.flat.get_mut(id.id as usize),
            BufferType::FlatCell => self.flat_cell.get_mut(id.id as usize),
            BufferType::Interpolated => self.interpolated.get_mut(id.id as usize),
        }
    }

    pub fn get_dst_src(&mut self, dst: BufferId, src: BufferId) -> Option<(&mut Buffer, &Buffer)> {
        if dst == src {
            return None;
        }

        match (dst.ty, src.ty) {
            (BufferType::Out, BufferType::Out) => unreachable!(),
            (BufferType::Full, BufferType::Full) => {
                split_two(&mut self.full, dst.id as _, src.id as _)
            }
            (BufferType::Flat, BufferType::Flat) => {
                split_two(&mut self.flat, dst.id as _, src.id as _)
            }
            (BufferType::FlatCell, BufferType::FlatCell) => {
                split_two(&mut self.flat, dst.id as _, src.id as _)
            }
            (BufferType::Interpolated, BufferType::Interpolated) => {
                split_two(&mut self.interpolated, dst.id as _, src.id as _)
            }

            (BufferType::Out, BufferType::Flat) => {
                Some((&mut self.out, self.flat.get(src.id as usize)?))
            }
            (BufferType::Out, BufferType::FlatCell) => {
                Some((&mut self.out, self.flat_cell.get(src.id as usize)?))
            }
            (BufferType::Out, BufferType::Full) => {
                Some((&mut self.out, self.full.get(dst.id as usize)?))
            }
            (BufferType::Out, BufferType::Interpolated) => {
                Some((&mut self.out, self.interpolated.get(dst.id as usize)?))
            }

            (BufferType::Full, BufferType::Out) => {
                Some((self.full.get_mut(dst.id as usize)?, &self.out))
            }
            (BufferType::Full, BufferType::Flat) => Some((
                self.full.get_mut(dst.id as usize)?,
                self.flat.get(src.id as usize)?,
            )),
            (BufferType::Full, BufferType::FlatCell) => Some((
                self.full.get_mut(dst.id as usize)?,
                self.flat_cell.get(src.id as usize)?,
            )),
            (BufferType::Full, BufferType::Interpolated) => Some((
                self.full.get_mut(dst.id as usize)?,
                self.interpolated.get(src.id as usize)?,
            )),

            (BufferType::Flat, BufferType::Out) => {
                Some((self.flat.get_mut(dst.id as usize)?, &self.out))
            }
            (BufferType::Flat, BufferType::Full) => Some((
                self.flat.get_mut(dst.id as usize)?,
                self.full.get(src.id as usize)?,
            )),
            (BufferType::Flat, BufferType::FlatCell) => Some((
                self.flat.get_mut(dst.id as usize)?,
                self.flat_cell.get(src.id as usize)?,
            )),
            (BufferType::Flat, BufferType::Interpolated) => Some((
                self.flat.get_mut(dst.id as usize)?,
                self.interpolated.get(src.id as usize)?,
            )),

            (BufferType::FlatCell, BufferType::Out) => {
                Some((self.flat_cell.get_mut(dst.id as usize)?, &self.out))
            }
            (BufferType::FlatCell, BufferType::Full) => Some((
                self.flat_cell.get_mut(dst.id as usize)?,
                self.full.get(src.id as usize)?,
            )),
            (BufferType::FlatCell, BufferType::Flat) => Some((
                self.flat_cell.get_mut(dst.id as usize)?,
                self.flat.get(src.id as usize)?,
            )),
            (BufferType::FlatCell, BufferType::Interpolated) => Some((
                self.flat_cell.get_mut(dst.id as usize)?,
                self.interpolated.get(src.id as usize)?,
            )),

            (BufferType::Interpolated, BufferType::Out) => {
                Some((self.interpolated.get_mut(dst.id as usize)?, &self.out))
            }
            (BufferType::Interpolated, BufferType::Full) => Some((
                self.interpolated.get_mut(dst.id as usize)?,
                self.full.get(src.id as usize)?,
            )),
            (BufferType::Interpolated, BufferType::Flat) => Some((
                self.interpolated.get_mut(dst.id as usize)?,
                self.flat.get(src.id as usize)?,
            )),
            (BufferType::Interpolated, BufferType::FlatCell) => Some((
                self.interpolated.get_mut(dst.id as usize)?,
                self.flat_cell.get(src.id as usize)?,
            )),
        }
    }

    pub fn execute(&mut self) -> Option<()> {
        execute_function(self)
    }
}

fn split_two<T>(slice: &mut [T], a: usize, b: usize) -> Option<(&mut T, &T)> {
    if a == b {
        return None;
    }

    if a < b {
        let (left, right) = slice.split_at_mut(b);
        Some((&mut left[a], &right[0]))
    } else {
        let (left, right) = slice.split_at_mut(a);
        Some((&mut right[0], &mut left[0]))
    }
}
