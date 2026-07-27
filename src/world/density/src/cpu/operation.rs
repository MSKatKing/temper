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
    
    /// Subtracts values from `destination` by `source`
    SubBuffer {
        destination: BufferId,
        source: ValueSource,
    },
    
    /// Divides values from `destination` by `source`. Result is stored in `destination`
    DivBuffer {
        destination: BufferId,
        source: ValueSource,
    },

    /// Multiplies `source` and `destination` and stores the result in `destination`
    MulBuffer {
        destination: BufferId,
        source: ValueSource,
    },
    
    /// Takes the minimum of `destination` and `source` and stores it in `destination`
    MinBuffer {
        destination: BufferId,
        source: ValueSource,
    },
    
    /// Takes the maximum of `destination` and `source` and stores it in `destination`
    MaxBuffer {
        destination: BufferId,
        source: ValueSource,
    },
    
    /// Calculates the absolute value of every value in `buffer`
    AbsBuffer {
        buffer: BufferId,
    },
    
    /// Raises every value in `buffer` to the amount specified by `amount`
    PowBuffer {
        buffer: BufferId,
        amount: PowAmount,
    },
    
    /// Divides the values in `buffer` by `kind` if they are negative
    NegativeDecayBuffer {
        buffer: BufferId,
        kind: NegativeDecayType,
    },
    
    /// Clamps the buffer values between `min` and `max`
    ClampBuffer {
        buffer: BufferId,
        min: f32,
        max: f32,
    }
}

pub enum NegativeDecayType {
    Half,
    Quarter,
}

pub enum PowAmount {
    Square,
    Cube,
    Reciprocal,
}

impl PowAmount {
    pub fn as_i32(&self) -> i32 {
        match self {
            Self::Square => 2,
            Self::Cube => 3,
            Self::Reciprocal => -1,
        }
    }
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
