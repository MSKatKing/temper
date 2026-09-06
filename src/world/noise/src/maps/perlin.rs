use crate::ImprovedNoise;
use bevy_math::DVec3;
use std::ops::Mul;
use temper_core::random::{PositionalRandom, RandomSource};

#[derive(Clone)]
pub struct PerlinNoise {
    noise_levels: Box<[Option<(ImprovedNoise, f64)>]>,
    lowest_freq_value_factor: f64,
    lowest_freq_input_factor: f64,
}

impl PerlinNoise {
    pub fn new<R: RandomSource>(
        rand: &mut R,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> PerlinNoise {
        let octaves = amplitudes.len();

        let mut noise_levels = Vec::with_capacity(octaves);
        let positional = rand.fork_positional();
        for (i, amp) in amplitudes.iter().enumerate() {
            noise_levels.push(if *amp != 0.0 {
                let octave = first_octave + i as i32;
                let mut rand = positional.spawn_from_hash(format!("octave_{}", octave));
                Some((ImprovedNoise::new(&mut rand), amplitudes[i]))
            } else {
                None
            })
        }

        let lowest_freq_input_factor = 2f64.powi(first_octave);
        let lowest_freq_value_factor =
            2f64.powi(octaves as i32 - 1) / (2f64.powi(octaves as i32) - 1.0);

        Self {
            noise_levels: noise_levels.into_boxed_slice(),
            lowest_freq_value_factor,
            lowest_freq_input_factor,
        }
    }

    pub fn new_legacy<R: RandomSource>(rand: &mut R, octaves: &[i32]) -> PerlinNoise {
        debug_assert!(!octaves.is_empty());

        let low_freq_octaves = -octaves[0];
        let high_freq_octaves = octaves[octaves.len() - 1];
        let octave_range = low_freq_octaves + high_freq_octaves + 1;
        debug_assert!(octave_range >= 1);

        let mut amplitudes = vec![0.0; octave_range as usize];
        for octave in octaves {
            amplitudes[(octave + low_freq_octaves) as usize] = 1.0;
        }

        let first_octave = octaves[0];
        let octaves = amplitudes.len() as i32;
        let zero_octave_index = -first_octave;
        let mut noise_levels = vec![None; amplitudes.len()];

        let zero_octave = ImprovedNoise::new(rand);
        if zero_octave_index >= 0 && zero_octave_index < octaves {
            let zero_octave_amplitude = amplitudes[zero_octave_index as usize];
            if zero_octave_amplitude != 0.0 {
                noise_levels[zero_octave_index as usize] =
                    Some((zero_octave, zero_octave_amplitude));
            }
        }

        for i in (0..zero_octave_index).rev() {
            if i < octaves {
                let amplitude = amplitudes[i as usize];
                if amplitude != 0.0 {
                    noise_levels[i as usize] = Some((ImprovedNoise::new(rand), amplitude));
                } else {
                    rand.consume_count(262)
                }
            } else {
                rand.consume_count(262)
            }
        }

        debug_assert_eq!(
            noise_levels.iter().filter(|v| v.is_some()).count(),
            amplitudes.iter().filter(|&&v| v != 0.0).count(),
        );

        let lowest_freq_input_factor = 2f64.powi(-zero_octave_index);
        let lowest_freq_value_factor = 2f64.powi(octaves - 1) / (2f64.powi(octaves) - 1.0);

        Self {
            lowest_freq_input_factor,
            lowest_freq_value_factor,
            noise_levels: noise_levels.into_boxed_slice(),
        }
    }

    pub fn get_octave_noise(&self, octave: usize) -> Option<&(ImprovedNoise, f64)> {
        self.noise_levels[self.noise_levels.len() - 1 - octave].as_ref()
    }

    pub(crate) fn wrap(x: f64) -> f64 {
        x - (x / 3.3554432E7 + 0.5).floor() * 3.3554432E7
    }

    #[inline(always)]
    pub fn noise(&self, pos: DVec3) -> f64 {
        self.noise_advanced(pos, 0.0, 0.0)
    }

    pub fn noise_advanced(&self, pos: DVec3, y_scale: f64, y_fudge: f64) -> f64 {
        let mut value = 0.0;
        let mut factor = self.lowest_freq_input_factor;
        let mut value_factor = self.lowest_freq_value_factor;

        for level in self.noise_levels.iter() {
            if let Some((noise, amp)) = level.as_ref() {
                let val = noise.noise_advanced(
                    pos.mul(factor).map(Self::wrap),
                    y_scale * factor,
                    y_fudge * factor,
                );
                value += *amp * val * value_factor;
            }

            factor *= 2.0;
            value_factor /= 2.0;
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps::tests::data::{PERLIN_LEGACY_TEST, PERLIN_TEST};
    use crate::maps::tests::run_test;

    #[test]
    fn test_perlin_noise() {
        run_test(
            &PERLIN_TEST,
            |rand, (first_octave, amplitudes)| PerlinNoise::new(rand, *first_octave, amplitudes),
            PerlinNoise::noise,
        )
    }

    #[test]
    fn test_legacy_perlin_noise() {
        run_test(
            &PERLIN_LEGACY_TEST,
            |rand, range| {
                PerlinNoise::new_legacy(
                    rand,
                    range.clone().into_iter().collect::<Vec<_>>().as_slice(),
                )
            },
            PerlinNoise::noise,
        )
    }
}
