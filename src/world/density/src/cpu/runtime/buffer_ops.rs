use crate::cpu::buffer::{Buffer, BufferType};
use crate::cpu::{pack_buffer_coord, unpack_buffer_coord};
use std::arch::x86_64;
use std::ops::{Add, Deref, Mul};
use temper_core::math::{lerp3_f32, lerp3_f32_simd};
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_copy_to(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, _| src) }
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_add(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, dst| x86_64::_mm256_add_ps(src, dst)) }
    } else {
        buffer_apply_func(dst, src, f32::add)
    }
}

/// Multiplies the values from `src` and `dst`, storing the result in `dst`. `dst` must be a larger
/// buffer than `src` or nothing will be multiplied.
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_mul(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, dst| x86_64::_mm256_mul_ps(src, dst)) }
    } else {
        buffer_apply_func(dst, src, f32::mul)
    }
}

/// Finds the minimum value between `dst` and `src` and stores the value into `dst`.
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_min(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, dst| x86_64::_mm256_min_ps(src, dst)) }
    } else {
        buffer_apply_func(dst, src, f32::min)
    }
}

/// Finds the maximum value between `dst` and `src` and stores the value into `dst`.
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_max(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, dst| x86_64::_mm256_max_ps(src, dst)) }
    } else {
        buffer_apply_func(dst, src, f32::max)
    }
}

/// Subtracts the values from `dst` by `src`, storing the result in `dst`. `dst` must be a larger
/// buffer than `src` or nothing will be subtracted.
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_sub(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, dst| x86_64::_mm256_sub_ps(dst, src)) }
    } else {
        buffer_apply_func(dst, src, |src, dst| dst - src)
    }
}

