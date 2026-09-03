use temper_core::pos::ChunkBlockPos;
use crate::cpu::buffer::Buffer;
use std::arch::x86_64;
use std::fmt::Debug;
use crate::cpu::buffer::op::BufferOperation;

pub trait BufferType: Sized + Debug {
    const SIZE: usize;
    const LEVEL: usize;
    const Z_STRIDE: usize;
    const Y_STRIDE: usize;

    fn unpack_coord(idx: usize) -> ChunkBlockPos;
    fn pack_coord(pos: ChunkBlockPos) -> usize;
    
    /// Performs the BufferOperation on self (`src` and `dst` are both values from self).
    fn apply_to_self<T: BufferOperation>(this: &mut Buffer<Self>) {
        this
            .iter_mut()
            .for_each(|v| *v = T::scalar(*v, *v))
    }

    /// Performs the BufferOperation on self (`src` and `dst` are both values from self).
    /// 
    /// # Safety
    /// The caller must ensure that the system supports the avx2 feature set.
    #[target_feature(enable = "avx2")]
    unsafe fn apply_to_self_simd<T: BufferOperation>(this: &mut Buffer<Self>) {
        this
            .as_chunks_mut::<8>()
            .0
            .iter_mut()
            .for_each(|v| unsafe {
                let src = x86_64::_mm256_load_ps(v.as_ptr());
                x86_64::_mm256_store_ps(v.as_mut_ptr(), T::simd(src, src))
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
    fn apply_to<F: BufferOperation>(
        src: &Buffer<Self>,
        dst: &mut Buffer<Dst>,
    );

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
    unsafe fn apply_to_simd<F: BufferOperation>(
        src: &Buffer<Self>,
        dst: &mut Buffer<Dst>,
    );
}

impl<Dst: BufferType> BufferApplyTo<Dst> for Dst {
    fn apply_to<F: BufferOperation>(
        src: &Buffer<Self>,
        dst: &mut Buffer<Dst>,
    ) {
        dst
            .iter_mut()
            .zip(src.iter())
            .for_each(|(dst, src)| {
                *dst = F::scalar(*src, if F::READS_DST { *dst } else { 0.0 })
            });
    }

    unsafe fn apply_to_simd<F: BufferOperation>(
        src: &Buffer<Self>,
        dst: &mut Buffer<Dst>,
    ) {
        dst
            .as_chunks_mut::<8>()
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

                let dst_v = F::simd(src_v, dst_v);

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
    const LEVEL: usize = 0;
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

    // TODO: pack cell block offset as well
    fn pack_coord(pos: ChunkBlockPos) -> usize {
        let cell_x = pos.x() as usize / 4;
        let cell_z = pos.z() as usize / 4;
        let cell_y = (pos.y() + 64) as usize / 4;

        let cell_idx = (cell_x & 0x4) | ((cell_z & 0x4) << 2) | (cell_y << 4);
        cell_idx << 3
    }
}

impl BufferApplyTo<Full> for Interpolated {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>) {
        todo!()
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>) {
        todo!()
    }
}

impl BufferType for Flat {
    const SIZE: usize = 16 * 16;
    const LEVEL: usize = 0;
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
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>) {
        todo!()
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>) {
        todo!()
    }
}

impl BufferApplyTo<Interpolated> for Flat {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Interpolated>) {
        todo!()
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Interpolated>) {
        todo!()
    }
}

impl BufferType for FlatCell {
    const SIZE: usize = 4 * 4;
    const LEVEL: usize = 0;
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
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>) {
        todo!()
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Full>) {
        todo!()
    }
}

impl BufferApplyTo<Interpolated> for FlatCell {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Interpolated>) {
        todo!()
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Interpolated>) {
        todo!()
    }
}

impl BufferApplyTo<Flat> for FlatCell {
    fn apply_to<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Flat>) {
        todo!()
    }

    unsafe fn apply_to_simd<F: BufferOperation>(src: &Buffer<Self>, dst: &mut Buffer<Flat>) {
        todo!()
    }
}
