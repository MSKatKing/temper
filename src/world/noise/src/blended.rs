use crate::PerlinNoise;
use bevy_math::DVec3;
use std::fmt::{Debug, Formatter};
use temper_core::math::lerp;
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
                main_noise_value = noise.noise_advanced(
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

        lerp(
            factor.clamp(0.0, 1.0),
            [blend_min / 512.0, blend_max / 512.0],
        ) / 128.0
    }
}

impl Debug for BlendedNoise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlendedNoise {{}}")
    }
}
