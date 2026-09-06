use std::collections::HashMap;
use include_dir::{include_dir, Dir};
use gen_core::{ChunkGenerator, GenStage, GenerationError, GeneratorId, StageDependencies, StageInput, StageSpec};
use temper_core::pos::ChunkBlockPos;
use temper_core::random::{RandomSource, XoroshiroRandomSource};
use temper_core::block_state_id::BlockStateId;
use temper_core::math::TemperMathExt;
use temper_density::compile::{CompiledDensityFunction, Compiler};
use temper_density::DensityFunctionContext;
use temper_density::json::{deserialize_function, DensityFunctionArgument};
use temper_macros::block;

pub struct VanillaGenerator {
    rand: XoroshiroRandomSource,
    final_density: CompiledDensityFunction,
}

impl VanillaGenerator {
    pub fn new(seed: u64) -> VanillaGenerator {
        const BASE: &str = include_str!("function.json");
        const EXTERNAL: Dir = include_dir!("assets/generated/generated/data/minecraft/worldgen/density_function");

        let mut rand = XoroshiroRandomSource::new(seed);
        let func = deserialize_function(BASE).unwrap();

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

                    let func = deserialize_function(file.contents_utf8().unwrap()).unwrap_or_else(|e| panic!("{}: {}", name, e));

                    external.insert(name, func);
                }
            }
        }

        gather(&mut external, &EXTERNAL);
        let compiled = Compiler::compile(&mut rand.fork_positional(), &external, func);

        Self {
            rand,
            final_density: compiled,
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
    fn generate_surface(&self, input: StageInput) -> Result<(), GenerationError> {
        let stone = block!("stone");

        for y in -4..4 {
            input.target.fill_section(y, block!("water", { level: 0 }))
        }

        let cell_height = 8;
        let cell_width = 4;

        let mut ctx = DensityFunctionContext::new(input.pos.block_offset(0, 0, 0), &self.final_density);
        for y_cell in (-64 / cell_height)..(320 / cell_height) {
            let y_pos = y_cell * cell_height;

            for z_cell in 0..(16 / cell_width) {
                let z_pos = z_cell * cell_width;

                let mut p000;
                let mut p001;
                let mut p010;
                let mut p011;
                let mut p100 = {
                    ctx.block_pos = input.pos.block_offset(0, y_pos, z_pos);
                    self.final_density.root.compute(&mut ctx)
                };
                let mut p101 = {
                    ctx.block_pos = input.pos.block_offset(0, y_pos + cell_height, z_pos);
                    self.final_density.root.compute(&mut ctx)
                };
                let mut p110 = {
                    ctx.block_pos = input.pos.block_offset(0, y_pos, z_pos + cell_width);
                    self.final_density.root.compute(&mut ctx)
                };
                let mut p111 = {
                    ctx.block_pos = input.pos.block_offset(0, y_pos + cell_height, z_pos + cell_width);
                    self.final_density.root.compute(&mut ctx)
                };

                for x_cell in 0..(16 / cell_width) {
                    let x_pos = x_cell * cell_width;

                    p000 = p100;
                    p001 = p101;
                    p010 = p110;
                    p011 = p111;

                    p100 = {
                        ctx.block_pos = input.pos.block_offset(x_pos + cell_width, y_pos, z_pos);
                        self.final_density.root.compute(&mut ctx)
                    };
                    p101 = {
                        ctx.block_pos = input.pos.block_offset(x_pos + cell_width, y_pos + cell_height, z_pos);
                        self.final_density.root.compute(&mut ctx)
                    };
                    p110 = {
                        ctx.block_pos = input.pos.block_offset(x_pos + cell_width, y_pos, z_pos + cell_width);
                        self.final_density.root.compute(&mut ctx)
                    };
                    p111 = {
                        ctx.block_pos = input.pos.block_offset(x_pos + cell_width, y_pos + cell_height, z_pos + cell_width);
                        self.final_density.root.compute(&mut ctx)
                    };

                    for y in 0..cell_height {
                        let t0 = y as f64 / 8.0;
                        let y00 = t0.lerp(p000, p001);
                        let y01 = t0.lerp(p010, p011);
                        let y10 = t0.lerp(p100, p101);
                        let y11 = t0.lerp(p110, p111);

                        for z in 0..cell_width {
                            let t1 = z as f64 * 0.25;
                            let z0 = t1.lerp(y00, y01);
                            let z1 = t1.lerp(y10, y11);

                            for x in 0..cell_width {
                                let t2 = x as f64 * 0.25;
                                let val = t2.lerp(z0, z1);

                                if val > 0.0 {
                                    input.target.set_block_without_heightmap(
                                        ChunkBlockPos::new((x_pos + x) as _, (y_pos + y) as _, (z_pos + z) as _),
                                        stone
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
        for y in -64..320 {
            for z in 0..16 {
                for x in 0..16 {
                    ctx.block_pos = input.pos.block_offset(x, y, z);

                    let val = self.final_density.root.compute(&mut ctx);
                    if val > 0.0 {
                        input.target.set_block_without_heightmap(ChunkBlockPos::new(x as _, y as _, z as _), stone)
                    }
                }
            }
        }

        input.target.recalculate_heightmap();

        Ok(())
    }
}
