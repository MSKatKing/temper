use std::collections::HashMap;
use include_dir::{include_dir, Dir};
use gen_core::{ChunkGenerator, GenStage, GenerationError, GeneratorId, StageDependencies, StageInput, StageSpec};
use temper_core::pos::ChunkBlockPos;
use temper_core::random::XoroshiroRandomSource;
use temper_core::block_state_id::BlockStateId;
use temper_density::cpu::compiler::{CompiledDensityFunction, Compiler};
use temper_density::DensityFunctionArgument;
use temper_density::cpu::buffer::{Full, BufferType};
use temper_density::cpu::workspace::Workspace;
use temper_macros::block;

pub struct VanillaGenerator {
    rand: XoroshiroRandomSource,
    final_density: CompiledDensityFunction,
}

impl VanillaGenerator {
    pub fn new(seed: u64) -> VanillaGenerator {
        const BASE: &str = include_str!("density/custom.json");
        const EXTERNAL: Dir = include_dir!("assets/generated/generated/data/minecraft/worldgen/density_function");

        let mut rand = XoroshiroRandomSource::new(seed);
        let mut func = DensityFunctionArgument::parse(BASE).unwrap();

        let mut external = HashMap::new();
        fn gather(external: &mut HashMap<String, DensityFunctionArgument>, root: &Dir) {
            for entry in root.entries() {
                if let Some(dir) = entry.as_dir() {
                    gather(external, dir);
                    continue;
                }

                if let Some(file) = entry.as_file() {
                    let path = file.path().display().to_string();
                    let name = format!("minecraft:{}", path.strip_suffix(".json").unwrap_or(path.as_str()));

                    let func = DensityFunctionArgument::parse(file.contents_utf8().unwrap()).unwrap_or_else(|e| panic!("{}: {}", name, e));

                    external.insert(name, func);
                }
            }
        }

        gather(&mut external, &EXTERNAL);

        func.link_arg(&external);
        let func = func.fold();
        let final_density = Compiler::compile(&mut rand, func);

        println!("{:?}", final_density);

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
