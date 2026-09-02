use crate::cpu::buffer::{Buffer, BufferType};
use crate::cpu::{pack_buffer_coord, unpack_buffer_coord};
use std::ops::{Add, Deref};
use temper_core::math::{lerp3_f32, lerp3_f32_simd};
use temper_core::pos::ChunkBlockPos;
use std::arch::x86_64;

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
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe {
            buffer_apply_func_simd(
                dst,
                src,
                |src, _| src
            )
        }
    } else {
        buffer_apply_func(dst, src, |src, _| src)
    }
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
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe {
            buffer_apply_func_simd(
                dst,
                src,
                |src, dst| x86_64::_mm256_add_ps(src, dst)
            )
        }
    } else {
        buffer_apply_func(dst, src, f32::add)
    }
}

/// Subtracts the values from `dst` by `src`, storing the result in `dst`. `dst` must be a larger
/// buffer than `src` or nothing will be added.
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
///  * `None`: `dst` was greater than `src`. No data was subtracted.
#[must_use]
#[inline(always)]
pub fn buffer_sub_from(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe {
            buffer_apply_func_simd(
                dst,
                src,
                |src, dst| x86_64::_mm256_sub_ps(dst, src)
            )
        }
    } else {
        buffer_apply_func(dst, src, |src, dst| dst - src)
    }
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

/// Expands values from `src` and uses the action function to determine the value to store in `dst`.
/// This function uses SIMD instructions to accomplish this, and will be automatically called if
/// SIMD is supported.
///
/// # Notes
/// This function will perform the interpolation for [`BufferType::Interpolated`]. Those values will
/// be passed in as the `src` value in `action`.
///
/// Additionally, although the values in the `__m256` may not be in order according to position, it
/// is guaranteed that each f32 in `src` and `dst` will map to the same position in the final chunk.
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
#[target_feature(enable = "avx2")]
pub fn buffer_apply_func_simd<F: Fn(x86_64::__m256, x86_64::__m256) -> x86_64::__m256>(
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
                let x = x86_64::_mm256_setr_ps(
                    0.0,
                    0.25,
                    0.5,
                    0.75,
                    0.0,
                    0.25,
                    0.5,
                    0.75
                );

                let z = [
                    x86_64::_mm256_setr_ps(
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.25,
                        0.25,
                        0.25,
                        0.25,
                    ),
                    x86_64::_mm256_setr_ps(
                        0.5,
                        0.5,
                        0.5,
                        0.5,
                        0.75,
                        0.75,
                        0.75,
                        0.75,
                    ),
                ];

                src
                    .chunks_exact(8)
                    .enumerate()
                    .for_each(|(i, data)| {
                        let pos = unpack_buffer_coord((i as u32) << 3, BufferType::Interpolated);

                        let Some(data): Option<[f32; 8]> = data.as_array().cloned() else {
                            unreachable!()
                        };

                        // run 8 times because there's 64 values in the cell
                        for i in 0..8 {
                            let y_idx = (i >> 1) as i16;
                            let y = x86_64::_mm256_set1_ps(y_idx as f32 / 4.0);

                            let z_idx = (i & 1) as usize;

                            let i = pack_buffer_coord(
                                ChunkBlockPos::new(
                                    pos.x(),
                                    pos.y() + y_idx,
                                    pos.z() + 2 * (i & 1),
                                ),
                                BufferType::Full,
                            ) as usize;

                            let src_v = lerp3_f32_simd([x, z[z_idx], y], data);
                            let dst_v = unsafe {
                                x86_64::_mm256_set_m128(
                                    x86_64::_mm_load_ps(&raw const dst[i + 0x10]),
                                    x86_64::_mm_load_ps(&raw const dst[i]),
                                )
                            };

                            let dst_v = action(src_v, dst_v);

                            unsafe {
                                let lo = x86_64::_mm256_castps256_ps128(dst_v);
                                let hi = x86_64::_mm256_extractf128_ps::<1>(dst_v);

                                x86_64::_mm_store_ps(&raw mut dst[i], lo);
                                x86_64::_mm_store_ps(&raw mut dst[i + 0x10], hi)
                            }
                        }
                    });
            },
            BufferType::Flat => {
                src
                    .chunks_exact(8)
                    .enumerate()
                    .for_each(|(i, data)| {
                        let x = (i & 1) * 8;
                        let z = (i >> 1) & 0xF;

                        let src_v = unsafe {
                            x86_64::_mm256_load_ps(data.as_ptr())
                        };

                        for y in 0usize..384 {
                            let idx = (y << 8) | (z << 4) | x;

                            let dst_v = unsafe {
                                x86_64::_mm256_load_ps(&raw const dst[idx])
                            };

                            let dst_v = action(src_v, dst_v);

                            unsafe {
                                x86_64::_mm256_store_ps(&raw mut dst[idx], dst_v);
                            }
                        }
                    })
            },
            BufferType::FlatCell => {
                todo!()
            }
        },
        BufferType::Interpolated => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => {
                dst
                    .chunks_exact_mut(8)
                    .enumerate()
                    .for_each(|(i, val)| {
                        let pos = unpack_buffer_coord((i as u32) << 3, BufferType::Interpolated);

                        let src_v = x86_64::_mm_setr_ps(
                            src[(pos.x() as usize) | ((pos.z() as usize) << 4)],
                            src[(pos.x() as usize + 3) | ((pos.z() as usize) << 4)],
                            src[(pos.x() as usize) | ((pos.z() as usize + 3) << 4)],
                            src[(pos.x() as usize + 3) | ((pos.z() as usize + 3) << 4)],
                        );
                        let src_v = x86_64::_mm256_set_m128(src_v, src_v);

                        let dst_v = unsafe {
                            x86_64::_mm256_load_ps(val.as_ptr())
                        };

                        let dst_v = action(src_v, dst_v);

                        unsafe {
                            x86_64::_mm256_store_ps(val.as_mut_ptr(), dst_v);
                        }
                    })
            },
            BufferType::FlatCell => {
                todo!()
            },
        },
        BufferType::Flat => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => unreachable!(),
            BufferType::FlatCell => {
                src
                    .chunks_exact(2)
                    .enumerate()
                    .for_each(|(i, val)| {
                        let x = (i & 1) * 8;
                        let z = i >> 1;

                        let src_v = x86_64::_mm256_set_m128(
                            x86_64::_mm_set1_ps(val[1]),
                            x86_64::_mm_set1_ps(val[0]),
                        );

                        let dst_v = unsafe {
                            x86_64::_mm256_load_ps(&raw const dst[(z << 4) | x])
                        };

                        let dst_v = action(src_v, dst_v);

                        unsafe {
                            x86_64::_mm256_store_ps(&raw mut dst[(z << 4) | x], dst_v);
                        }
                    })
            }
        }
        BufferType::FlatCell => unreachable!(),
    }

    Some(())
}