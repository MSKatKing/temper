//! Nested terrain splines.
//!
//! The spline graph is evaluated once per `(x, z)` column.
//! The resulting offset, factor, and jaggedness values are then reused for
//! every Y position in that column.

use std::sync::Arc;
use thiserror::Error;

use bevy_math::cubic_splines::CubicSegment;

/// A shared reference to a spline node.
///
/// `Arc` allows multiple parent spline points to reuse the same child spline
/// without duplicating the entire subtree.
pub type SplineRef = Arc<NestedSpline>;

/// All noise coordinates that a terrain spline may read.
///
/// These values should generally be normalized to approximately `[-1, 1]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainPoint {
    pub continentalness: f32,
    pub erosion: f32,

    /// The raw weirdness/ridge noise.
    pub ridges: f32,

    /// The folded peaks-and-valleys transformation of `ridges`.
    pub ridges_folded: f32,
}

impl TerrainPoint {
    pub fn new(
        continentalness: f32,
        erosion: f32,
        weirdness: f32,
    ) -> Self {
        Self {
            continentalness,
            erosion,
            ridges: weirdness,
            ridges_folded: peaks_and_valleys(weirdness),
        }
    }

    pub fn all_finite(self) -> bool {
        self.continentalness.is_finite()
            && self.erosion.is_finite()
            && self.ridges.is_finite()
            && self.ridges_folded.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplineCoordinate {
    Continentalness,
    Erosion,
    Ridges,
    RidgesFolded,
}

impl SplineCoordinate {
    pub fn get(self, point: TerrainPoint) -> f32 {
        match self {
            Self::Continentalness => point.continentalness,
            Self::Erosion => point.erosion,
            Self::Ridges => point.ridges,
            Self::RidgesFolded => point.ridges_folded,
        }
    }
}

/// The output attached to a spline knot.
#[derive(Debug, Clone)]
pub enum SplineValue {
    Constant(f32),
    Spline(SplineRef),
}

impl SplineValue {
    pub fn sample(&self, point: TerrainPoint) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Spline(spline) => spline.sample(point),
        }
    }

    pub fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    pub fn nested(spline: SplineRef) -> Self {
        Self::Spline(spline)
    }
}

#[derive(Debug, Clone)]
pub struct SplineKnot {
    pub location: f32,
    pub value: SplineValue,
    pub derivative: f32,
}

impl SplineKnot {
    pub fn constant(
        location: f32,
        value: f32,
        derivative: f32,
    ) -> Self {
        Self {
            location,
            value: SplineValue::Constant(value),
            derivative,
        }
    }

    pub fn nested(
        location: f32,
        spline: SplineRef,
        derivative: f32,
    ) -> Self {
        Self {
            location,
            value: SplineValue::Spline(spline),
            derivative,
        }
    }
}

#[derive(Debug)]
pub struct NestedSpline {
    coordinate: SplineCoordinate,
    knots: Box<[SplineKnot]>,
}

impl NestedSpline {
    pub fn new(
        coordinate: SplineCoordinate,
        knots: impl Into<Vec<SplineKnot>>,
    ) -> Result<Self, SplineError> {
        let knots = knots.into();

        if knots.len() < 2 {
            return Err(SplineError::TooFewKnots {
                coordinate,
                count: knots.len(),
            });
        }

        for (index, knot) in knots.iter().enumerate() {
            if !knot.location.is_finite() {
                return Err(SplineError::NonFiniteLocation {
                    coordinate,
                    index,
                    value: knot.location,
                });
            }

            if !knot.derivative.is_finite() {
                return Err(SplineError::NonFiniteDerivative {
                    coordinate,
                    index,
                    value: knot.derivative,
                });
            }

            if let SplineValue::Constant(value) = knot.value
                && !value.is_finite()
            {
                return Err(SplineError::NonFiniteConstant {
                    coordinate,
                    index,
                    value,
                });
            }
        }

        for (index, pair) in knots.windows(2).enumerate() {
            if pair[0].location >= pair[1].location {
                return Err(SplineError::LocationsNotIncreasing {
                    coordinate,
                    left_index: index,
                    left: pair[0].location,
                    right: pair[1].location,
                });
            }
        }

        Ok(Self {
            coordinate,
            knots: knots.into_boxed_slice(),
        })
    }

