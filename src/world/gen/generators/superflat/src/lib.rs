use bevy_math::IVec3;
use gen_core::{
    ChunkGenerator, GenStage, GenerationError, GeneratorId, StageDependencies, StageInput,
    StageSpec,
};
use gen_structures::tree::generate_tree;
use temper_core::block_state_id::BlockStateId;
use temper_core::pos::{BlockPos, ChunkBlockPos, ChunkPos};
use temper_macros::block;

const TREE_ORIGIN_ATTEMPTS: i32 = 4;
const TREE_NEIGHBOR_RADIUS: i32 = 1;
const TREE_CHANCE: u64 = 3;

pub struct SuperflatGenerator {
    seed: u64,
}

impl SuperflatGenerator {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl ChunkGenerator for SuperflatGenerator {
    fn id(&self) -> GeneratorId {
        GeneratorId::new("superflat")
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
                StageDependencies::with_neighbors(GenStage::CARVERS, GenStage::SURFACE, 1),
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
            GenStage::EMPTY => {
                let target = input.target;
                for y in -4..3 {
                    target.fill_section(y, block!("stone"));
                }
                target.fill_section(3, block!("dirt"));
                for x in 0..16 {
                    for z in 0..16 {
                        target.set_block(
                            ChunkBlockPos::new(x, 64, z),
                            block!("grass_block", {snowy: false}),
                        );
                    }
                }
                Ok(())
            }
            GenStage::FEATURES => {
                for origin_chunk in nearby_chunks(input.pos, TREE_NEIGHBOR_RADIUS) {
                    for origin in tree_origins_for_chunk(self.seed, origin_chunk) {
                        let blocks = generate_tree(origin, self.seed);

                        for (offset, block) in &blocks {
                            let block_pos = origin
                                + IVec3::new(
                                    i32::from(offset.x),
                                    i32::from(offset.y),
                                    i32::from(offset.z),
                                );

                            if block_pos.chunk() == input.pos {
                                input.target.set_block(block_pos.chunk_block_pos(), *block);
                            }
                        }
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn nearby_chunks(pos: ChunkPos, radius: i32) -> impl Iterator<Item = ChunkPos> {
    (-radius..=radius).flat_map(move |x| (-radius..=radius).map(move |z| pos + (x, z)))
}

fn tree_origins_for_chunk(seed: u64, chunk: ChunkPos) -> impl Iterator<Item = BlockPos> {
    (0..TREE_ORIGIN_ATTEMPTS).filter_map(move |attempt| {
        let attempt_pos = chunk.block_offset(attempt, 65, attempt);
        let rand = attempt_pos.deterministic_rand(seed);

        if !rand.is_multiple_of(TREE_CHANCE) {
            return None;
        }

        let x = ((rand >> 8) & 0xf) as i32;
        let z = ((rand >> 12) & 0xf) as i32;

        Some(chunk.block_offset(x, 65, z))
    })
}
