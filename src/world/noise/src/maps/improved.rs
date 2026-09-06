use crate::maps::GRADIENT;
use bevy_math::{DVec3, IVec3};
use temper_core::math::TemperMathExt;
use temper_core::random::RandomSource;

#[derive(Clone)]
pub struct ImprovedNoise {
    p: [u8; 256],
    pos: DVec3,
}

impl ImprovedNoise {
    pub fn new<T: RandomSource>(rand: &mut T) -> ImprovedNoise {
        let xo = rand.next_f64() * 256.0;
        let yo = rand.next_f64() * 256.0;
        let zo = rand.next_f64() * 256.0;

        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);
        for i in 0..256 {
            let offset = rand.next_u32_bounded(256 - i as u32) as usize;
            p.swap(i, i + offset);
        }

        Self {
            p,
            pos: DVec3::new(xo, yo, zo),
        }
    }

    fn p(&self, x: i32) -> i32 {
        self.p[(x & 0xFF) as usize] as i32
    }

    fn grad_dot(hash: i32, pos: DVec3) -> f64 {
        GRADIENT[(hash & 0xF) as usize].dot(pos)
    }

    #[inline(always)]
    pub fn noise(&self, pos: DVec3) -> f64 {
        self.noise_advanced(pos, 0.0, 0.0)
    }

    pub fn noise_advanced(&self, pos: DVec3, y_scale: f64, y_fudge: f64) -> f64 {
        let pos = self.pos + pos;
        let pos_f = pos.floor();
        let pos_r = pos - pos_f;

        let yr_fudge = if y_scale != 0.0 {
            let limit = if y_fudge >= 0.0 && y_fudge < pos_r.y {
                y_fudge
            } else {
                pos_r.y
            };

            (limit / y_scale + 1.0e-7f32 as f64).floor() * y_scale
        } else {
            0.0
        };

        self.sample_and_lerp(
            pos_f.as_ivec3(),
            DVec3::new(pos_r.x, pos_r.y - yr_fudge, pos_r.z),
            pos_r.y,
        )
    }

    fn sample_and_lerp(&self, pos: IVec3, pos_r: DVec3, yr_original: f64) -> f64 {
        const OFFSETS: [DVec3; 8] = [
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(0.0, 1.0, 0.0),
            DVec3::new(1.0, 1.0, 0.0),
            DVec3::new(0.0, 0.0, 1.0),
            DVec3::new(1.0, 0.0, 1.0),
            DVec3::new(0.0, 1.0, 1.0),
            DVec3::new(1.0, 1.0, 1.0),
        ];

        let x: [i32; 2] = std::array::from_fn(|i| self.p(pos.x + i as i32));
        let xy: [i32; 4] = std::array::from_fn(|i| self.p(x[i % 2] + pos.y + (i / 2) as i32));
        let [d000, d001, d010, d011, d100, d101, d110, d111] = std::array::from_fn(|i| {
            Self::grad_dot(
                self.p(xy[i % 4] + pos.z + (i / 4) as i32),
                pos_r - OFFSETS[i],
            )
        });

        f64::lerp3(
            pos_r.x.smooth_step(),
            yr_original.smooth_step(),
            pos_r.z.smooth_step(),
            d000,
            d001,
            d010,
            d011,
            d100,
            d101,
            d110,
            d111,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps::tests::data::IMPROVED_TEST;
    use crate::maps::tests::run_test;

    #[test]
    fn test_improved_noise() {
        run_test(
            &IMPROVED_TEST,
            |rand, _| ImprovedNoise::new(rand),
            ImprovedNoise::noise,
        )
    }
}
