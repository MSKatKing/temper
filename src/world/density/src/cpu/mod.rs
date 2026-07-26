use crate::cpu::buffer::{Buffer, BufferId, BufferType};
use crate::cpu::operation::{Operation, ValueSource};
use bevy_math::DVec3;

pub mod operation;
pub mod buffer;
pub mod meta;

pub const OUT_BUFFER_LEN: usize = 16 * 16 * 320;

pub struct Workspace {
    pub out: Buffer,
    pub full: Vec<Buffer>,
    pub flat: Vec<Buffer>,
    pub flat_cell: Vec<Buffer>,
    pub interpolated: Vec<Buffer>,
}

impl Workspace {
    fn get_buffer(&self, id: BufferId) -> Option<&Buffer> {
        match id.ty {
            BufferType::Out => Some(&self.out),
            BufferType::Full => self.full.get(id.id as usize),
            BufferType::Flat => self.flat.get(id.id as usize),
            BufferType::FlatCell => self.flat_cell.get(id.id as usize),
            BufferType::Interpolated => self.interpolated.get(id.id as usize),
        }
    }

    fn get_buffer_mut(&mut self, id: BufferId) -> Option<&mut Buffer> {
        match id.ty {
            BufferType::Out => Some(&mut self.out),
            BufferType::Full => self.full.get_mut(id.id as usize),
            BufferType::Flat => self.flat.get_mut(id.id as usize),
            BufferType::FlatCell => self.flat_cell.get_mut(id.id as usize),
            BufferType::Interpolated => self.interpolated.get_mut(id.id as usize),
        }
    }
}

pub fn run(ops: &[Operation], workspace: &mut Workspace) -> Option<()> {
    for op in ops {
        match op {
            Operation::ClearBuffer {
                destination,
                value
            } => {
                let buffer = workspace.get_buffer_mut(*destination)?;
                buffer.data.fill(*value);
            },
            Operation::AddBuffer {
                destination,
                source: ValueSource::Constant(v),
            } => {
                let buffer = workspace.get_buffer_mut(*destination)?;
                buffer.data.iter_mut().for_each(|f| *f += v);
            },
            Operation::AddBuffer {
                destination,
                source: ValueSource::Noise(n),
            } => {
                let buffer = workspace.get_buffer_mut(*destination)?;
                buffer.data.iter_mut().enumerate().for_each(|(i, f)| {
                    let (x, y, z) = unpack_coord(i as u32);
                    *f += n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32;
                })
            },
            Operation::AddBuffer {
                destination,
                source: ValueSource::Buffer(id),
            } if destination == id => {
                let buf = workspace.get_buffer_mut(*id)?;
                buf.data.iter_mut().for_each(|f| *f *= 2.0);
            },
            Operation::AddBuffer {
                destination,
                source: ValueSource::Buffer(id),
            } if destination.ty == id.ty => {
                let source = workspace.get_buffer(*id)?;

                // SAFETY: the previous match statement checks if the destination and source are
                // the same. if we are here, they are not, so it is ok to borrow the source and
                // mutably borrow the destination from the same workspace as they are guaranteed
                // to be different buffers.
                let dest = unsafe {
                    // Basically telling the compiler to forget that workspace is immutably borrowed
                    // by source.
                    (workspace as *const Workspace).cast_mut().as_mut_unchecked().get_buffer_mut(*destination)?
                };

                dest.data.iter_mut().zip(source.data.iter()).for_each(|(d, s)| *d += s);
            },
            Operation::AddBuffer {
                destination,
                source: ValueSource::Buffer(id),
            } => {
                let source = workspace.get_buffer(*id)?;

                // SAFETY: the previous match statement checks if the destination and source are
                // the same. if we are here, they are not, so it is ok to borrow the source and
                // mutably borrow the destination from the same workspace as they are guaranteed
                // to be different buffers.
                let dest = unsafe {
                    // Basically telling the compiler to forget that workspace is immutably borrowed
                    // by source.
                    (workspace as *const Workspace).cast_mut().as_mut_unchecked().get_buffer_mut(*destination)?
                };

                dest.data.iter_mut().enumerate().for_each(|(i, f)| *f += source.data[0]); //id.ty.transform_idx(&destination.ty, i)
            }
            Operation::MulBuffer {
                destination,
                source,
            } => {
                let buffer = workspace.get_buffer_mut(*destination)?;

                for (i, pos) in buffer.data.iter_mut().enumerate() {
                    *pos *= source.get(unpack_coord(i as _));
                }
            }
        }
    }

    Some(())
}

fn unpack_coord(coord: u32) -> (u8, i16, u8) {
    let x = coord as u8 & 0xF;
    let z = (coord >> 4) as u8 & 0xF;
    let y = (coord >> 8) as i16 - 64;
    (x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::buffer::BufferId;
    use crate::cpu::operation::ValueSource;
    use temper_core::random::XoroshiroRandomSource;
    use temper_noise::NormalNoise;

    #[test]
    pub fn test_simple() {
        let mut rand = XoroshiroRandomSource::new(10);

        let ops = [
            Operation::ClearBuffer { destination: BufferId::OUT, value: 0.0 },
            Operation::AddBuffer { destination: BufferId::OUT, source: ValueSource::Noise(NormalNoise::new(&mut rand, 1, &[3.0, 2.0, 1.0])) },
            Operation::MulBuffer { destination: BufferId::OUT, source: ValueSource::Constant(5.0) },
            Operation::ClearBuffer { destination: BufferId::flat(0), value: 3.0 },
            Operation::AddBuffer { destination: BufferId::flat(0), source: ValueSource::Noise(NormalNoise::new(&mut rand, 4, &[1.0, 2.0, 3.0])) },
            Operation::AddBuffer { destination: BufferId::OUT, source: ValueSource::Buffer(BufferId::flat(0)) },
        ];

        let mut workspace = Workspace {
            out: Buffer { ty: BufferType::Out, data: vec![0.0; OUT_BUFFER_LEN].into_boxed_slice() },
            full: Vec::new(),
            flat: vec![Buffer { ty: BufferType::Flat, data: vec![0.0; BufferType::Flat.size()].into_boxed_slice() }],
            flat_cell: Vec::new(),
            interpolated: Vec::new(),
        };

        let out_buffer = run(&ops, &mut workspace);
        assert!(out_buffer.is_some());
    }
}