    pub fn coordinate(&self) -> SplineCoordinate {
        self.coordinate
    }

    pub fn knots(&self) -> &[SplineKnot] {
        &self.knots
    }

    /// Evaluates this node and any nested spline values.
    pub fn sample(&self, point: TerrainPoint) -> f32 {
        let input = self.coordinate.get(point);

        if !input.is_finite() {
            return f32::NAN;
        }

        let first = &self.knots[0];
        let last = &self.knots[self.knots.len() - 1];

        // Outside the authored range, continue linearly using the endpoint
        // derivative.
        if input <= first.location {
            let endpoint_value = first.value.sample(point);

            return endpoint_value
                + first.derivative * (input - first.location);
        }

        if input >= last.location {
            let endpoint_value = last.value.sample(point);

            return endpoint_value
                + last.derivative * (input - last.location);
        }

        // Find the first knot whose location is greater than `input`.
        let right_index = self
            .knots
            .partition_point(|knot| knot.location <= input);

        let left = &self.knots[right_index - 1];
        let right = &self.knots[right_index];

        let width = right.location - left.location;
        let t = (input - left.location) / width;

        // These may recursively evaluate entirely different spline branches.
        let left_value = left.value.sample(point);
        let right_value = right.value.sample(point);

        hermite_with_bevy(
            left_value,
            right_value,
            left.derivative,
            right.derivative,
            width,
            t,
        )
    }
}

