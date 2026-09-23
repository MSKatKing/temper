use std::ops::Range;

pub const PARAMETER_COUNT: usize = 7;

pub type Parameter = Range<i64>;
pub type ParameterSpace = [Parameter; PARAMETER_COUNT];
pub type ParameterValues = [i64; PARAMETER_COUNT];

#[inline(always)]
pub fn parameter_span(a: &Parameter, b: &Parameter) -> Parameter {
    a.start.min(b.start)..a.end.max(b.end)
}

#[inline(always)]
pub fn parameter_distance(a: &Parameter, b: &Parameter) -> i64 {
    let above = b.start - a.end;
    let below = a.start - b.end;

    if above > 0 { above } else { below.max(0) }
}
