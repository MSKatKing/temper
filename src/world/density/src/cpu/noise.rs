use bevy_math::DVec3;
use temper_core::random::{PositionalRandom, RandomSource, XoroshiroRandomSource};
use temper_data::noise::NoiseParameter;
use temper_noise::NormalNoise;

#[derive(Clone, PartialEq, Debug)]
pub enum NoiseAccessType {
    Basic {
        xz_scale: f32,
        y_scale: f32,
    },
    Shift,
}

pub struct NoiseAccessor {
    noise: NormalNoise,
    pub access_type: NoiseAccessType,
}

impl NoiseAccessor {
    pub fn new(noise_param: &'static NoiseParameter, access_type: NoiseAccessType) -> Self {
        let mut rand = XoroshiroRandomSource::new(0); // TODO: replace with actual initialization for noise
        let noise = NormalNoise::new(&mut rand.fork_positional().spawn_from_hash("test"), noise_param.first_octave, noise_param.amplitudes);

        Self {
            noise,
            access_type,
        }
    }
    
    pub fn new_noise(noise: NormalNoise, access_type: NoiseAccessType) -> Self {
        Self {
            noise,
            access_type,
        }
    }

    pub fn noise(&self, pos: DVec3) -> f32 {
        self.apply_noise(
            self.noise.noise(
                self.apply_accessor(pos)
            ) as f32
        )
    }

    fn apply_accessor(&self, pos: DVec3) -> DVec3 {
        match self.access_type {
            NoiseAccessType::Basic { xz_scale, y_scale } => {
                DVec3::new(pos.x * xz_scale as f64, pos.y * y_scale as f64, pos.z * xz_scale as f64)
            },
            NoiseAccessType::Shift => pos / 4.0,
        }
    }

    fn apply_noise(&self, val: f32) -> f32 {
        match self.access_type {
            NoiseAccessType::Shift => val * 4.0,
            _ => val
        }
    }
}