fn hermite_with_bevy(
    start_value: f32,
    end_value: f32,
    start_derivative: f32,
    end_derivative: f32,
    width: f32,
    t: f32,
) -> f32 {
    debug_assert!(width > 0.0);
    debug_assert!((0.0..=1.0).contains(&t));

    let start_tangent = start_derivative * width;
    let end_tangent = end_derivative * width;

    let bezier_points = [
        start_value,
        start_value + start_tangent / 3.0,
        end_value - end_tangent / 3.0,
        end_value,
    ];

    CubicSegment::new_bezier(bezier_points).position(t)
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum SplineError {
    #[error("{coordinate:?} spline requires at least two knots; got {count}")]
    TooFewKnots {
        coordinate: SplineCoordinate,
        count: usize,
    },

    #[error("{coordinate:?} spline knot {index} has non-finite location {value}")]
    NonFiniteLocation {
        coordinate: SplineCoordinate,
        index: usize,
        value: f32,
    },

    #[error("{coordinate:?} spline knot {index} has non-finite derivative {value}")]
    NonFiniteDerivative {
        coordinate: SplineCoordinate,
        index: usize,
        value: f32,
    },

    #[error("{coordinate:?} spline knot {index} has non-finite constant value {value}")]
    NonFiniteConstant {
        coordinate: SplineCoordinate,
        index: usize,
        value: f32,
    },

    #[error("{coordinate:?} spline knot locations are not strictly increasing at index {left_index}: {left} >= {right}")]
    LocationsNotIncreasing {
        coordinate: SplineCoordinate,
        left_index: usize,
        left: f32,
        right: f32,
    },
}


#[inline]
pub fn spline(
    coordinate: SplineCoordinate,
    knots: impl Into<Vec<SplineKnot>>,
) -> Result<SplineRef, SplineError> {
    Ok(Arc::new(NestedSpline::new(coordinate, knots)?))
}

#[derive(Debug, Clone)]
pub struct TerrainSplines {
    pub offset: SplineRef,
    pub factor: SplineRef,
    pub jaggedness: SplineRef,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnShape {
    /// Vertical terrain displacement relative to sea level, in blocks.
    pub offset_blocks: f32,

    /// Multiplier applied to the vertical density gradient.
    pub factor: f32,

    /// Maximum displacement from the separate jagged noise, in blocks.
    pub jaggedness_blocks: f32,
}

impl TerrainSplines {
    pub fn sample(&self, point: TerrainPoint) -> ColumnShape {
        ColumnShape {
            offset_blocks: self.offset.sample(point),
            factor: self.factor.sample(point),
            jaggedness_blocks: self.jaggedness.sample(point),
        }
    }
}

/// Folds raw weirdness into a repeating valley-to-peak signal.
#[inline]
pub fn peaks_and_valleys(weirdness: f32) -> f32 {
    -((weirdness.abs() - 2.0 / 3.0).abs() - 1.0 / 3.0) * 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(value: f32) -> TerrainPoint {
        TerrainPoint {
            continentalness: value,
            erosion: value,
            ridges: value,
            ridges_folded: value,
        }
    }

    #[test]
    fn samples_endpoints_exactly() {
        let spline = NestedSpline::new(
            SplineCoordinate::Continentalness,
            vec![
                SplineKnot::constant(-1.0, 10.0, 0.0),
                SplineKnot::constant(1.0, 20.0, 0.0),
            ],
        )
            .unwrap();

        assert_eq!(spline.sample(point(-1.0)), 10.0);
        assert_eq!(spline.sample(point(1.0)), 20.0);
    }

    #[test]
    fn zero_tangents_give_smooth_midpoint() {
        let spline = NestedSpline::new(
            SplineCoordinate::Continentalness,
            vec![
                SplineKnot::constant(0.0, 0.0, 0.0),
                SplineKnot::constant(1.0, 10.0, 0.0),
            ],
        )
            .unwrap();

        let value = spline.sample(point(0.5));

        assert!((value - 5.0).abs() < 0.0001);
    }

    #[test]
    fn endpoint_extrapolation_uses_derivative() {
        let spline = NestedSpline::new(
            SplineCoordinate::Continentalness,
            vec![
                SplineKnot::constant(0.0, 10.0, 2.0),
                SplineKnot::constant(1.0, 20.0, 3.0),
            ],
        )
            .unwrap();

        assert!(
            (spline.sample(point(-1.0)) - 8.0).abs()
                < 0.0001
        );

        assert!(
            (spline.sample(point(2.0)) - 23.0).abs()
                < 0.0001
        );
    }

    #[test]
    fn nested_spline_uses_same_terrain_point() {
        let erosion_child = spline(
            SplineCoordinate::Erosion,
            vec![
                SplineKnot::constant(-1.0, 0.0, 0.0),
                SplineKnot::constant(1.0, 100.0, 0.0),
            ],
        )
            .unwrap();

        let root = NestedSpline::new(
            SplineCoordinate::Continentalness,
            vec![
                SplineKnot::nested(
                    -1.0,
                    Arc::clone(&erosion_child),
                    0.0,
                ),
                SplineKnot::nested(
                    1.0,
                    erosion_child,
                    0.0,
                ),
            ],
        )
            .unwrap();

        let sample = TerrainPoint {
            continentalness: 0.0,
            erosion: 0.5,
            ridges: 0.0,
            ridges_folded: 0.0,
        };

        let value = root.sample(sample);

        assert!((value - 84.375).abs() < 0.001);
    }

    #[test]
    fn rejects_duplicate_locations() {
        let result = NestedSpline::new(
            SplineCoordinate::Continentalness,
            vec![
                SplineKnot::constant(0.0, 1.0, 0.0),
                SplineKnot::constant(0.0, 2.0, 0.0),
            ],
        );

        assert!(matches!(
            result,
            Err(SplineError::LocationsNotIncreasing { .. })
        ));
    }
}