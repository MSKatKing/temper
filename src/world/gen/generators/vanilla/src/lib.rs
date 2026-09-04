use gen_core::{ChunkGenerator, GenStage, GenerationError, GeneratorId, StageDependencies, StageInput, StageSpec};
use temper_core::pos::ChunkBlockPos;
use temper_core::random::XoroshiroRandomSource;
use temper_core::block_state_id::BlockStateId;
use temper_density::cpu::compiler::{CompiledDensityFunction, Compiler};
use temper_density::{DensityFunction, DensityFunctionArgument};
use temper_density::cpu::buffer::{Full, BufferType};
use temper_density::cpu::workspace::Workspace;
use temper_macros::block;

pub struct VanillaGenerator {
    rand: XoroshiroRandomSource,
    final_density: CompiledDensityFunction,
}

impl VanillaGenerator {
    pub fn new(seed: u64) -> VanillaGenerator {
        let mut rand = XoroshiroRandomSource::new(seed);
        let func = DensityFunctionArgument::Function(Box::new(DensityFunction::Interpolated {
            input: DensityFunctionArgument::Function(Box::new(DensityFunction::Add {
                left: DensityFunctionArgument::Function(Box::new(DensityFunction::Cache2d {
                    input: DensityFunctionArgument::Function(Box::new(DensityFunction::Noise {
                        noise: "minecraft:surface".to_string(),
                        xz_scale: 1.0,
                        y_scale: 1.0,
                    })),
                })),
                right: DensityFunctionArgument::Function(Box::new(DensityFunction::YClampedGradient {
                    from_y: 32,
                    to_y: 96,
                    from_value: 2.0,
                    to_value: -2.0,
                })),
            }))
        }));

        let func = func.fold();
        let final_density = Compiler::compile(&mut rand, func);

        Self {
            rand,
            final_density,
        }
    }
}

impl ChunkGenerator for VanillaGenerator {
    fn id(&self) -> GeneratorId {
        GeneratorId::new("vanilla")
    }

    fn final_stage(&self) -> GenStage {
        GenStage::SURFACE
    }

    fn stage_spec(&self, stage: GenStage) -> Option<StageSpec> {
        match stage {
            GenStage::EMPTY => Some(StageSpec::new(
                stage,
                "empty",
                StageDependencies::NONE,
            )),
            GenStage::SURFACE => Some(StageSpec::new(
                stage,
                "surface",
                StageDependencies::NONE,
            )),
            _ => None,
        }
    }

    fn advance_stage(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        match input.stage {
            GenStage::EMPTY => Ok(()),
            GenStage::SURFACE => self.generate_surface(input),
            _ => Ok(())
        }
    }
}

impl VanillaGenerator {
    fn generate_surface(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        let mut workspace = Workspace::new(&self.final_density);
        workspace.set_pos(input.pos);
        workspace.execute().map_err(|e| GenerationError::Failed(format!("{e:?}")))?;

        let stone = block!("stone");

        for y in -4..4 {
            input.target.fill_section(y, block!("water", { level: 0 }))
        }

        for y in 0..384 {
            for z in 0..16 {
                for x in 0..16 {
                    let i = y * Full::Y_STRIDE + z * Full::Z_STRIDE + x;

                    if workspace.out()[i] > 0.0 {
                        input.target.set_block_without_heightmap(
                            ChunkBlockPos::new(x as _, y as i16 - 64, z as _),
                            stone,
                        );
                    }
                }
            }
        }

        input.target.recalculate_heightmap();

        Ok(())
    }
}
