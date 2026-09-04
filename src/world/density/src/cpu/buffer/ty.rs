use crate::cpu::buffer::Buffer;
use crate::cpu::buffer::op::BufferOperation;
use std::arch::x86_64;
use std::fmt::Debug;
use temper_core::math::{lerp3_f32, lerp3_f32_simd};
use temper_core::pos::ChunkBlockPos;

pub trait BufferType: Sized + Debug + Send + Sync {
    const SIZE: usize;
    const LEVEL: usize;
    const Z_STRIDE: usize;
    const Y_STRIDE: usize;

    fn unpack_coord(idx: usize) -> ChunkBlockPos;
    fn pack_coord(pos: ChunkBlockPos) -> usize;

    /// Performs the BufferOperation on self (`src` and `dst` are both values from self).
    fn apply_to_self<T: BufferOperation>(this: &mut Buffer<Self>, apply: T) {
        this.iter_mut().for_each(|v| *v = apply.scalar(*v, *v))
    }

    /// Performs the BufferOperation on self (`src` and `dst` are both values from self).
    ///
    /// # Safety
    /// The caller must ensure that the system supports the avx2 feature set.
    #[target_feature(enable = "avx2")]
    unsafe fn apply_to_self_simd<T: BufferOperation>(this: &mut Buffer<Self>, apply: T) {
        this.as_chunks_mut::<8>().0.iter_mut().for_each(|v| unsafe {
            let src = x86_64::_mm256_load_ps(v.as_ptr());
            x86_64::_mm256_store_ps(v.as_mut_ptr(), apply.simd(src, src))
        })
    }
}

pub trait BufferApplyTo<Dst: BufferType>: BufferType {
    /// Moves bytes from `src` into `dst`, first applying the `apply` function. The `apply` function
    /// should be in this format:
    ///
    /// ```rust
    /// fn apply(src_value: f32, dst_value: f32) -> f32 {
    ///     todo!()
    /// }
    /// ```
    ///
    /// The resulting data from `apply` will be stored in `dst`.
    ///
    /// # Arguments
    /// `src`: the source buffer
    /// `dst`: the destination buffer
    /// `apply`: the function to transform the values from `src` and `dst` before writing.
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Dst>, apply: F);

    /// Moves bytes from `src` into `dst`, first applying the `apply` function. The `apply` function
    /// should be in this format:
    ///
    /// ```rust
    /// use std::arch::x86_64::__m256;
    ///
    /// fn apply(src_value: __m256, dst_value: __m256) -> __m256 {
    ///     todo!()
    /// }
    /// ```
    ///
    /// The resulting data from `apply` will be stored in `dst`. This function is the SIMD version
    /// of [`apply_to`](BufferApplyTo::apply_to).
    ///
    /// # Safety
    /// The caller must guarantee these preconditions, otherwise this function may cause UB or
    /// crash the program:
    ///
    ///  * The system must support at the minimum avx2 SIMD. It also must support any additional
    ///    features used by `apply`.
    ///  * The data in the buffers must be aligned to 32-bytes or the processor will throw an error.
    ///    This function may make use of [`_mm256_load_ps`](x86_64::_mm256_load_ps) and
    ///    [`_mm256_store_ps`](x86_64::_mm256_store_ps) or the 128-bit equivalent.
    ///
    /// # Arguments
    /// `src`: the source buffer
    /// `dst`: the destination buffer
    /// `apply`: the function to transform the values from `src` and `dst` before writing.
    #[cfg(target_arch = "x86_64")]
    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Dst>, apply: F);
}

impl<Dst: BufferType> BufferApplyTo<Dst> for Dst {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Dst>, apply: F) {
        dst.iter_mut()
            .zip(src.iter())
            .for_each(|(dst, src)| *dst = apply.scalar(*src, if F::READS_DST { *dst } else { 0.0 }));
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Dst>, apply: F) {
        dst.as_chunks_mut::<8>()
            .0
            .iter_mut()
            .zip(src.as_chunks::<8>().0.iter())
            .for_each(|(dst, src)| unsafe {
                let src_v = x86_64::_mm256_load_ps(src.as_ptr());
                let dst_v = if F::READS_DST {
                    x86_64::_mm256_load_ps(dst.as_ptr())
                } else {
                    x86_64::_mm256_setzero_ps()
                };

                let dst_v = apply.simd(src_v, dst_v);

                x86_64::_mm256_store_ps(dst.as_mut_ptr(), dst_v);
            })
    }
}

