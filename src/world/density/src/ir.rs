use std::collections::HashMap;
use serde::Deserialize;
use temper_core::math::lerp;

macro_rules! density_function {
    ($($name:ident ($alias:literal) $({ $($field:ident $(($field_alias:literal))?: $ty:ty),* })?;)*) => {
        #[derive(Deserialize, PartialEq, Debug, Clone)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum DensityFunction {
            $(
                #[serde(alias = $alias)]
                $name $({
                    $(
                        $(#[serde(alias = $field_alias)])?
                        $field: $ty,
                    )*
                })?
            ),*
        }
    };
}

density_function!(
    Cache2d ("minecraft:cache_2d") { input ("argument"): DensityFunctionArgument };
    CacheAllInCell ("minecraft:cache_all_in_cell") { input ("argument"): DensityFunctionArgument };
    CacheOnce ("minecraft:cache_once") { input ("argument"): DensityFunctionArgument };
    FlatCache ("minecraft:flat_cache") { input ("argument"): DensityFunctionArgument };
    Interpolated ("minecraft:interpolated") { input ("argument"): DensityFunctionArgument };
    Abs ("minecraft:abs") { input ("argument"): DensityFunctionArgument };
    Cube ("minecraft:cube") { input ("argument"): DensityFunctionArgument };
    HalfNegative ("minecraft:half_negative") { input ("argument"): DensityFunctionArgument };
    IntervalSelect ("minecraft:interval_select") { input: DensityFunctionArgument, thresholds: Vec<f64>, functions: Vec<DensityFunctionArgument> };
    Invert ("minecraft:invert") { input ("argument"): DensityFunctionArgument };
    QuarterNegative ("minecraft:quarter_negative") { input ("argument"): DensityFunctionArgument };
    Square ("minecraft:square") { input ("argument"): DensityFunctionArgument };
    Squeeze ("minecraft:squeeze") { input ("argument"): DensityFunctionArgument };
    Add ("minecraft:add") { left ("argument1"): DensityFunctionArgument, right ("argument2"): DensityFunctionArgument };
    Div ("minecraft:div") { left ("argument1"): DensityFunctionArgument, right ("argument2"): DensityFunctionArgument };
    Mul ("minecraft:mul") { left ("argument1"): DensityFunctionArgument, right ("argument2"): DensityFunctionArgument };
    Sub ("minecraft:sub") { left ("argument1"): DensityFunctionArgument, right ("argument2"): DensityFunctionArgument };
    Min ("minecraft:min") { left ("argument1"): DensityFunctionArgument, right ("argument2"): DensityFunctionArgument };
    Max ("minecraft:max") { left ("argument1"): DensityFunctionArgument, right ("argument2"): DensityFunctionArgument };
    Beardifier ("minecraft:beardifier");
    BlendAlpha ("minecraft:blend_alpha");
    BlendDensity ("minecraft:blend_density") { input ("argument"): DensityFunctionArgument };
    BlendOffset ("minecraft:blend_offset");
    Ceil ("minecraft:ceil") { input: DensityFunctionArgument, multiple: i32 };
    Clamp ("minecraft:clamp") { input: DensityFunctionArgument, min: f64, max: f64 };
    Constant ("minecraft:constant") { value ("argument"): f64 };
    EndIslands ("minecraft:end_islands");
    FindTopSurface ("minecraft:find_top_surface") { density: DensityFunctionArgument, upper_bound: DensityFunctionArgument, lower_bound: i32, cell_height: i32 };
    Floor ("minecraft:floor") { input: DensityFunctionArgument, multiple: i32 };
    Lerp ("minecraft:lerp") { alpha: DensityFunctionArgument, first: DensityFunctionArgument, second: DensityFunctionArgument };
    Negate ("minecraft:negate") { input: DensityFunctionArgument };
    Noise ("minecraft:noise") { noise: String, xz_scale: f64, y_scale: f64 };
    OldBlendedNoise ("minecraft:old_blended_noise") { xz_scale: f64, y_scale: f64, xz_factor: f64, y_factor: f64, smear_scale_multiplier: f64 };
    RangeChoice ("minecraft:range_choice") { input: DensityFunctionArgument, min_inclusive: f64, max_exclusive: f64, when_in_range: DensityFunctionArgument, when_out_of_range: DensityFunctionArgument };
    Round ("minecraft:round") { input: DensityFunctionArgument, multiple: i32 };
    Shift ("minecraft:shift") { noise ("argument"): String };
    ShiftA ("minecraft:shift_a") { noise ("argument"): String };
    ShiftB ("minecraft:shift_b") { noise ("argument"): String };
    ShiftedNoise ("minecraft:shifted_noise") { noise: String, xz_scale: f64, y_scale: f64, shift_x: DensityFunctionArgument, shift_y: DensityFunctionArgument, shift_z: DensityFunctionArgument };
    Truncate ("minecraft:truncate") { input: DensityFunctionArgument, multiple: i32 };
    YClampedGradient ("minecraft:y_clamped_gradient") { from_y: i32, to_y: i32, from_value: f64, to_value: f64 };
    // TODO: splines
    WeirdScaledSampler ("minecraft:weird_scaled_sampler") { rarity_value_mapper: String, noise: String, input: DensityFunctionArgument };
);

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum DensityFunctionArgument {
    Function(Box<DensityFunction>),
    Constant(f64),
    External(String),
}

