use crate::cpu::buffer::{Buffer, BufferType};
use crate::cpu::{pack_buffer_coord, unpack_buffer_coord};
use std::ops::{Add, Deref};
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
#[inline(always)]
pub fn buffer_copy_to(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    buffer_apply_func(dst, src, |src, _| src)
}

/// Adds the values from `src` and `dst`, storing the result in `dst`. `dst` must be a larger buffer
/// than `src` or nothing will be added.
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
///  * `None`: `dst` was greater than `src`. No data was added.
#[must_use]
#[inline(always)]
pub fn buffer_add_to(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    buffer_apply_func(dst, src, f32::add)
}

/// Expands values from `src` and uses the action function to determine the value to store in `dst`.
///
/// # Notes
/// This function will perform the interpolation for [`BufferType::Interpolated`]. Those values will
/// be passed in as the `src` value in `action`.
///
/// # Arguments
///  * `dst`: the destination buffer.
///  * `src`: the source buffer.
///  * `action`: the function that takes in two f32s, one from `src` and one from `dst`, and
/// returns the value to store into `dst`. The arguments are `fn action(src: f32, dst: f32) -> f32`.
///
/// # Returns
///  * `Some(())`: the operation completed successfully.
///  * `None`: `dst` was greater than `src`. Nothing was stored into `dst`.
#[must_use]
pub fn buffer_apply_func<F: Fn(f32, f32) -> f32>(
    dst: &mut Buffer,
    src: &Buffer,
    action: F
) -> Option<()> {
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

                                    dst[i] = action(
                                        lerp3_f32(
                                            [x as f32 / 4.0, z as f32 / 4.0, y as f32 / 4.0],
                                            data
                                        ),
                                        dst[i],
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
                        let xz_idx = ((pos.z() as usize) << 4) | (pos.x() as usize);

                        for i in (xz_idx..((384 << 8) | xz_idx)).step_by(1 << 8) {
                            dst[i] = action(
                                *val,
                                dst[i],
                            );
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

                                    dst[i] = action(
                                        *val,
                                        dst[i],
                                    );
                                }
                            }
                        }
                    })
            }
        },
        BufferType::Interpolated => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => {
                dst
                    .pos_iter_mut()
                    .for_each(|(pos, val)| {
                        let i = pack_buffer_coord(
                            ChunkBlockPos::new(
                                pos.x(),
                                0,
                                pos.z(),
                            ),
                            BufferType::Flat,
                        ) as usize;

                        *val = action(
                            src[i],
                            *val,
                        );
                    })
            },
            BufferType::FlatCell => {
                dst
                    .chunks_exact_mut(8)
                    .enumerate()
                    .for_each(|(i, val)| {
                        let pos = unpack_buffer_coord(
                            (i as u32) << 3,
                            BufferType::Interpolated
                        );
                        let i = pack_buffer_coord(pos, BufferType::FlatCell) as usize;

                        val
                            .iter_mut()
                            .for_each(|v| {
                                *v = action(
                                    src[i],
                                    *v,
                                );
                            })
                    })
            },
        },
        BufferType::Flat => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => unreachable!(),
            BufferType::FlatCell => {
                dst
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, val)| {
                        let x = (i >> 2) & 0x3;
                        let z = (i >> 6) & 0x3;

                        let i = (z << 2) | x;

                        *val = action(
                            src[i],
                            *val
                        );
                    })
            }
        }
        BufferType::FlatCell => unreachable!(),
    }

    Some(())
}
