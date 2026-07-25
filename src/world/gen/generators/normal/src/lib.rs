pub mod build_terrain_splines;
pub mod density;
pub mod splines;
pub mod stages;

use gen_core::{
    ChunkGenerator, GenStage, GenerationError, GeneratorId, StageDependencies, StageInput,
    StageSpec,
};

pub struct NormalGenerator {
    seed: u64,
}

impl NormalGenerator {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl ChunkGenerator for NormalGenerator {
    fn id(&self) -> GeneratorId {
        GeneratorId::new("normal")
    }

    fn final_stage(&self) -> GenStage {
        GenStage::FULL
    }

    fn stage_spec(&self, stage: GenStage) -> Option<StageSpec> {
        match stage {
            GenStage::EMPTY => Some(StageSpec::new(stage, "empty", StageDependencies::NONE)),
            GenStage::NOISE => Some(StageSpec::new(
                stage,
                "noise",
                StageDependencies::only_own(GenStage::EMPTY),
            )),
            GenStage::BIOMES => Some(StageSpec::new(
                stage,
                "biomes",
                StageDependencies::only_own(GenStage::NOISE),
            )),
            GenStage::SURFACE => Some(StageSpec::new(
                stage,
                "surface",
                StageDependencies::only_own(GenStage::BIOMES),
            )),
            GenStage::CARVERS => Some(StageSpec::new(
                stage,
                "carvers",
                StageDependencies::only_own(GenStage::SURFACE),
            )),
            GenStage::FEATURES => Some(StageSpec::new(
                stage,
                "features",
                StageDependencies::only_own(GenStage::CARVERS),
            )),
            GenStage::FULL => Some(StageSpec::new(
                stage,
                "full",
                StageDependencies::only_own(GenStage::FEATURES),
            )),
            _ => None,
        }
    }

    fn advance_stage(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        match input.stage {
            GenStage::EMPTY => generate_empty(input),
            GenStage::NOISE => self.generate_noises(input),
            GenStage::BIOMES => generate_biomes(input, self.seed),
            GenStage::SURFACE => self.generate_surface(input),
            GenStage::CARVERS => generate_carvers(input, self.seed),
            GenStage::FEATURES => generate_features(input, self.seed),
            GenStage::FULL => finish_chunk(input),
            _ => Ok(()),
        }
    }
}

fn generate_empty(_input: StageInput<'_>) -> Result<(), GenerationError> {
    Ok(())
}

fn generate_biomes(_input: StageInput<'_>, _seed: u64) -> Result<(), GenerationError> {
    Ok(())
}

fn generate_carvers(_input: StageInput<'_>, _seed: u64) -> Result<(), GenerationError> {
    Ok(())
}

fn generate_features(_input: StageInput<'_>, _seed: u64) -> Result<(), GenerationError> {
    Ok(())
}

fn finish_chunk(input: StageInput<'_>) -> Result<(), GenerationError> {
    // Clearing so we don't try to compress like 1.6mb of data we don't need on save
    input.target.noise.base3d.clear();
    input.target.noise.spaghetti_caves.clear();
    input.target.noise.cheese_caves.clear();
    input.target.noise.noddle_caves.clear();
    Ok(())
}

fn index3d(x: usize, y: usize, z: usize) -> usize {
    assert!(x < 16);
    assert!(y < 384);
    assert!(z < 16);
    // Layout is array[z][y][x], so x is the fastest-changing index.
    // index = z * (Y * X) + y * X + x
    const X: usize = 16;
    const Y: usize = 384;
    z * (Y * X) + y * X + x
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn inverse_lerp_clamped(start: f32, end: f32, value: f32) -> f32 {
    if start == end {
        return if value >= end { 1.0 } else { 0.0 };
    }

    ((value - start) / (end - start)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dependencies(stage: GenStage) -> StageDependencies {
        NormalGenerator::new(0)
            .stage_spec(stage)
            .expect("normal generator should define this stage")
            .dependencies
    }

    #[test]
    fn current_stages_only_depend_on_their_own_previous_stage() {
        assert_eq!(
            dependencies(GenStage::SURFACE),
            StageDependencies::only_own(GenStage::BIOMES),
        );
        assert_eq!(
            dependencies(GenStage::CARVERS),
            StageDependencies::only_own(GenStage::SURFACE),
        );
        assert_eq!(
            dependencies(GenStage::FEATURES),
            StageDependencies::only_own(GenStage::CARVERS),
        );
    }

    #[test]
    fn index3d_matches_quick_noise_grid_layout() {
        assert_eq!(index3d(0, 0, 0), 0);
        assert_eq!(index3d(15, 0, 0), 15);
        assert_eq!(index3d(0, 1, 0), 16);
        assert_eq!(index3d(0, 0, 1), 16 * 384);
    }
}
