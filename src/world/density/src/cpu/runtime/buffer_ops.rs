use crate::cpu::buffer::{Buffer, BufferType};
use crate::cpu::{pack_buffer_coord, unpack_buffer_coord};
use std::ops::Deref;
use temper_core::math::lerp3_f32;
use temper_core::pos::ChunkBlockPos;

/// Copies the values from `src` into `dst`. `dst` must be a larger buffer than `src` or nothing
/// will be copied.
///
/// # Notes
/// This function will perform the interpolation for [`BufferType::Interpolated`].
///
/// # Arguments
///  * `dst`: the destination buffer.
///  * `src`: the source buffer.
///
/// # Returns
///  * `Some(())`: the operation completed successfully.
///  * `None`: `dst` was greater than `src`. No data was copied.
#[must_use]
pub fn buffer_copy_to(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if dst.ty < src.ty {
        return None;
    }

    // if the buffers are equal in size we can just copy everything without rearranging
    if dst.ty.size() == src.ty.size() {
        dst.copy_from_slice(src.deref());
        return Some(());
    }

    match dst.ty {
        BufferType::Out | BufferType::Full => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => {
                src
                    .chunks_exact(8)
                    .enumerate()
                    .for_each(|(i, data)| {
                        let pos = unpack_buffer_coord((i as u32) << 3, BufferType::Interpolated);

                        let Some(data): Option<[f32; 8]> = data.as_array().cloned() else {
                            unreachable!()
                        };

                        for y in 0..4 {
                            for z in 0..4 {
                                for x in 0..4 {
                                    let i = pack_buffer_coord(
                                        ChunkBlockPos::new(
                                            pos.x() + x,
                                            pos.y() + y,
                                            pos.z() + z,
                                        ),
                                        BufferType::Full,
                                    ) as usize;

                                    dst[i] = lerp3_f32(
                                        [x as f32 / 4.0, z as f32 / 4.0, y as f32 / 4.0],
                                        data
                                    );
                                }
                            }
                        }
                    });
            },
            BufferType::Flat => {
                src
                    .pos_iter()
                    .for_each(|(pos, val)| {
                        for y in -64..320 {
                            let i = pack_buffer_coord(
                                ChunkBlockPos::new(
                                    pos.x(),
                                    y,
                                    pos.z(),
                                ),
                                BufferType::Full
                            ) as usize;

                            dst[i] = *val;
                        }
                    })
            },
            BufferType::FlatCell => {
                src
                    .pos_iter()
                    .for_each(|(pos, val)| {
                        for y in -64..320 {
                            for z in 0..4 {
                                for x in 0..4 {
                                    let i = pack_buffer_coord(
                                        ChunkBlockPos::new(
                                            pos.x() + x,
                                            y,
                                            pos.z() + z,
                                        ),
                                        BufferType::Full,
                                    ) as usize;

                                    dst[i] = *val;
                                }
                            }
                        }
                    })
            }
        },
        _ => todo!()
    }

    Some(())
}