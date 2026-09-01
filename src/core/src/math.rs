use std::arch::x86_64;

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

#[inline(always)]
pub fn lerp_f32(a: f32, p: [f32; 2]) -> f32 {
    p[0] + a * (p[1] - p[0])
}

#[inline(always)]
pub fn lerp2_f32(a: [f32; 2], p: [f32; 4]) -> f32 {
    lerp_f32(a[1], [lerp_f32(a[0], [p[0], p[1]]), lerp_f32(a[0], [p[2], p[3]])])
}

#[inline(always)]
pub fn lerp3_f32(a: [f32; 3], p: [f32; 8]) -> f32 {
    lerp_f32(
        a[2],
        [
            lerp2_f32([a[0], a[1]], [p[0], p[1], p[2], p[3]]),
            lerp2_f32([a[0], a[1]], [p[4], p[5], p[6], p[7]]),
        ],
    )
}

#[target_feature(enable = "avx2")]
fn _mm256_fmadd_ps_fallback(a: x86_64::__m256, b: x86_64::__m256, c: x86_64::__m256) -> x86_64::__m256 {
    if is_x86_feature_detected!("fma") {
        // SAFETY: required features are detected at this point
        unsafe {
            x86_64::_mm256_fmadd_ps(
                a,
                b,
                c
            )
        }
    } else {
        x86_64::_mm256_add_ps(
            x86_64::_mm256_mul_ps(
                a,
                b,
            ),
            c,
        )
    }
}

#[target_feature(enable = "avx2")]
pub fn lerp_f32_simd(a: x86_64::__m256, p: [f32; 2]) -> x86_64::__m256 {
    let p = [
        x86_64::_mm256_set1_ps(p[0]),
        x86_64::_mm256_set1_ps(p[1]),
    ];

    _mm256_fmadd_ps_fallback(
        a,
        x86_64::_mm256_sub_ps(p[1], p[0]),
        p[0]
    )
}

#[target_feature(enable = "avx2")]
pub fn lerp3_f32_simd(a: [x86_64::__m256; 3], p: [f32; 8]) -> x86_64::__m256 {
    let x00 = lerp_f32_simd(a[0], [p[0], p[1]]);
    let x10 = lerp_f32_simd(a[0], [p[2], p[3]]);
    let x01 = lerp_f32_simd(a[0], [p[4], p[5]]);
    let x11 = lerp_f32_simd(a[0], [p[6], p[7]]);

    let y0 = _mm256_fmadd_ps_fallback(
        a[1],
        x86_64::_mm256_sub_ps(x10, x00),
        x00,
    );
    let y1 = _mm256_fmadd_ps_fallback(
        a[1],
        x86_64::_mm256_sub_ps(x11, x01),
        x01,
    );

    _mm256_fmadd_ps_fallback(
        a[2],
        x86_64::_mm256_sub_ps(y1, y0),
        y0,
    )
}
