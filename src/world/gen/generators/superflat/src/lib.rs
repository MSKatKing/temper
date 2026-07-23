use gen_core::{ChunkGenerator, GenStage, GenerationError, GeneratorId, StageInput, StageSpec};

pub struct SuperflatGenerator;

impl ChunkGenerator for SuperflatGenerator {
    fn id(&self) -> GeneratorId {
        GeneratorId::new("superflat")
    }

    fn final_stage(&self) -> GenStage {
        GenStage::FULL
    }

    fn stage_spec(&self, stage: GenStage) -> Option<StageSpec> {
        todo!()
    }

    fn advance_stage(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        todo!()
    }
}
