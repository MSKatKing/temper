use crate::cpu::workspace::Workspace;

pub mod buffer;
pub mod operation;
mod runtime;
pub mod workspace;
mod compiler;

pub const OUT_BUFFER_LEN: usize = 16 * 16 * 320;

fn unpack_coord(coord: u32) -> (u8, i16, u8) {
    let x = coord as u8 & 0xF;
    let z = (coord >> 4) as u8 & 0xF;
    let y = (coord >> 8) as i16 - 64;
    (x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::buffer::{Buffer, BufferId, BufferType};
    use crate::cpu::operation::{Operation, Projection, ValueSource};
    use temper_core::random::XoroshiroRandomSource;
    use temper_noise::NormalNoise;

    #[test]
    pub fn test_simple() {
        let mut rand = XoroshiroRandomSource::new(10);

        let ops = [
            Operation::ClearBuffer {
                destination: BufferId::OUT,
                value: 0.0,
            },
            Operation::AddBuffer {
                destination: BufferId::OUT,
                source: ValueSource::Noise(NormalNoise::new(&mut rand, 1, &[3.0, 2.0, 1.0])),
            },
            Operation::MulBuffer {
                destination: BufferId::OUT,
                source: ValueSource::Constant(5.0),
            },
            Operation::ClearBuffer {
                destination: BufferId::flat(0),
                value: 3.0,
            },
            Operation::AddBuffer {
                destination: BufferId::flat(0),
                source: ValueSource::Noise(NormalNoise::new(&mut rand, 4, &[1.0, 2.0, 3.0])),
            },
            Operation::AddBuffer {
                destination: BufferId::OUT,
                source: ValueSource::Buffer(BufferId::flat(0), Projection::DropY),
            },
        ];

        let mut workspace = Workspace {
            out: Buffer {
                ty: BufferType::Out,
                data: vec![0.0; OUT_BUFFER_LEN].into_boxed_slice(),
            },
            full: Vec::new(),
            flat: vec![Buffer {
                ty: BufferType::Flat,
                data: vec![0.0; BufferType::Flat.size()].into_boxed_slice(),
            }],
            flat_cell: Vec::new(),
            interpolated: Vec::new(),
            operations: &ops,
        };

        let out_buffer = workspace.execute();
        assert!(out_buffer.is_some());
    }
}