#[derive(Debug)]
pub struct Full;

#[derive(Debug)]
pub struct Interpolated;

#[derive(Debug)]
pub struct Flat;

#[derive(Debug)]
pub struct FlatCell;

impl BufferType for Full {
    const SIZE: usize = 16 * 16 * 384;
    const LEVEL: usize = 0;
    const Z_STRIDE: usize = 16;
    const Y_STRIDE: usize = <Self as BufferType>::Z_STRIDE * 16;

    fn unpack_coord(idx: usize) -> ChunkBlockPos {
        let x = idx as u8 & 0xF;
        let z = (idx >> 4) as u8 & 0xF;
        let y = (idx >> 8) as i16 - 64;
        ChunkBlockPos::new(x, y, z)
    }

    fn pack_coord(pos: ChunkBlockPos) -> usize {
        let x = pos.x() as usize;
        let z = pos.z() as usize;
        let y = (pos.y() + 64) as usize;

        (y << 8) | (z << 4) | x
    }
}

impl BufferType for Interpolated {
    const SIZE: usize = 8 * (4 * 4 * 96);
    const LEVEL: usize = 1;
    const Z_STRIDE: usize = 8 * 4;
    const Y_STRIDE: usize = <Self as BufferType>::Z_STRIDE * 4;

    fn unpack_coord(idx: usize) -> ChunkBlockPos {
        let corner_idx = idx & 0x7;
        let cell_idx = idx >> 3;

        let cell_x = (cell_idx as u8 & 0x3) * 4;
        let cell_z = ((cell_idx >> 2) as u8 & 0x3) * 4;
        let cell_y = ((cell_idx >> 4) as i16) * 4;

        ChunkBlockPos::new(
            cell_x + 3 * (corner_idx & 1) as u8,
            cell_y + 3 * (corner_idx >> 2 & 1) as i16 - 64,
            cell_z + 3 * (corner_idx >> 1 & 1) as u8,
        )
    }

    fn pack_coord(pos: ChunkBlockPos) -> usize {
        let cell_x = pos.x() as usize / 4;
        let cell_z = pos.z() as usize / 4;
        let cell_y = (pos.y() + 64) as usize / 4;

        let cell_idx = (cell_x & 0x4) | ((cell_z & 0x4) << 2) | (cell_y << 4);
        cell_idx << 3
    }
}

