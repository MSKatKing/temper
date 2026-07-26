use bevy_math::DVec3;
use temper_noise::NormalNoise;
use crate::cpu::buffer::BufferId;

/// Represents a specific operation to carry out on a buffer(s)
pub enum Operation {
    /// Clears the specified buffer, setting all values to `value`
    ClearBuffer {
        destination: BufferId,
        value: f32,
    },

    /// Adds values from `source` into `destination`
    AddBuffer {
        destination: BufferId,
        source: ValueSource,
    },

    /// Multiplies `source` and `destination` and stores the result in `destination`
    MulBuffer {
        destination: BufferId,
        source: ValueSource,
    },
}

pub enum ValueSource {
    Buffer(BufferId),
    Constant(f32),
    Noise(NormalNoise),
}

impl ValueSource {
    pub fn get(&self, position: (u8, i16, u8)) -> f32 {
        match self {
            ValueSource::Constant(v) => *v,
            ValueSource::Noise(n) => n.noise(DVec3::new(position.0 as f64, position.1 as f64, position.2 as f64)) as f32,
            ValueSource::Buffer(id) => {
                0.0
            }
        }
    }
}
