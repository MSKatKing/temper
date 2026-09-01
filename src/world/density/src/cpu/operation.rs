use crate::cpu::buffer::BufferId;
use crate::cpu::noise::{NoiseAccessType, NoiseAccessor};
use std::ops::RangeInclusive;
use temper_core::random::{PositionalRandom, RandomSource};
use temper_data::noise::NoiseParameter;
use crate::{DensityFunction, DensityFunctionArgument};

/// Represents a specific operation to carry out on a buffer(s)
#[derive(Clone)]
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

#[derive(Clone)]
pub enum NegativeDecayType {
    Half,
    Quarter,
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub enum ValueSource {
    Buffer(BufferId, Projection),
    Constant(f32),
    Noise(NoiseAccessor),
}

impl ValueSource {
    pub fn try_from<R: RandomSource, P: PositionalRandom<R>>(func: &DensityFunctionArgument, rand: &mut P) -> Option<ValueSource> {
        match func {
            DensityFunctionArgument::Function(func) => match func.as_ref() {
                DensityFunction::Constant { value } => Some(ValueSource::Constant(*value as _)),
                DensityFunction::Noise { noise, xz_scale, y_scale } => {
                    let param = NoiseParameter::get_by_name(noise.as_str())?;

                    Some(
                        ValueSource::Noise(
                            NoiseAccessor::new(
                                param,
                                rand,
                                noise.as_str(),
                                NoiseAccessType::Basic {
                                    xz_scale: * xz_scale as f32,
                                    y_scale: *y_scale as f32
                                }
                            )
                        )
                    )
                },
                DensityFunction::Shift { noise } => {
                    let param = NoiseParameter::get_by_name(noise.as_str())?;

                    Some(
                        ValueSource::Noise(
                            NoiseAccessor::new(
                                param,
                                rand,
                                noise.as_str(),
                                NoiseAccessType::Shift,
                            ),
                        ),
                    )
                }
                _ => None,
            }
            DensityFunctionArgument::Constant(value) => Some(ValueSource::Constant(*value as _)),
            DensityFunctionArgument::External(_) => panic!("functions should be linked prior to being compiled"),
        }
    }
}
