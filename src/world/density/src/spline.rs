use crate::wrapped::WrappedDensityFunction;
use crate::{BoxedDensityFunction, DensityFunction, DensityFunctionContext};
use temper_core::math::TemperMathExt;

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

#[derive(Debug)]
pub enum WrappedSpline<'a> {
    Multipoint {
        coordinate: Box<dyn WrappedDensityFunction + 'a>,
        locations: &'a [f64],
        derivatives: &'a [f64],
        values: Vec<WrappedSpline<'a>>,
    },
    Constant {
        value: f64,
    },
}

impl WrappedSpline<'_> {
    fn sample(&mut self, ctx: &DensityFunctionContext) -> f64 {
        match self {
            WrappedSpline::Multipoint {
                coordinate,
                locations,
                values,
                derivatives
            } => {
                let input = coordinate.compute(ctx);
                let start = Self::find_interval_start(locations, input);
                let last_idx = locations.len() - 1;

                if start < 0 {
                    let value = values[0].sample(ctx);
                    return self.linear_extend(input, value, 0);
                }

                let start = start as usize;

                if start == last_idx {
                    let value = values.last_mut().unwrap().sample(ctx);
                    return self.linear_extend(input, value, last_idx);
                }

                let x1 = locations[start];
                let x2 = locations[start + 1];
                let t = (input - x1) / (x2 - x1);
                let y1 = values[start].sample(ctx);
                let y2 = values[start + 1].sample(ctx);
                let d1 = derivatives[start];
                let d2 = derivatives[start + 1];
                let a = d1 * (x2 - x1) - (y2 - y1);
                let b = -d2 * (x2 - x1) + (y2 - y1);
                t.lerp(y1, y2) + t * (1.0 - t) * t.lerp(a, b)
            },
            WrappedSpline::Constant { value } => *value,
        }
    }

    fn linear_extend(&self, input: f64, value: f64, index: usize) -> f64 {
        match self {
            WrappedSpline::Multipoint { locations, derivatives, .. } => {
                let derivative = derivatives[index];
                if derivative == 0.0 {
                    value
                } else {
                    value + derivative * (input - locations[index])
                }
            },
            WrappedSpline::Constant { value } => *value,
        }
    }

    fn find_interval_start(locations: &[f64], input: f64) -> isize {
        let mut from = 0;
        let to = locations.len();
        let mut len = to - from;

        while len > 0 {
            let half = len / 2;
            let middle = from + half;

            if input < locations[middle] {
                len = half;
            } else {
                from = middle + 1;
                len -= half + 1;
            }
        }

        from as isize - 1
    }
}

impl Spline {
    fn wrap(&self) -> WrappedSpline<'_> {
        match self {
            Spline::Multipoint {
                values,
                locations,
                derivatives,
                coordinate
            } => {
                WrappedSpline::Multipoint {
                    locations,
                    derivatives,
                    coordinate: coordinate.wrap(),
                    values: values.iter().map(|v| v.wrap()).collect(),
                }
            },
            Spline::Constant { value } => WrappedSpline::Constant { value: *value },
        }
    }
}

impl DensityFunction for Spline {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(Self::wrap(self))
    }
}

impl WrappedDensityFunction for WrappedSpline<'_> {
    fn compute(&mut self, ctx: &DensityFunctionContext) -> f64 {
        self.sample(ctx)
    }
}