impl BufferApplyTo<Full> for Interpolated {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>, apply: F) {
        for cell_y in 0..(386 / 4) {
            let cell_base_y = cell_y * Self::Y_STRIDE;

            for cell_z in 0..4 {
                let cell_base_z = cell_base_y + cell_z * Self::Z_STRIDE;

                for cell_x in 0..4 {
                    let cell_base_x = cell_base_z + cell_x;

                    let Some(data): Option<[f32; 8]> = src
                        [(cell_base_x << 3)..((cell_base_x << 3) + 8)]
                        .as_array()
                        .copied()
                    else {
                        unreachable!()
                    };

                    for y in 0..4 {
                        let base_y = (y + cell_y * 4) * Full::Y_STRIDE;

                        for z in 0..4 {
                            let base_z = (z + cell_z * 4) * Full::Z_STRIDE + base_y;

                            for x in 0..4 {
                                let base_x = (x + cell_x * 4) + base_z;

                                let src = lerp3_f32(
                                    [x as f32 / 4.0, z as f32 / 4.0, y as f32 / 4.0],
                                    data,
                                );

                                dst[base_x] =
                                    apply.scalar(src, if F::READS_DST { dst[base_x] } else { 0.0 });
                            }
                        }
                    }
                }
            }
        }
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>, apply: F) {
        // SAFETY: requirements passed to caller
        unsafe {
            let x = x86_64::_mm256_setr_ps(0.0, 0.25, 0.5, 0.75, 0.0, 0.25, 0.5, 0.75);

            let z = [
                x86_64::_mm256_setr_ps(0.0, 0.0, 0.0, 0.0, 0.25, 0.25, 0.25, 0.25),
                x86_64::_mm256_setr_ps(0.5, 0.5, 0.5, 0.5, 0.75, 0.75, 0.75, 0.75),
            ];

            let mut src_cell_idx = 0;
            for cell_y in 0..(384 / 4) {
                let dst_base_y = cell_y * 4 * Full::Y_STRIDE;

                for cell_z in 0..(16 / 4) {
                    let dst_base_z = dst_base_y + cell_z * 4 * Full::Z_STRIDE;

                    for cell_x in 0..(16 / 4) {
                        let dst_base_x = dst_base_z + cell_x * 4;

                        let src_data = x86_64::_mm256_load_ps(&raw const src[src_cell_idx * 8]);
                        src_cell_idx += 1;

                        for i in 0..8usize {
                            let z_idx = i & 1;
                            let y_idx = i >> 1;
                            let y = x86_64::_mm256_set1_ps(y_idx as f32 / 4.0);

                            let z_offset = z_idx * 2 * Full::Z_STRIDE;
                            let y_offset = y_idx * Full::Y_STRIDE;
                            let dst_offset = dst_base_x + z_offset + y_offset;

                            let interpolated = lerp3_f32_simd([x, z[z_idx], y], src_data);

                            let dst_data = apply.simd(
                                interpolated,
                                if F::READS_DST {
                                    x86_64::_mm256_set_m128(
                                        x86_64::_mm_load_ps(
                                            &raw const dst[dst_offset + Full::Z_STRIDE],
                                        ),
                                        x86_64::_mm_load_ps(&raw const dst[dst_offset]),
                                    )
                                } else {
                                    x86_64::_mm256_setzero_ps()
                                },
                            );

                            let lo = x86_64::_mm256_castps256_ps128(dst_data);
                            let hi = x86_64::_mm256_extractf128_ps::<1>(dst_data);

                            x86_64::_mm_store_ps(&raw mut dst[dst_offset], lo);
                            x86_64::_mm_store_ps(&raw mut dst[dst_offset + Full::Z_STRIDE], hi)
                        }
                    }
                }
            }
        }
    }
}

impl BufferType for Flat {
    const SIZE: usize = 16 * 16;
    const LEVEL: usize = 2;
    const Z_STRIDE: usize = 16;
    const Y_STRIDE: usize = 0;

    fn unpack_coord(idx: usize) -> ChunkBlockPos {
        let x = idx as u8 & 0xF;
        let z = (idx >> 4) as u8 & 0xF;
        ChunkBlockPos::new(x, 0, z)
    }

    fn pack_coord(pos: ChunkBlockPos) -> usize {
        let x = pos.x() as usize;
        let z = pos.z() as usize;

        (z << 4) | x
    }
}

impl BufferApplyTo<Full> for Flat {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>, apply: F) {
        for z in 0..16 {
            let base_z = z * Self::Z_STRIDE;

            for x in 0..16 {
                let base_x = base_z + x;

                let val = src[base_x];

                for y in 0..384 {
                    let base_y = y * Full::Y_STRIDE + base_x;

                    dst[base_y] = apply.scalar(val, if F::READS_DST { dst[base_y] } else { 0.0 });
                }
            }
        }
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>, apply: F) {
        for y in 0..384 {
            let base_y = y * Full::Y_STRIDE;

            for z in 0..16 {
                let base_z = z * Self::Z_STRIDE;
                let base_dst = base_y + base_z;

                // SAFETY: requirements passed to caller
                unsafe {
                    let src0 = x86_64::_mm256_load_ps(&raw const src[base_z]);
                    let src1 = x86_64::_mm256_load_ps(&raw const src[base_z + 0x8]);

                    let dst0 = apply.simd(
                        src0,
                        if F::READS_DST {
                            x86_64::_mm256_load_ps(&raw const dst[base_dst])
                        } else {
                            x86_64::_mm256_setzero_ps()
                        },
                    );

                    let dst1 = apply.simd(
                        src1,
                        if F::READS_DST {
                            x86_64::_mm256_load_ps(&raw const dst[base_dst + 0x8])
                        } else {
                            x86_64::_mm256_setzero_ps()
                        },
                    );

                    x86_64::_mm256_store_ps(&raw mut dst[base_dst], dst0);
                    x86_64::_mm256_store_ps(&raw mut dst[base_dst + 0x8], dst1);
                }
            }
        }
    }
}

