use crate::cpu::buffer::BufferId;
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use std::ops::RangeInclusive;
use temper_data::noise::NoiseParameter;

/// Represents a specific operation to carry out on a buffer(s)
pub enum Operation {
    /// Clears the specified buffer, setting all values to `value`
    ClearBuffer { destination: BufferId, source: ValueSource },

    YClampedGradient { destination: BufferId, y_range: RangeInclusive<i16>, value_range: RangeInclusive<f32> },

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
    AbsBuffer { buffer: BufferId },

    /// Raises every value in `buffer` to the amount specified by `amount`
    PowBuffer { buffer: BufferId, amount: PowAmount },

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
    },
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

pub enum Projection {
    None,
    DropY,
    ShrinkXZ2,
}

impl Projection {
    pub fn project(&self, idx: usize) -> usize {
        match self {
            Projection::None => idx,
            Projection::DropY => idx & 0xFF,
            Projection::ShrinkXZ2 => ((idx & 0x03) >> 2) | (idx & 0x3),
        }
    }
}

pub enum ValueSource {
    Buffer(BufferId, Projection),
    Constant(f32),
    Noise(NoiseAccessor),
}