impl DensityFunctionArgument {
    fn wrap_func(func: DensityFunction) -> DensityFunctionArgument {
        DensityFunctionArgument::Function(Box::new(func))
    }

    fn wrap_const(val: f64) -> DensityFunctionArgument {
        DensityFunctionArgument::Constant(val)
    }

    fn constant(&self) -> Option<f64> {
        match self {
            Self::Constant(c) => Some(*c),
            Self::Function(func) if let DensityFunction::Constant { value } = func.as_ref() => {
                Some(*value)
            }
            _ => None,
        }
    }

    fn link_arg(&mut self, externals: &HashMap<String, DensityFunctionArgument>) {
        if let Self::External(ext) = self {
            let mut ext = externals.get(ext).cloned().unwrap();
            ext.link_arg(externals);
            *self = ext;
        }
    }
}

impl DensityFunction {
    pub fn fold(self) -> DensityFunctionArgument {
        use DensityFunctionArgument as Arg;

        macro_rules! fold_cache {
            ($ty:ident, $input:expr) => {{
                let input = fold_arg($input);
                match input.constant() {
                    Some(val) => DensityFunctionArgument::Constant(val),
                    _ if let DensityFunctionArgument::Function(f) = &input
                        && let DensityFunction::$ty { input } = f.as_ref() =>
                    {
                        input.clone()
                    }
                    _ => Arg::wrap_func(DensityFunction::$ty { input }),
                }
            }};
        }

        match self {
            DensityFunction::Cache2d { input } => fold_cache!(Cache2d, input),
            DensityFunction::FlatCache { input } => fold_cache!(FlatCache, input),
            DensityFunction::CacheAllInCell { input } => fold_cache!(CacheAllInCell, input),
            DensityFunction::CacheOnce { input } => fold_cache!(CacheOnce, input),
            DensityFunction::Interpolated { input } => fold_cache!(Interpolated, input),
            DensityFunction::Add { left, right } => {
                let (left, right) = (fold_arg(left), fold_arg(right));
                match (left.constant(), right.constant()) {
                    (Some(left), Some(right)) => Arg::wrap_const(left + right),
                    (Some(0.0), _) => right,
                    (_, Some(0.0)) => left,
                    _ if left == right => Arg::wrap_func(DensityFunction::Mul {
                        left,
                        right: Arg::wrap_const(2.0),
                    }),
                    _ => Arg::wrap_func(DensityFunction::Add { left, right }),
                }
            }
            DensityFunction::Sub { left, right } => {
                let (left, right) = (fold_arg(left), fold_arg(right));
                match (left.constant(), right.constant()) {
                    (Some(left), Some(right)) => Arg::wrap_const(left - right),
                    (Some(0.0), _) => Arg::wrap_func(DensityFunction::Negate { input: right }),
                    (_, Some(0.0)) => left,
                    _ if left == right => Arg::wrap_const(0.0),
                    _ => Arg::wrap_func(DensityFunction::Sub { left, right }),
                }
            }
            DensityFunction::Mul { left, right } => {
                let (left, right) = (fold_arg(left), fold_arg(right));
                match (left.constant(), right.constant()) {
                    (Some(0.0), _) | (_, Some(0.0)) => Arg::wrap_const(0.0),
                    (Some(left), Some(right)) => Arg::wrap_const(left * right),
                    (Some(1.0), _) => right,
                    (_, Some(1.0)) => left,
                    _ if left == right => Arg::wrap_func(DensityFunction::Square { input: left }),
                    _ => Arg::wrap_func(DensityFunction::Mul { left, right }),
                }
            }
            DensityFunction::Div { left, right } => {
                let (left, right) = (fold_arg(left), fold_arg(right));
                match (left.constant(), right.constant()) {
                    (Some(left), Some(right)) => Arg::wrap_const(left / right),
                    (Some(0.0), _) => Arg::wrap_const(0.0),
                    (_, Some(0.0)) => panic!("divide by zero"),
                    _ if left == right => Arg::wrap_const(1.0),
                    _ => Arg::wrap_func(DensityFunction::Div { left, right }),
                }
            }
            DensityFunction::Abs { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(val.abs()),
                    _ if let DensityFunctionArgument::Function(f) = &input
                        && let DensityFunction::Negate { input } = f.as_ref() =>
                    {
                        input.clone()
                    }
                    _ => Arg::wrap_func(DensityFunction::Abs { input }),
                }
            }
            DensityFunction::Cube { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(val.powi(3)),
                    _ => Arg::wrap_func(DensityFunction::Cube { input }),
                }
            }
            DensityFunction::HalfNegative { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(if val.is_sign_negative() {
                        val / 2.0
                    } else {
                        val
                    }),
                    _ => Arg::wrap_func(DensityFunction::HalfNegative { input }),
                }
            }
            DensityFunction::IntervalSelect {
                input,
                thresholds,
                functions,
            } => {
                let input = fold_arg(input);
                let functions = functions.into_iter().map(fold_arg).collect::<Vec<_>>();

                match input.constant() {
                    Some(_) => {
                        todo!()
                    }
                    _ => Arg::wrap_func(DensityFunction::IntervalSelect {
                        input,
                        thresholds,
                        functions,
                    }),
                }
            }
            DensityFunction::Invert { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(val.recip()),
                    _ if let DensityFunctionArgument::Function(f) = &input
                        && let DensityFunction::Invert { input } = f.as_ref() =>
                    {
                        input.clone()
                    }
                    _ => Arg::wrap_func(DensityFunction::Invert { input }),
                }
            }
            DensityFunction::QuarterNegative { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(if val.is_sign_negative() {
                        val / 4.0
                    } else {
                        val
                    }),
                    _ => Arg::wrap_func(DensityFunction::QuarterNegative { input }),
                }
            }
            DensityFunction::Square { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(val.powi(2)),
                    _ => Arg::wrap_func(DensityFunction::Square { input }),
                }
            }
            DensityFunction::Squeeze { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => {
                        let val = val.clamp(-1.0, 1.0);
                        Arg::wrap_const((val / 2.0) - (val.powi(3) / 24.0))
                    }
                    _ => Arg::wrap_func(DensityFunction::Squeeze { input }),
                }
            }
            DensityFunction::Min { left, right } => {
                let (left, right) = (fold_arg(left), fold_arg(right));
                match (left.constant(), right.constant()) {
                    (Some(left), Some(right)) => Arg::wrap_const(left.min(right)),
                    _ if left == right => left,
                    _ => Arg::wrap_func(DensityFunction::Min { left, right }),
                }
            }
            DensityFunction::Max { left, right } => {
                let (left, right) = (fold_arg(left), fold_arg(right));
                match (left.constant(), right.constant()) {
                    (Some(left), Some(right)) => Arg::wrap_const(left.max(right)),
                    _ if left == right => left,
                    _ => Arg::wrap_func(DensityFunction::Max { left, right }),
                }
            }
            DensityFunction::Beardifier => Arg::wrap_const(0.0), // TODO
            DensityFunction::BlendAlpha => Arg::wrap_const(1.0), // TODO
            DensityFunction::BlendDensity { input } => fold_arg(input), // TODO
            DensityFunction::BlendOffset => Arg::wrap_const(0.0), // TODO
            DensityFunction::Ceil { input, multiple } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(nearest_multiple(val.ceil(), multiple as f64)),
                    _ => Arg::wrap_func(DensityFunction::Ceil { input, multiple }),
                }
            }
            DensityFunction::Clamp { input, min, max } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(val.clamp(min, max)),
                    _ => Arg::wrap_func(DensityFunction::Clamp { input, min, max }),
                }
            }
            DensityFunction::Constant { value } => Arg::wrap_const(value),
            DensityFunction::EndIslands => Arg::wrap_func(DensityFunction::EndIslands),
            DensityFunction::FindTopSurface {
                density,
                upper_bound,
                lower_bound,
                cell_height,
            } => {
                // TODO: fold this
                Arg::wrap_func(DensityFunction::FindTopSurface {
                    density,
                    upper_bound,
                    lower_bound,
                    cell_height,
                })
            }
            DensityFunction::Floor { input, multiple } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(nearest_multiple(val.floor(), multiple as f64)),
                    _ => Arg::wrap_func(DensityFunction::Floor { input, multiple }),
                }
            }
            DensityFunction::Lerp {
                alpha,
                first,
                second,
            } => {
                let (alpha, first, second) = (fold_arg(alpha), fold_arg(first), fold_arg(second));
                match (alpha.constant(), first.constant(), second.constant()) {
                    (Some(0.0), _, _) => first,
                    (Some(1.0), _, _) => second,
                    (Some(alpha), Some(first), Some(second)) => {
                        Arg::wrap_const(lerp(alpha, [first, second]))
                    }
                    _ if first == second => first,
                    _ => Arg::wrap_func(DensityFunction::Lerp {
                        alpha,
                        first,
                        second,
                    }),
                }
            }
            DensityFunction::Negate { input } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(-val),
                    _ if let DensityFunctionArgument::Function(f) = &input
                        && let DensityFunction::Negate { input } = f.as_ref() =>
                    {
                        input.clone()
                    }
                    _ => Arg::wrap_func(DensityFunction::Negate { input }),
                }
            }
            DensityFunction::Noise {
                noise,
                xz_scale,
                y_scale,
            } => Arg::wrap_func(DensityFunction::Noise {
                noise,
                xz_scale,
                y_scale,
            }),
            DensityFunction::OldBlendedNoise {
                xz_scale,
                y_scale,
                xz_factor,
                y_factor,
                smear_scale_multiplier,
            } => Arg::wrap_func(DensityFunction::OldBlendedNoise {
                xz_scale,
                y_scale,
                xz_factor,
                y_factor,
                smear_scale_multiplier,
            }),
            DensityFunction::RangeChoice {
                input,
                min_inclusive,
                max_exclusive,
                when_in_range,
                when_out_of_range,
            } => {
                let (input, when_in_range, when_out_of_range) = (
                    fold_arg(input),
                    fold_arg(when_in_range),
                    fold_arg(when_out_of_range),
                );
                match input.constant() {
                    Some(val) if (min_inclusive..max_exclusive).contains(&val) => when_in_range,
                    Some(_) => when_out_of_range,
                    _ => Arg::wrap_func(DensityFunction::RangeChoice {
                        input,
                        min_inclusive,
                        max_exclusive,
                        when_in_range,
                        when_out_of_range,
                    }),
                }
            }
            DensityFunction::Round { input, multiple } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(nearest_multiple(val.round(), multiple as f64)),
                    _ => Arg::wrap_func(DensityFunction::Round { input, multiple }),
                }
            }
            DensityFunction::Shift { noise } => Arg::wrap_func(DensityFunction::Shift { noise }),
            DensityFunction::ShiftA { noise } => Arg::wrap_func(DensityFunction::ShiftA { noise }),
            DensityFunction::ShiftB { noise } => Arg::wrap_func(DensityFunction::ShiftB { noise }),
            DensityFunction::ShiftedNoise {
                noise,
                xz_scale,
                y_scale,
                shift_x,
                shift_y,
                shift_z,
            } => {
                let (shift_x, shift_y, shift_z) =
                    (fold_arg(shift_x), fold_arg(shift_y), fold_arg(shift_z));
                Arg::wrap_func(DensityFunction::ShiftedNoise {
                    noise,
                    xz_scale,
                    y_scale,
                    shift_x,
                    shift_y,
                    shift_z,
                })
            }
            DensityFunction::Truncate { input, multiple } => {
                let input = fold_arg(input);
                match input.constant() {
                    Some(val) => Arg::wrap_const(nearest_multiple(val.trunc(), multiple as f64)),
                    _ => Arg::wrap_func(DensityFunction::Truncate { input, multiple }),
                }
            }
            DensityFunction::YClampedGradient {
                from_y,
                to_y,
                from_value,
                to_value,
            } => {
                if from_value == to_value {
                    Arg::wrap_const(from_value)
                } else {
                    Arg::wrap_func(DensityFunction::YClampedGradient {
                        from_y,
                        to_y,
                        from_value,
                        to_value,
                    })
                }
            }
            DensityFunction::WeirdScaledSampler {
                rarity_value_mapper,
                noise,
                input,
            } => {
                let input = fold_arg(input);
                Arg::wrap_func(DensityFunction::WeirdScaledSampler {
                    rarity_value_mapper,
                    noise,
                    input,
                })
            }
        }
    }

    pub fn link(&mut self, externals: &HashMap<String, DensityFunctionArgument>) {
        match self {
            DensityFunction::Cache2d { input }
                | DensityFunction::CacheAllInCell { input }
                | DensityFunction::CacheOnce { input }
                | DensityFunction::FlatCache { input }
                | DensityFunction::Interpolated { input }
                | DensityFunction::Abs { input }
                | DensityFunction::Cube { input }
                | DensityFunction::HalfNegative { input }
                | DensityFunction::Invert { input }
                | DensityFunction::QuarterNegative { input }
                | DensityFunction::Square { input }
                | DensityFunction::Squeeze { input }
                | DensityFunction::BlendDensity { input }
                | DensityFunction::IntervalSelect { input, .. }
                | DensityFunction::Ceil { input, .. }
                | DensityFunction::Clamp { input, .. }
                | DensityFunction::Floor { input, .. }
                | DensityFunction::Round { input, .. }
                | DensityFunction::Truncate { input, .. }
                | DensityFunction::WeirdScaledSampler { input, .. }
                | DensityFunction::Negate { input } => input.link_arg(externals),
            DensityFunction::Add { left, right }
                | DensityFunction::Div { left, right }
                | DensityFunction::Mul { left, right }
                | DensityFunction::Sub { left, right }
                | DensityFunction::Min { left, right }
                | DensityFunction::Max { left, right } => {
                left.link_arg(externals);
                right.link_arg(externals);
            },
            DensityFunction::Lerp { alpha, first, second } => {
                alpha.link_arg(externals);
                first.link_arg(externals);
                second.link_arg(externals);
            }
            DensityFunction::FindTopSurface { density, upper_bound, .. } => {
                density.link_arg(externals);
                upper_bound.link_arg(externals);
            }
            DensityFunction::RangeChoice { input, when_in_range, when_out_of_range, .. } => {
                input.link_arg(externals);
                when_in_range.link_arg(externals);
                when_out_of_range.link_arg(externals);
            }
            DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => {
                shift_x.link_arg(externals);
                shift_y.link_arg(externals);
                shift_z.link_arg(externals);
            }
            DensityFunction::Beardifier
                | DensityFunction::BlendAlpha
                | DensityFunction::BlendOffset
                | DensityFunction::Constant { .. }
                | DensityFunction::Noise { .. }
                | DensityFunction::Shift { .. }
                | DensityFunction::ShiftA { .. }
                | DensityFunction::ShiftB { .. }
                | DensityFunction::OldBlendedNoise { .. }
                | DensityFunction::YClampedGradient { .. }
                | DensityFunction::EndIslands => {}
        }
    }
}

fn fold_arg(arg: DensityFunctionArgument) -> DensityFunctionArgument {
    match arg {
        DensityFunctionArgument::Function(f) => {
            let f = f.fold();
            match f.constant() {
                Some(val) => DensityFunctionArgument::Constant(val),
                _ => f,
            }
        }
        DensityFunctionArgument::Constant(c) => DensityFunctionArgument::Constant(c),
        DensityFunctionArgument::External(_) => panic!("linking should happen before folding"),
    }
}

fn nearest_multiple(x: f64, multiple: f64) -> f64 {
    ((x + multiple / 2.0) / multiple).floor() * multiple
}