impl BufferApplyTo<Interpolated> for Flat {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Interpolated>, apply: F) {
        for z in 0..16 {
            let base_z = z * Self::Z_STRIDE;
            let cell_base_z = (z / 4) * Interpolated::Z_STRIDE;

            for x in 0..16 {
                let base_x = base_z + x;
                let cell_base_x = cell_base_z + (x / 4);

                let val = src[base_x];

                for y in 0..(384 / 4) {
                    let cell_base_y = y * Interpolated::Y_STRIDE + cell_base_x;

                    dst[cell_base_y << 3] = apply.scalar(
                        val,
                        if F::READS_DST {
                            dst[cell_base_y << 3]
                        } else {
                            0.0
                        },
                    );
                }
            }
        }
    }

    unsafe fn apply_to_simd<F: BufferOperation>(
        src: &Buffer<Self>,
        dst: &mut Buffer<Interpolated>,
        apply: F,
    ) {
        // SAFETY: requirements passed to caller
        unsafe {
            let src_offsets = x86_64::_mm256_setr_epi32(
                0,
                0x3,
                0x3 * Self::Z_STRIDE as i32,
                0x3 * Self::Z_STRIDE as i32 + 0x3,
                0,
                0x3,
                0x3 * Self::Z_STRIDE as i32,
                0x3 * Self::Z_STRIDE as i32 + 0x3,
            );

            for z in 0..4 {
                let src_base_z = z * 4 * Self::Z_STRIDE;
                let dst_base_z = z * 4;

                for x in 0..4 {
                    let src_base_x = src_base_z + x * 4;
                    let dst_base_x = dst_base_z + x;

                    let src_data =
                        x86_64::_mm256_i32gather_ps::<4>(&raw const src[src_base_x], src_offsets);

                    for y in 0..(384 / 4) {
                        let dst_base_y = dst_base_x + y * 16;

                        let dst_data = apply.simd(
                            src_data,
                            if F::READS_DST {
                                x86_64::_mm256_load_ps(&raw const dst[dst_base_y << 3])
                            } else {
                                x86_64::_mm256_setzero_ps()
                            },
                        );

                        x86_64::_mm256_store_ps(&raw mut dst[dst_base_y << 3], dst_data);
                    }
                }
            }
        }
    }
}

impl BufferType for FlatCell {
    const SIZE: usize = 4 * 4;
    const LEVEL: usize = 3;
    const Z_STRIDE: usize = 4;
    const Y_STRIDE: usize = 0;

    fn unpack_coord(idx: usize) -> ChunkBlockPos {
        let x = (idx as u8 & 0x3) * 4;
        let z = ((idx >> 2) as u8 & 0x3) * 4;
        ChunkBlockPos::new(x, 0, z)
    }

    fn pack_coord(_pos: ChunkBlockPos) -> usize {
        todo!()
    }
}

