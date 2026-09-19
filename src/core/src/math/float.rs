use crate::math::TemperMathExt;

impl TemperMathExt for f32 {
    fn smooth_step(self) -> Self {
        self * self * self * (self * (self * 6.0 - 15.0) + 10.0)
    }

    fn lerp(self, p0: Self, p1: Self) -> Self {
        p0 + self * (p1 - p0)
    }

    fn clamped_lerp(self, p0: Self, p1: Self) -> Self {
        self.clamp(0.0, 1.0).lerp(p0, p1)
    }

    fn inverse_lerp(self, p0: Self, p1: Self) -> Self {
        (self - p0) / (p1 - p0)
    }

    fn lerp2(t0: Self, t1: Self, p00: Self, p01: Self, p10: Self, p11: Self) -> Self {
        t1.lerp(t0.lerp(p00, p01), t0.lerp(p10, p11))
    }

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
    ) -> Self {
        t2.lerp(
            f32::lerp2(t0, t1, p000, p001, p010, p011),
            f32::lerp2(t0, t1, p100, p101, p110, p111),
        )
    }
}
