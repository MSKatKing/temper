pub mod buffer;
pub mod compiler;
pub mod noise;
pub mod workspace;
mod runtime;

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::cpu::buffer::{Buffer, BufferId, Flat};
//     use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
//     use crate::cpu::operation::{Operation, ValueSource};
//     use temper_core::pos::ChunkPos;
//     use temper_core::random::XoroshiroRandomSource;
//     use temper_noise::NormalNoise;
// 
//     #[test]
//     pub fn test_simple() {
//         let mut rand = XoroshiroRandomSource::new(10);
// 
//         let ops = [
//             Operation::ClearBuffer {
//                 destination: BufferId::OUT,
//                 source: ValueSource::Constant(0.0),
//             },
//             Operation::AddBuffer {
//                 destination: BufferId::OUT,
//                 source: ValueSource::Noise(NoiseAccessor::new_noise(
//                     NormalNoise::new(&mut rand, 1, &[3.0, 2.0, 1.0]),
//                     NoiseAccessType::Basic {
//                         xz_scale: 1.0,
//                         y_scale: 1.0,
//                     },
//                 )),
//             },
//             Operation::MulBuffer {
//                 destination: BufferId::OUT,
//                 source: ValueSource::Constant(5.0),
//             },
//             Operation::ClearBuffer {
//                 destination: BufferId::<Flat>::new(1),
//                 source: ValueSource::Constant(3.0),
//             },
//             Operation::AddBuffer {
//                 destination: BufferId::<Flat>::new(1),
//                 source: ValueSource::Noise(NoiseAccessor::new_noise(
//                     NormalNoise::new(&mut rand, 4, &[1.0, 2.0, 3.0]),
//                     NoiseAccessType::Basic {
//                         xz_scale: 1.0,
//                         y_scale: 1.0,
//                     },
//                 )),
//             },
//             Operation::AddBuffer {
//                 destination: BufferId::OUT,
//                 source: ValueSource::Buffer(BufferId::<Flat>::new(1)),
//             },
//         ];
// 
//         let mut workspace = Workspace {
//             out: Buffer::new(BufferType::Out),
//             full: Vec::new(),
//             flat: vec![Buffer::new(BufferType::Flat)],
//             flat_cell: Vec::new(),
//             interpolated: Vec::new(),
//             operations: &ops,
//             current_pos: ChunkPos::new(0, 0),
//         };
// 
//         let out_buffer = workspace.execute();
//         assert!(out_buffer.is_some());
//     }
// }
