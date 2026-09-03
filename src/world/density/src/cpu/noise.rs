use bevy_math::DVec3;
use temper_core::pos::BlockPos;
use temper_core::random::{PositionalRandom, RandomSource};
use temper_data::noise::NoiseParameter;
use temper_noise::{BlendedNoise, NormalNoise};
use crate::DensityFunction;

#[derive(Clone, PartialEq, Debug)]
pub enum NoiseAccessType {
    Basic { xz_scale: f32, y_scale: f32 },
    Shift,
}

#[derive(Debug, Clone)]
pub enum NoiseType {
    Normal(NormalNoise),
    Blended(BlendedNoise),
}

#[derive(Clone, Debug)]
pub struct NoiseAccessor {
    noise: NoiseType,
    pub access_type: NoiseAccessType,
}

impl NoiseAccessor {
    pub fn new<R: RandomSource, P: PositionalRandom<R>>(
        noise_param: &'static NoiseParameter,
        rand: &mut P,
        name: &str,
        access_type: NoiseAccessType,
    ) -> Self {
        let noise = NormalNoise::new(
            &mut rand.spawn_from_hash(name),
            noise_param.first_octave,
            noise_param.amplitudes,
        );

        Self { noise: NoiseType::Normal(noise), access_type }
    }

    pub fn new_blended(
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Self {
        let noise = BlendedNoise::new_unseeded(
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        );

        Self {
            noise: NoiseType::Blended(noise),
            access_type: NoiseAccessType::Basic {
                xz_scale: 1.0,
                y_scale: 1.0,
            }
        }
    }

    pub fn new_noise(noise: NormalNoise, access_type: NoiseAccessType) -> Self {
        Self { noise: NoiseType::Normal(noise), access_type }
    }

    pub fn noise(&self, pos: BlockPos) -> f32 {
        self.apply_noise(self.noise.noise(self.apply_accessor(pos.pos.as_dvec3())) as f32)
    }

    fn apply_accessor(&self, pos: DVec3) -> DVec3 {
        match self.access_type {
            NoiseAccessType::Basic { xz_scale, y_scale } => DVec3::new(
                pos.x * xz_scale as f64,
                pos.y * y_scale as f64,
                pos.z * xz_scale as f64,
            ),
            NoiseAccessType::Shift => pos / 4.0,
        }
    }

    fn apply_noise(&self, val: f32) -> f32 {
        match self.access_type {
            NoiseAccessType::Shift => val * 4.0,
            _ => val,
        }
    }
}

impl NoiseType {
    pub fn noise(&self, pos: DVec3) -> f64 {
        match self {
            NoiseType::Normal(noise) => noise.noise(pos),
            NoiseType::Blended(noise) => noise.noise(BlockPos::of(
                pos.x as i32,
                pos.y as i32,
                pos.z as i32,
            )),
        }
    }
}

impl TryFrom<&DensityFunction> for NoiseAccessor {
    type Error = ();
    
    fn try_from(value: &DensityFunction) -> Result<Self, Self::Error> {
        todo!()
    }
}
