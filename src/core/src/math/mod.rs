mod float;
mod double;

/// A trait to provide math extensions to various primitives.
pub trait TemperMathExt {
    fn smooth_step(self) -> Self;

    fn lerp(self, p0: Self, p1: Self) -> Self;

    #[expect(clippy::too_many_arguments)]
    fn lerp2(
        t0: Self,
        t1: Self,
        p00: Self,
        p01: Self,
        p10: Self,
        p11: Self
    ) -> Self;

    #[expect(clippy::too_many_arguments)]
    fn lerp3(
        t0: Self,
        t1: Self,
        t2: Self,
        p000: Self,
        p001: Self,
        p010: Self,
        p011: Self,
        p100: Self,
        p101: Self,
        p110: Self,
        p111: Self,
    ) -> Self;
}

/// A trait to provide math extensions to various unsafe primitives (like SIMD
/// primitives).
pub trait TemperMathExtUnsafe {
    unsafe fn square(self) -> Self;

    unsafe fn cube(self) -> Self;

    unsafe fn inverse(self) -> Self;

    unsafe fn smooth_step(self) -> Self;

    unsafe fn lerp(self, p0: Self, p1: Self) -> Self;

    #[expect(clippy::too_many_arguments)]
    unsafe fn lerp2(
        t0: Self,
        t1: Self,
        p00: Self,
        p01: Self,
        p10: Self,
        p11: Self
    ) -> Self;

    #[expect(clippy::too_many_arguments)]
    unsafe fn lerp3(
        t0: Self,
        t1: Self,
        t2: Self,
        p000: Self,
        p001: Self,
        p010: Self,
        p011: Self,
        p100: Self,
        p101: Self,
        p110: Self,
        p111: Self,
    ) -> Self;
}