/// Divides the values from `dst` by `src`, storing the result in `dst`. `dst` must be a larger
/// buffer than `src` or nothing will be divided.
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
///  * `None`: `dst` was greater than `src`. No data was stored.
#[must_use]
#[inline(always)]
pub fn buffer_div(dst: &mut Buffer, src: &Buffer) -> Option<()> {
    if is_x86_feature_detected!("avx2") {
        // SAFETY: avx2 is supported if we made it here
        unsafe { buffer_apply_func_simd(dst, src, |src, dst| x86_64::_mm256_div_ps(dst, src)) }
    } else {
        buffer_apply_func(dst, src, |src, dst| dst / src)
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
///    returns the value to store into `dst`. The arguments are `fn action(src: f32, dst: f32) -> f32`.
///
/// # Returns
///  * `Some(())`: the operation completed successfully.
///  * `None`: `dst` was greater than `src`. Nothing was stored into `dst`.
#[must_use]
pub fn buffer_apply_func<F: Fn(f32, f32) -> f32>(
    dst: &mut Buffer,
    src: &Buffer,
    action: F,
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
                src.as_chunks::<8>()
                    .0
                    .iter()
                    .enumerate()
                    .for_each(|(i, data)| {
                        let pos = unpack_buffer_coord((i as u32) << 3, BufferType::Interpolated);

                        for y in 0..4 {
                            for z in 0..4 {
                                for x in 0..4 {
                                    let i = pack_buffer_coord(
                                        ChunkBlockPos::new(pos.x() + x, pos.y() + y, pos.z() + z),
                                        BufferType::Full,
                                    ) as usize;

                                    dst[i] = action(
                                        lerp3_f32(
                                            [x as f32 / 4.0, z as f32 / 4.0, y as f32 / 4.0],
                                            *data,
                                        ),
                                        dst[i],
                                    );
                                }
                            }
                        }
                    });
            }
            BufferType::Flat => src.pos_iter().for_each(|(pos, val)| {
                let xz_idx = ((pos.z() as usize) << 4) | (pos.x() as usize);

                for i in (xz_idx..((384 << 8) | xz_idx)).step_by(1 << 8) {
                    dst[i] = action(*val, dst[i]);
                }
            }),
            BufferType::FlatCell => src.pos_iter().for_each(|(pos, val)| {
                for y in -64..320 {
                    for z in 0..4 {
                        for x in 0..4 {
                            let i = pack_buffer_coord(
                                ChunkBlockPos::new(pos.x() + x, y, pos.z() + z),
                                BufferType::Full,
                            ) as usize;

                            dst[i] = action(*val, dst[i]);
                        }
                    }
                }
            }),
        },
        BufferType::Interpolated => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => dst.pos_iter_mut().for_each(|(pos, val)| {
                let i = pack_buffer_coord(ChunkBlockPos::new(pos.x(), 0, pos.z()), BufferType::Flat)
                    as usize;

                *val = action(src[i], *val);
            }),
            BufferType::FlatCell => {
                dst.as_chunks_mut::<8>()
                    .0
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, val)| {
                        let pos = unpack_buffer_coord((i as u32) << 3, BufferType::Interpolated);
                        let i = pack_buffer_coord(pos, BufferType::FlatCell) as usize;

                        val.iter_mut().for_each(|v| {
                            *v = action(src[i], *v);
                        })
                    })
            }
        },
        BufferType::Flat => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => unreachable!(),
            BufferType::FlatCell => dst.iter_mut().enumerate().for_each(|(i, val)| {
                let x = (i >> 2) & 0x3;
                let z = (i >> 6) & 0x3;

                let i = (z << 2) | x;

                *val = action(src[i], *val);
            }),
        },
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
///    returns the value to store into `dst`. The arguments are `fn action(src: f32, dst: f32) -> f32`.
///
/// # Returns
///  * `Some(())`: the operation completed successfully.
///  * `None`: `dst` was greater than `src`. Nothing was stored into `dst`.
#[must_use]
#[target_feature(enable = "avx2")]
pub fn buffer_apply_func_simd<F: Fn(x86_64::__m256, x86_64::__m256) -> x86_64::__m256>(
    dst: &mut Buffer,
    src: &Buffer,
    action: F,
) -> Option<()> {
    if dst.ty < src.ty {
        return None;
    }

    // if the buffers are equal in size we can just copy everything without rearranging
    if dst.ty.size() == src.ty.size() {
        dst.copy_from_slice(src.deref());
        return Some(());
    }

    let z_stride = 0x10; // x width
    let y_stride = z_stride * 0x10; // x width * z width

    match dst.ty {
        BufferType::Out | BufferType::Full => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => {
                let x = x86_64::_mm256_setr_ps(0.0, 0.25, 0.5, 0.75, 0.0, 0.25, 0.5, 0.75);

                let z = [
                    x86_64::_mm256_setr_ps(0.0, 0.0, 0.0, 0.0, 0.25, 0.25, 0.25, 0.25),
                    x86_64::_mm256_setr_ps(0.5, 0.5, 0.5, 0.5, 0.75, 0.75, 0.75, 0.75),
                ];

                let mut src_cell_idx = 0;
                for cell_y in 0..(384 / 4) {
                    let dst_base_y = cell_y * 4 * y_stride;

                    for cell_z in 0..(16 / 4) {
                        let dst_base_z = dst_base_y + cell_z * 4 * z_stride;

                        for cell_x in 0..(16 / 4) {
                            let dst_base_x = dst_base_z + cell_x * 4;

                            let src_data = unsafe {
                                x86_64::_mm256_load_ps(&raw const src[src_cell_idx * 8])
                            };
                            src_cell_idx += 1;

                            for i in 0..8usize {
                                let z_idx = i & 1;
                                let y_idx = i >> 1;
                                let y = x86_64::_mm256_set1_ps(y_idx as f32 / 4.0);

                                let z_offset = z_idx * 2 * z_stride;
                                let y_offset = y_idx * y_stride;
                                let dst_offset = dst_base_x + z_offset + y_offset;

                                unsafe {
                                    let interpolated = lerp3_f32_simd(
                                        [x, z[z_idx], y],
                                        src_data
                                    );

                                    let dst_data = x86_64::_mm256_set_m128(
                                        x86_64::_mm_load_ps(&raw const dst[dst_offset + z_stride]),
                                        x86_64::_mm_load_ps(&raw const dst[dst_offset]),
                                    );

                                    let dst_data = action(interpolated, dst_data);

                                    let lo = x86_64::_mm256_castps256_ps128(dst_data);
                                    let hi = x86_64::_mm256_extractf128_ps::<1>(dst_data);

                                    x86_64::_mm_store_ps(&raw mut dst[dst_offset], lo);
                                    x86_64::_mm_store_ps(&raw mut dst[dst_offset + z_stride], hi)
                                }
                            }
                        }
                    }
                }
            }
            BufferType::Flat => {
                for z in 0..16 {
                    let base_z = z * z_stride;

                    for x in 0..16 {
                        let base_x = base_z + x;

                        let src_data0 = unsafe {
                            x86_64::_mm256_load_ps(&raw const src[base_x])
                        };

                        // x stride is 16, only 8 values fit into a __m256
                        let src_data1 = unsafe {
                            x86_64::_mm256_load_ps(&raw const src[base_x + 0x8])
                        };

                        for y in 0..384 {
                            let dst_base_y = base_x + y * y_stride;

                            unsafe {
                                let dst_data0 = x86_64::_mm256_load_ps(&raw const dst[dst_base_y]);
                                let dst_data1 = x86_64::_mm256_load_ps(&raw const dst[dst_base_y + 0x8]);

                                let dst_data0 = action(src_data0, dst_data0);
                                let dst_data1 = action(src_data1, dst_data1);

                                x86_64::_mm256_store_ps(&raw mut dst[dst_base_y], dst_data0);
                                x86_64::_mm256_store_ps(&raw mut dst[dst_base_y + 0x8], dst_data1);
                            }
                        }
                    }
                }
            },
            BufferType::FlatCell => {
                todo!()
            }
        },
        BufferType::Interpolated => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => {
                let src_offsets = x86_64::_mm256_setr_epi32(
                    0,
                    0x3,
                    0x3 * z_stride as i32,
                    0x3 * z_stride as i32 + 0x3,
                    0,
                    0x3,
                    0x3 * z_stride as i32,
                    0x3 * z_stride as i32 + 0x3,
                );

                for z in 0..4 {
                    let src_base_z = z * 4 * z_stride;
                    let dst_base_z = z * 4;

                    for x in 0..4 {
                        let src_base_x = src_base_z + x * 4;
                        let dst_base_x = dst_base_z + x;

                        let src_data = unsafe {
                            x86_64::_mm256_i32gather_ps::<4>(&raw const src[src_base_x], src_offsets)
                        };

                        for y in 0..(384 / 4) {
                            let dst_base_y = dst_base_x + y * 16;

                            let dst_data = unsafe {
                                x86_64::_mm256_load_ps(&raw const dst[dst_base_y << 3])
                            };

                            let dst_data = action(src_data, dst_data);

                            unsafe {
                                x86_64::_mm256_store_ps(&raw mut dst[dst_base_y << 3], dst_data);
                            }
                        }
                    }
                }
            }
            BufferType::FlatCell => {
                for z in 0..4 {
                    let cell_base_z = z * 4;

                    for x in 0..4 {
                        let cell_base_x = cell_base_z + x;

                        let val = x86_64::_mm256_set1_ps(src[cell_base_x]);

                        for y in 0..(384 / 4) {
                            let dst_base_y = cell_base_x + y * 16;

                            unsafe {
                                let dst_data = x86_64::_mm256_load_ps(&raw const dst[dst_base_y << 3]);

                                let dst_data = action(val, dst_data);

                                x86_64::_mm256_store_ps(&raw mut dst[dst_base_y << 3], dst_data);
                            }
                        }
                    }
                }
            }
        },
        BufferType::Flat => match src.ty {
            BufferType::Out | BufferType::Full => unreachable!(),
            BufferType::Interpolated => unreachable!(),
            BufferType::Flat => unreachable!(),
            BufferType::FlatCell => {
                src.as_chunks::<2>()
                    .0
                    .iter()
                    .enumerate()
                    .for_each(|(i, val)| {
                        let x = (i & 1) * 8;
                        let z = i >> 1;

                        let src_v = x86_64::_mm256_set_m128(
                            x86_64::_mm_set1_ps(val[1]),
                            x86_64::_mm_set1_ps(val[0]),
                        );

                        let dst_v = unsafe { x86_64::_mm256_load_ps(&raw const dst[(z << 4) | x]) };

                        let dst_v = action(src_v, dst_v);

                        unsafe {
                            x86_64::_mm256_store_ps(&raw mut dst[(z << 4) | x], dst_v);
                        }
                    })
            }
        },
        BufferType::FlatCell => unreachable!(),
    }

    Some(())
}