impl BufferApplyTo<Full> for FlatCell {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>, apply: F) {
        for cell_z in 0..4 {
            let src_base_z = cell_z * Self::Z_STRIDE;

            for cell_x in 0..4 {
                let src_base_x = src_base_z + cell_x;

                let val = src[src_base_x];

                for y in 0..384 {
                    let base_y = y * Full::Y_STRIDE;

                    for z in 0..4 {
                        let base_z = base_y + (cell_z * 4 + z) * Full::Z_STRIDE;

                        for x in 0..4 {
                            let base_x = base_z + (cell_x * 4 + x);

                            dst[base_x] =
                                apply.scalar(val, if F::READS_DST { dst[base_x] } else { 0.0 });
                        }
                    }
                }
            }
        }
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>, apply: F) {
        // SAFETY: requirements passed to caller
        unsafe {
            for cell_z in 0..4 {
                let src_base_z = cell_z * Self::Z_STRIDE;

                for cell_x in 0..2 {
                    let src_base_x = src_base_z + (cell_x * 2);
                    let dst_base_x = cell_x * 8;

                    let val = x86_64::_mm256_setr_m128(
                        x86_64::_mm_set1_ps(src[src_base_x]),
                        x86_64::_mm_set1_ps(src[src_base_x + 1]),
                    );

                    for y in 0..384 {
                        let base_y = y * Full::Y_STRIDE;

                        for z in 0..4 {
                            let base_x = base_y + (cell_z * 4 + z) * 4 + dst_base_x;

                            let dst_val = apply.simd(
                                val,
                                if F::READS_DST {
                                    x86_64::_mm256_load_ps(&raw const dst[base_x])
                                } else {
                                    x86_64::_mm256_setzero_ps()
                                },
                            );

                            x86_64::_mm256_store_ps(&raw mut dst[base_x], dst_val);
                        }
                    }
                }
            }
        }
    }
}

impl BufferApplyTo<Interpolated> for FlatCell {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Interpolated>, apply: F) {
        for cell_z in 0..4 {
            let src_base_z = cell_z * Self::Z_STRIDE;
            let dst_base_z = cell_z * 4;

            for cell_x in 0..4 {
                let src_base_x = src_base_z + cell_x;
                let dst_base_x = dst_base_z + cell_x;

                let val = src[src_base_x];

                for y in 0..(384 / 4) {
                    let base_y = (y * 16) + dst_base_x;

                    for i in 0..8 {
                        dst[(base_y << 3) + i] = apply.scalar(
                            val,
                            if F::READS_DST {
                                dst[(base_y << 3) + i]
                            } else {
                                0.0
                            },
                        )
                    }
                }
            }
        }
    }

    unsafe fn apply_to_simd<F: BufferOperation>(
        src: &Buffer<Self>,
        dst: &mut Buffer<Interpolated>,
        apply: F,
    ) {
        // SAFETY: requirements passed to caller
        unsafe {
            for z in 0..4 {
                let cell_base_z = z * 4;

                for x in 0..4 {
                    let cell_base_x = cell_base_z + x;

                    let val = x86_64::_mm256_set1_ps(src[cell_base_x]);

                    for y in 0..(384 / 4) {
                        let dst_base_y = cell_base_x + y * 16;

                        let dst_data = apply.simd(
                            val,
                            if F::READS_DST {
                                x86_64::_mm256_load_ps(&raw const dst[dst_base_y << 3])
                            } else {
                                x86_64::_mm256_setzero_ps()
                            },
                        );

                        x86_64::_mm256_store_ps(&raw mut dst[dst_base_y << 3], dst_data);
                    }
                }
            }
        }
    }
}

impl BufferApplyTo<Flat> for FlatCell {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Flat>, apply: F) {
        dst.iter_mut().enumerate().for_each(|(i, val)| {
            let x = (i >> 2) & 0x3;
            let z = (i >> 6) & 0x3;

            let i = (z << 2) | x;

            *val = apply.scalar(src[i], if F::READS_DST { *val } else { 0.0 });
        })
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Flat>, apply: F) {
        // SAFETY: requirements passed to caller
        unsafe {
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

                    let dst_v = apply.simd(
                        src_v,
                        if F::READS_DST {
                            x86_64::_mm256_load_ps(&raw const dst[(z << 4) | x])
                        } else {
                            x86_64::_mm256_setzero_ps()
                        },
                    );

                    x86_64::_mm256_store_ps(&raw mut dst[(z << 4) | x], dst_v);
                })
        }
    }
}
