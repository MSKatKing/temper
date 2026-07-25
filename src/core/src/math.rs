#[inline(always)]
pub fn smooth_step(x: f64) -> f64 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
pub fn lerp(a: f64, p: [f64; 2]) -> f64 {
    p[0] + a * (p[1] - p[0])
}

#[inline(always)]
pub fn lerp2(a: [f64; 2], p: [f64; 4]) -> f64 {
    lerp(a[1], [lerp(a[0], [p[0], p[1]]), lerp(a[0], [p[2], p[3]])])
}

#[inline(always)]
pub fn lerp3(a: [f64; 3], p: [f64; 8]) -> f64 {
    lerp(
        a[2],
        [
            lerp2([a[0], a[1]], [p[0], p[1], p[2], p[3]]),
            lerp2([a[0], a[1]], [p[4], p[5], p[6], p[7]]),
        ],
    )
}
