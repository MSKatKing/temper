use std::arch::x86_64;
use std::arch::x86_64::__m256;

/// Represents an operation that can occur on a buffer source and destination value.
pub trait BufferOperation {
    /// Whether the operation reads from DST or not. Used to hint the compiler to remove
    /// destination loads from functions if not used. Mainly for the SIMD variant of this operation.
    /// If this value is false, the data fed into `dst` arguments in the functions will be garbage
    /// and must not be used.
    const READS_DST: bool;

    /// Defines this buffer operation in terms of two scalar values. These two values will map to
    /// the same index in the destination buffer.
    ///
    /// # Arguments
    ///  * `src`: the value from the source buffer.
    ///  * `dst`: the value from the destination buffer.
    ///
    /// # Returns
    /// A value that will be stored in the destination buffer.
    fn scalar(&self, src: f32, dst: f32) -> f32;

    /// Defines this buffer operation in terms of two SIMD values. The order of the values within
    /// the SIMD values does not matter, but they should map to the same index in the destination
    /// buffer (for instance, `src.a` should map to `dst.b` but `src.a` may map to a higher index
    /// than `src.b`).
    ///
    /// # Safety
    /// The caller must ensure that the processor supports the avx2 feature set. If not, the
    /// processor may throw an exception or undefined behavior will occur.
    ///
    /// # Arguments
    ///  * `src`: the value from the source buffer.
    ///  * `dst`: the value from the destination buffer.
    ///
    /// # Returns
    /// A value that will be stored in the destination buffer.
    #[cfg(target_arch = "x86_64")]
    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256;
}

/// Performs `dst = src`.
pub struct Replace;

/// Performs `dst = src + dst`.
pub struct Add;

/// Performs `dst = dst - src`.
pub struct Sub;

/// Performs `dst = src * dst`.
pub struct Mul;

/// Performs `dst = dst / src`.
pub struct Div;

/// Performs `dst = min(src, dst)`.
pub struct Min;

/// Performs `dst = max(src, dst)`.
pub struct Max;

impl BufferOperation for Replace {
    const READS_DST: bool = false;

    fn scalar(&self, src: f32, _: f32) -> f32 {
        src
    }

    unsafe fn simd(&self, src: __m256, _: __m256) -> __m256 {
        src
    }
}

impl BufferOperation for Add {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        src + dst
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe { x86_64::_mm256_add_ps(src, dst) }
    }
}

impl BufferOperation for Sub {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        dst - src
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe { x86_64::_mm256_sub_ps(dst, src) }
    }
}

impl BufferOperation for Mul {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        src * dst
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe { x86_64::_mm256_mul_ps(src, dst) }
    }
}

impl BufferOperation for Div {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        dst / src
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe { x86_64::_mm256_div_ps(dst, src) }
    }
}

impl BufferOperation for Min {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        src.min(dst)
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe { x86_64::_mm256_min_ps(src, dst) }
    }
}

impl BufferOperation for Max {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        src.max(dst)
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe { x86_64::_mm256_max_ps(src, dst) }
    }
}
