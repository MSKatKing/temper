pub mod stages;
pub mod splines;

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
                StageDependencies::with_neighbors(GenStage::BIOMES, GenStage::BIOMES, 1),
            )),
            GenStage::CARVERS => Some(StageSpec::new(
                stage,
                "carvers",
                StageDependencies::with_neighbors(GenStage::SURFACE, GenStage::SURFACE, 1),
            )),
            GenStage::FEATURES => Some(StageSpec::new(
                stage,
                "features",
                StageDependencies::with_neighbors(GenStage::CARVERS, GenStage::CARVERS, 1),
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
            GenStage::SURFACE => generate_surface(input, self.seed),
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

fn generate_surface(_input: StageInput<'_>, _seed: u64) -> Result<(), GenerationError> {
    Ok(())
}

fn generate_carvers(_input: StageInput<'_>, _seed: u64) -> Result<(), GenerationError> {
    Ok(())
}

fn generate_features(_input: StageInput<'_>, _seed: u64) -> Result<(), GenerationError> {
    Ok(())
}

fn finish_chunk(_input: StageInput<'_>) -> Result<(), GenerationError> {
    Ok(())
}
