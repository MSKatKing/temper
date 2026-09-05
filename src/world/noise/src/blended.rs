use crate::PerlinNoise;
use bevy_math::DVec3;
use std::fmt::{Debug, Formatter};
use temper_core::math::clamped_lerp;
use temper_core::pos::BlockPos;
use temper_core::random::XoroshiroRandomSource;

#[derive(Clone)]
pub struct BlendedNoise {
    min_limit_noise: PerlinNoise,
    max_limit_noise: PerlinNoise,
    main_noise: PerlinNoise,
    xz_multiplier: f64,
    y_multiplier: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
}

impl BlendedNoise {
    pub fn new_unseeded(
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Self {
        let mut rand = XoroshiroRandomSource::new(0);

        Self {
            min_limit_noise: PerlinNoise::new_legacy(
                &mut rand,
                &(-15..=0).into_iter().collect::<Vec<_>>(),
            ),
            max_limit_noise: PerlinNoise::new_legacy(
                &mut rand,
                &(-15..=0).into_iter().collect::<Vec<_>>(),
            ),
            main_noise: PerlinNoise::new_legacy(
                &mut rand,
                &(-7..=0).into_iter().collect::<Vec<_>>(),
            ),
            xz_factor,
            y_factor,
            smear_scale_multiplier,
            xz_multiplier: 684.412 * xz_scale,
            y_multiplier: 684.412 * y_scale,
        }
    }

    pub fn noise(&self, pos: BlockPos) -> f64 {
        let limit_x = pos.pos.x as f64 * self.xz_multiplier;
        let limit_y = pos.pos.y as f64 * self.y_multiplier;
        let limit_z = pos.pos.z as f64 * self.xz_multiplier;
        let limit = DVec3::new(limit_x, limit_y, limit_z);

        let main_x = limit_x / self.xz_factor;
        let main_y = limit_y / self.y_factor;
        let main_z = limit_z / self.xz_factor;
        let main = DVec3::new(main_x, main_y, main_z);

        let limit_smear = self.y_multiplier * self.smear_scale_multiplier;
        let main_smear = limit_smear / self.y_factor;
        let mut blend_min = 0.0;
        let mut blend_max = 0.0;
        let mut main_noise_value = 0.0;
        let mut pow = 1.0;

        for i in 0..8 {
            if let Some((noise, _)) = &self.main_noise.get_octave_noise(i) {
                main_noise_value += noise.noise_advanced(
                    main.map(|v| PerlinNoise::wrap(v * pow)),
                    main_smear * pow,
                    main_y * pow,
                ) / pow;
            }

            pow /= 2.0;
        }

        let factor = (main_noise_value / 10.0 + 1.0) / 2.0;
        let is_max = factor >= 1.0;
        let is_min = factor <= 0.0;
        pow = 1.0;

        for i in 0..16 {
            let w = limit.map(|v| PerlinNoise::wrap(v * pow));
            let y_scale_pow = limit_smear * pow;

            if !is_max && let Some((noise, _)) = self.min_limit_noise.get_octave_noise(i) {
                blend_min += noise.noise_advanced(w, y_scale_pow, limit_y * pow) / pow;
            }

            if !is_min && let Some((noise, _)) = self.max_limit_noise.get_octave_noise(i) {
                blend_max += noise.noise_advanced(w, y_scale_pow, limit_y * pow) / pow;
            }

            pow /= 2.0;
        }

        clamped_lerp(
            factor,
            blend_min / 512.0,
            blend_max / 512.0,
        ) / 128.0
    }
}

impl Debug for BlendedNoise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlendedNoise {{}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{NoiseMapTest, run_test};

    const BLENDED_TESTS: NoiseMapTest<(f64, f64, f64, f64, f64)> = &[((0x0, (1.0, 1.0, 0.25, 0.25, 8.0)), &[([245.0, 46.0, 38.0], -0.12654345658295169), ([79.0, 224.0, 252.0], 0.05858189062496527), ([20.0, 84.0, 167.0], 0.15598663909079818), ([92.0, 148.0, 7.0], 0.607647860818965), ([206.0, 111.0, 227.0], -0.09847710750813403), ([204.0, 42.0, 166.0], -0.10478960713988411), ([215.0, 69.0, 19.0], -0.20775163473244568), ([100.0, 250.0, 48.0], 0.13620409227923413), ([7.0, 32.0, 115.0], 0.002356496032795155), ([91.0, 113.0, 139.0], -0.34137862480868314), ([231.0, 53.0, 192.0], -0.05367432435298384), ([88.0, 84.0, 137.0], -0.05623176127529331), ([91.0, 7.0, 217.0], -0.014628661720083841), ([110.0, 243.0, 228.0], 0.18394520291329408), ([40.0, 144.0, 139.0], -0.12857364932438692), ([172.0, 72.0, 69.0], -0.030149820264244117), ([189.0, 123.0, 66.0], -0.0284654558195548), ([222.0, 9.0, 192.0], -0.14066521314952846), ([13.0, 198.0, 92.0], -0.03983141528692724), ([2.0, 210.0, 101.0], 0.2429129621236074), ]),];


    #[test]
    pub fn test_blended_noise() {
        run_test(
            &BLENDED_TESTS,
            |_, (xz_scale, y_scale, xz_factor, y_factor, smear_scale_multiplier)| {
                BlendedNoise::new_unseeded(*xz_scale, *y_scale, *xz_factor, *y_factor, *smear_scale_multiplier)
            },
            |noise, pos| noise.noise(BlockPos::of(pos.x as i32, pos.y as i32, pos.z as i32)),
        )
    }
}
