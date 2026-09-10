use crate::wrapped::spline::FlattenedSpline;
use crate::wrapped::{FlattenedDensityFunction, push_op};
use crate::{BoxedDensityFunction, DensityFunction};

#[derive(Debug)]
pub enum Spline {
    Multipoint {
        coordinate: BoxedDensityFunction,
        locations: Vec<f64>,
        values: Vec<Spline>,
        derivatives: Vec<f64>,
    },
    Constant {
        value: f64,
    },
}

impl Spline {
    fn wrap_type<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> FlattenedSpline<'a> {
        match self {
            Spline::Multipoint {
                values,
                locations,
                derivatives,
                coordinate,
            } => FlattenedSpline::Multipoint {
                locations,
                derivatives,
                coordinate: coordinate.wrap(ops),
                values: values.iter().map(|v| v.wrap_type(ops)).collect(),
            },
            Spline::Constant { value } => FlattenedSpline::Constant(*value),
        }
    }
}

impl DensityFunction for Spline {
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        match self {
            Spline::Constant { value } => {
                push_op(ops, |_| FlattenedDensityFunction::Constant(*value))
            }
            Spline::Multipoint {
                coordinate,
                locations,
                values,
                derivatives,
            } => push_op(ops, |ops| {
                FlattenedDensityFunction::Spline(FlattenedSpline::Multipoint {
                    coordinate: coordinate.wrap(ops),
                    locations,
                    derivatives,
                    values: values.iter().map(|s| s.wrap_type(ops)).collect(),
                })
            }),
        }
    }
}
