use temper_core::math::TemperMathExt;
use crate::wrapped::WrappedDensityFunction;

pub enum FlattenedSpline<'a> {
    Multipoint {
        coordinate: usize,
        locations: &'a [f64],
        derivatives: &'a [f64],
        values: Vec<FlattenedSpline<'a>>,
    },
    Constant(f64),
}

impl FlattenedSpline<'_> {
    pub fn sample(&mut self, func: &WrappedDensityFunction) -> f64 {
        match self {
            FlattenedSpline::Multipoint {
                coordinate,
                locations,
                derivatives,
                values,
            } => {
                let input = func.execute_inner(*coordinate);
                let start = Self::find_interval_start(locations, input);
                let last_index = locations.len() - 1;

                if start < 0 {
                    let value = values[0].sample(func);
                    return self.linear_extend(input, value, 0);
                }

                let start = start as usize;
                if start == last_index {
                    let value = values.last_mut().unwrap().sample(func);
                    return self.linear_extend(input, value, last_index);
                }

                let x1 = locations[start];
                let x2 = locations[start + 1];
                let t = (input - x1) / (x2 - x1);
                let y1 = values[start].sample(func);
                let y2 = values[start + 1].sample(func);
                let d1 = derivatives[start];
                let d2 = derivatives[start + 1];
                let a = d1 * (x2 - x1) - (y2 - y1);
                let b = -d2 * (x2 - x1) + (y2 - y1);
                t.lerp(y1, y2) + t * (1.0 - t) * t.lerp(a, b)
            }
            FlattenedSpline::Constant(c) => *c,
        }
    }

    fn linear_extend(&self, input: f64, value: f64, index: usize) -> f64 {
        match self {
            FlattenedSpline::Multipoint {
                locations,
                derivatives,
                ..
            } => {
                let derivative = derivatives[index];
                if derivative == 0.0 {
                    value
                } else {
                    value + derivative * (input - locations[index])
                }
            }
            FlattenedSpline::Constant(constant) => *constant,
        }
    }

    fn find_interval_start(locations: &[f64], input: f64) -> isize {
        let value = locations.binary_search_by(|v| v.total_cmp(&input)).unwrap_or_else(|x| x);
        value as isize - 1
    }
}
