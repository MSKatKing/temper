use crate::splines::{
    SplineCoordinate, SplineError, SplineKnot, SplineRef, TerrainSplines, spline,
};

/// Builds the complete terrain spline set.
pub fn build_terrain_splines() -> Result<TerrainSplines, SplineError> {
    Ok(TerrainSplines {
        offset: build_offset_spline()?,
        factor: build_factor_spline()?,
        jaggedness: build_jaggedness_spline()?,
    })
}
fn offset_by_ridges(base: f32, relief: f32) -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::RidgesFolded,
        vec![
            // Deep valley.
            SplineKnot::constant(-1.00, base - relief * 0.20, 0.0),
            // Shallow valley.
            SplineKnot::constant(-0.40, base - relief * 0.10, 0.0),
            // Ordinary terrain.
            SplineKnot::constant(0.00, base, 0.0),
            // Low highlands.
            SplineKnot::constant(0.35, base + relief * 0.20, 0.0),
            // Mountain slope.
            SplineKnot::constant(0.70, base + relief * 0.65, 0.0),
            // Peak.
            SplineKnot::constant(1.00, base + relief, 0.0),
        ],
    )
}
fn offset_by_erosion(base: f32, maximum_relief: f32) -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::Erosion,
        vec![
            SplineKnot::nested(-1.00, offset_by_ridges(base, maximum_relief)?, 0.0),
            SplineKnot::nested(-0.45, offset_by_ridges(base, maximum_relief * 0.80)?, 0.0),
            SplineKnot::nested(0.00, offset_by_ridges(base, maximum_relief * 0.45)?, 0.0),
            SplineKnot::nested(0.45, offset_by_ridges(base, maximum_relief * 0.12)?, 0.0),
            // Very high erosion is almost completely flat.
            SplineKnot::nested(1.00, offset_by_ridges(base, 0.0)?, 0.0),
        ],
    )
}
fn build_offset_spline() -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::Continentalness,
        vec![
            // Deep ocean: 48 blocks below sea level.
            SplineKnot::constant(-1.00, -48.0, 0.0),
            // Ocean floor.
            SplineKnot::constant(-0.65, -40.0, 0.0),
            // Shallow ocean.
            SplineKnot::constant(-0.40, -26.0, 0.0),
            // Coast, with only slight terrain relief.
            SplineKnot::nested(-0.15, offset_by_erosion(-5.0, 10.0)?, 0.0),
            // Near inland.
            SplineKnot::nested(0.05, offset_by_erosion(3.0, 30.0)?, 0.0),
            // Deep inland, capable of large mountains.
            SplineKnot::nested(0.45, offset_by_erosion(8.0, 68.0)?, 0.0),
            // Farthest inland.
            SplineKnot::nested(1.00, offset_by_erosion(12.0, 86.0)?, 0.0),
        ],
    )
}
fn factor_by_ridges(ordinary_factor: f32, peak_factor: f32) -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::RidgesFolded,
        vec![
            // Valleys and ordinary terrain remain relatively rigid.
            SplineKnot::constant(-1.00, ordinary_factor, 0.0),
            SplineKnot::constant(-0.20, ordinary_factor, 0.0),
            // Begin weakening the vertical gradient toward peaks.
            SplineKnot::constant(0.40, ordinary_factor * 0.90 + peak_factor * 0.10, 0.0),
            SplineKnot::constant(0.75, ordinary_factor * 0.35 + peak_factor * 0.65, 0.0),
            SplineKnot::constant(1.00, peak_factor, 0.0),
        ],
    )
}

fn factor_by_erosion() -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::Erosion,
        vec![
            // Rugged terrain allows more volumetric distortion near peaks.
            SplineKnot::nested(-1.00, factor_by_ridges(1.25, 0.72)?, 0.0),
            SplineKnot::nested(-0.30, factor_by_ridges(1.30, 0.85)?, 0.0),
            SplineKnot::nested(0.30, factor_by_ridges(1.38, 1.15)?, 0.0),
            // High erosion produces rigid, smooth terrain.
            SplineKnot::constant(1.00, 1.45, 0.0),
        ],
    )
}

fn build_factor_spline() -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::Continentalness,
        vec![
            // Keep ocean terrain relatively stable.
            SplineKnot::constant(-1.00, 1.55, 0.0),
            SplineKnot::constant(-0.20, 1.50, 0.0),
            // Inland terrain uses erosion and folded ridges.
            SplineKnot::nested(0.05, factor_by_erosion()?, 0.0),
            SplineKnot::nested(1.00, factor_by_erosion()?, 0.0),
        ],
    )
}
fn jaggedness_by_ridges(maximum_blocks: f32) -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::RidgesFolded,
        vec![
            // Valleys and ordinary terrain are not jagged.
            SplineKnot::constant(-1.00, 0.0, 0.0),
            SplineKnot::constant(0.10, 0.0, 0.0),
            SplineKnot::constant(0.45, maximum_blocks * 0.15, 0.0),
            SplineKnot::constant(0.75, maximum_blocks * 0.60, 0.0),
            // Maximum jaggedness at peak regions.
            SplineKnot::constant(1.00, maximum_blocks, 0.0),
        ],
    )
}

fn jaggedness_by_erosion() -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::Erosion,
        vec![
            // Very low erosion supports strongly jagged peaks.
            SplineKnot::nested(-1.00, jaggedness_by_ridges(32.0)?, 0.0),
            SplineKnot::nested(-0.40, jaggedness_by_ridges(24.0)?, 0.0),
            SplineKnot::nested(0.00, jaggedness_by_ridges(10.0)?, 0.0),
            SplineKnot::nested(0.35, jaggedness_by_ridges(3.0)?, 0.0),
            // High erosion removes jaggedness entirely.
            SplineKnot::constant(1.00, 0.0, 0.0),
        ],
    )
}

fn build_jaggedness_spline() -> Result<SplineRef, SplineError> {
    spline(
        SplineCoordinate::Continentalness,
        vec![
            // Oceans and coasts never receive jagged peaks.
            SplineKnot::constant(-1.00, 0.0, 0.0),
            SplineKnot::constant(-0.15, 0.0, 0.0),
            // Fade jaggedness in once inland.
            SplineKnot::nested(0.10, jaggedness_by_erosion()?, 0.0),
            SplineKnot::nested(1.00, jaggedness_by_erosion()?, 0.0),
        ],
    )
}
