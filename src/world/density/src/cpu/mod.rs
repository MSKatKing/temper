use temper_core::pos::ChunkBlockPos;
use crate::cpu::buffer::BufferType;
use crate::cpu::workspace::Workspace;

pub mod buffer;
pub mod operation;
mod runtime;
pub mod workspace;
pub mod compiler;
pub mod noise;

pub const OUT_BUFFER_LEN: usize = 16 * 16 * 320;

fn unpack_coord(coord: u32) -> ChunkBlockPos {
    let x = coord as u8 & 0xF;
    let z = (coord >> 4) as u8 & 0xF;
    let y = (coord >> 8) as i16 - 64;
    ChunkBlockPos::new(x, y, z)
}

fn unpack_buffer_coord(coord: u32, buffer_type: BufferType) -> ChunkBlockPos {
    match buffer_type {
        BufferType::Out | BufferType::Full => unpack_coord(coord),
        BufferType::Interpolated => {
            let corner_idx = coord & 0x7;
            let cell_idx = coord >> 3;

            let cell_x = (cell_idx as u8 & 0x4) * 4;
            let cell_z = ((cell_idx >> 2) as u8 & 0x4) * 4;
            let cell_y = ((cell_idx >> 4) as i16) * 4;

            ChunkBlockPos::new(
                cell_x + 3 * (corner_idx & 1) as u8,
                cell_y + 3 * (corner_idx >> 2 & 1) as i16 - 64,
                cell_z + 3 * (corner_idx >> 1 & 1) as u8,
            )
        },
        BufferType::Flat => {
            let x = coord as u8 & 0xF;
            let z = (coord >> 4) as u8 & 0xF;
            ChunkBlockPos::new(x, 0, z)
        },
        BufferType::FlatCell => {
            let x = (coord as u8 & 0x4) * 4;
            let z = ((coord >> 2) as u8 & 0x4) * 4;
            ChunkBlockPos::new(x, 0, z)
        }
    }
}

#[cfg(test)]
mod tests {
    use temper_core::pos::ChunkPos;
    use super::*;
    use crate::cpu::buffer::{Buffer, BufferId, BufferType};
    use crate::cpu::operation::{Operation, Projection, ValueSource};
    use temper_core::random::XoroshiroRandomSource;
    use temper_noise::NormalNoise;
    use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};

    #[test]
    pub fn test_simple() {
        let mut rand = XoroshiroRandomSource::new(10);

        let ops = [
            Operation::ClearBuffer {
                destination: BufferId::OUT,
                source: ValueSource::Constant(0.0),
            },
            Operation::AddBuffer {
                destination: BufferId::OUT,
                source: ValueSource::Noise(
                    NoiseAccessor::new_noise(
                        NormalNoise::new(&mut rand, 1, &[3.0, 2.0, 1.0]),
                        NoiseAccessType::Basic { xz_scale: 1.0, y_scale: 1.0 },
                    )
                ),
            },
            Operation::MulBuffer {
                destination: BufferId::OUT,
                source: ValueSource::Constant(5.0),
            },
            Operation::ClearBuffer {
                destination: BufferId::flat(0),
                source: ValueSource::Constant(3.0),
            },
            Operation::AddBuffer {
                destination: BufferId::flat(0),
                source: ValueSource::Noise(
                    NoiseAccessor::new_noise(
                        NormalNoise::new(&mut rand, 4, &[1.0, 2.0, 3.0]),
                        NoiseAccessType::Basic { xz_scale: 1.0, y_scale: 1.0 }
                    )
                ),
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
            current_pos: ChunkPos::new(0, 0),
        };

        let out_buffer = workspace.execute();
        assert!(out_buffer.is_some());
    }
}
