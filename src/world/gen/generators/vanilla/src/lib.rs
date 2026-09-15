mod biomes;
mod router;

use crate::biomes::{BiomeParameters, quantize};
use crate::router::{JsonNoiseRouter, NoiseRouter};
use gen_core::{
    ChunkGenerator, GenStage, GenerationError, GeneratorId, StageDependencies, StageInput,
    StageSpec,
};
use rstar::RTree;
use temper_core::block_state_id::BlockStateId;
use temper_core::math::TemperMathExt;
use temper_core::pos::{ChunkBlockPos, ChunkPos};
use temper_core::random::{RandomSource, XoroshiroRandomSource};
use temper_density::error::DensityResult;
use temper_density::runtime::DensityRuntime;
use temper_macros::block;

pub struct VanillaGenerator {
    _rand: XoroshiroRandomSource,
    router: NoiseRouter,
    default_block_state: BlockStateId,
    default_fluid_state: BlockStateId,
    water_level: i16,
    biome_tree: RTree<BiomeParameters>,
}

impl VanillaGenerator {
    pub fn new(seed: u64) -> VanillaGenerator {
        let mut rand = XoroshiroRandomSource::new(seed);

        let tree = RTree::bulk_load(BiomeParameters::OVERWORLD.to_vec());
        let router = JsonNoiseRouter::new().build(&mut rand.fork_positional());

        Self {
            _rand: rand,
            router: router.expect("failed to parse noise router"),
            default_block_state: block!("stone"),
            default_fluid_state: block!("water", { level: 0 }),
            water_level: 63,
            biome_tree: tree,
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
            GenStage::EMPTY => Some(StageSpec::new(stage, "empty", StageDependencies::NONE)),
            GenStage::BIOMES => Some(StageSpec::new(stage, "biomes", StageDependencies::NONE)),
            GenStage::NOISE => Some(StageSpec::new(
                stage,
                "noise",
                StageDependencies::only_own(GenStage::BIOMES),
            )),
            GenStage::SURFACE => Some(StageSpec::new(
                stage,
                "surface",
                StageDependencies::only_own(GenStage::NOISE),
            )),
            _ => None,
        }
    }

    fn advance_stage(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        match input.stage {
            GenStage::EMPTY => Ok(()),
            GenStage::BIOMES => self.fill_biomes(input),
            GenStage::NOISE => self.fill_noise(input),
            GenStage::SURFACE => self.generate_surface(input),
            _ => Ok(()),
        }
    }
}

impl VanillaGenerator {
    fn fill_biomes(&self, input: StageInput) -> Result<(), GenerationError> {
        let mut continentalness = DensityRuntime::new(&self.router.continents);
        let mut erosion = DensityRuntime::new(&self.router.erosion);
        let mut humidity = DensityRuntime::new(&self.router.vegetation);
        let mut temperature = DensityRuntime::new(&self.router.temperature);
        let mut weirdness = DensityRuntime::new(&self.router.ridges);
        let mut depth = DensityRuntime::new(&self.router.depth);

        for x in 0..4 {
            let block_x = x << 2;

            for z in 0..4 {
                let block_z = z << 2;

                for y in 0..(input.target.height().height as i32 >> 2) {
                    let block_y = (y << 2) + input.target.height().min_y as i32;

                    let pos = input.pos.block_offset(block_x, block_y, block_z);
                    let point =
                        [
                            quantize(temperature.execute_at(pos).map_err(|err| {
                                GenerationError::DensityError(format!("{err:?}"))
                            })?),
                            quantize(humidity.execute_at(pos).map_err(|err| {
                                GenerationError::DensityError(format!("{err:?}"))
                            })?),
                            quantize(continentalness.execute_at(pos).map_err(|err| {
                                GenerationError::DensityError(format!("{err:?}"))
                            })?),
                            quantize(erosion.execute_at(pos).map_err(|err| {
                                GenerationError::DensityError(format!("{err:?}"))
                            })?),
                            quantize(depth.execute_at(pos).map_err(|err| {
                                GenerationError::DensityError(format!("{err:?}"))
                            })?),
                            quantize(weirdness.execute_at(pos).map_err(|err| {
                                GenerationError::DensityError(format!("{err:?}"))
                            })?),
                        ];

                    let biome = self.biome_tree.nearest_neighbor(point).unwrap();
                    input.target.set_biome(
                        ChunkBlockPos::new(block_x as u8, block_y as i16, block_z as u8),
                        biome.biome,
                    );

                    println!("{} {} {} => {:?}", block_x, block_y, block_z, point)
                }
            }
        }

        Ok(())
    }

    fn fill_noise(&self, input: StageInput) -> Result<(), GenerationError> {
        for y in -4..(self.water_level >> 4) {
            input.target.fill_section(y as i8, self.default_fluid_state)
        }

        for y in ((self.water_level >> 4) << 4)..self.water_level {
            for x in 0..16 {
                for z in 0..16 {
                    input.target.set_block_without_heightmap(
                        ChunkBlockPos::new(x, y, z),
                        self.default_fluid_state,
                    )
                }
            }
        }

        for y in -64..-54 {
            for x in 0..16 {
                for z in 0..16 {
                    input.target.set_block_without_heightmap(
                        ChunkBlockPos::new(x, y, z),
                        block!("lava", { level: 0 }),
                    )
                }
            }
        }

        let cell_size_xz = 1;
        let cell_size_y = 2;

        let cell_height = cell_size_y + 1;
        let cell_width = cell_size_xz + 1;

        let cell_count_xz = 16usize >> cell_width;
        let cell_count_y = (input.target.height().height as usize) >> cell_height;

        let cell_width_blocks = 1 << cell_width;
        let cell_height_blocks = 1 << cell_height;

        let mut runtime = DensityRuntime::new(&self.router.final_density);

        let size_z = cell_count_xz + 1;
        let size_y = cell_count_y + 1;
        let mut slice0 = vec![0.0; size_z * size_y].into_boxed_slice();
        let mut slice1 = vec![0.0; size_z * size_y].into_boxed_slice();

        let min_y = input.target.height().min_y;

        fill_slice(
            &mut slice0,
            &input.pos,
            0,
            cell_count_xz,
            cell_count_y,
            cell_width,
            cell_height,
            min_y,
            &mut runtime,
        )
        .map_err(|err| GenerationError::DensityError(format!("{err:?}")))?;

        for x_cell in 0..cell_count_xz {
            let x_pos = x_cell << cell_width;
            fill_slice(
                &mut slice1,
                &input.pos,
                x_cell + 1,
                cell_count_xz,
                cell_count_y,
                cell_width,
                cell_height,
                min_y,
                &mut runtime,
            )
            .map_err(|err| GenerationError::DensityError(format!("{err:?}")))?;

            for z_cell in 0..cell_count_xz {
                let z_pos = z_cell << cell_width;

                for y_cell in (0..cell_count_y).rev() {
                    let y_pos = y_cell << cell_height;

                    let p000 = slice0[z_cell + y_cell * size_z];
                    let p001 = slice0[z_cell + (y_cell + 1) * size_z];
                    let p010 = slice1[z_cell + y_cell * size_z];
                    let p011 = slice1[z_cell + (y_cell + 1) * size_z];
                    let p100 = slice0[z_cell + 1 + y_cell * size_z];
                    let p101 = slice0[z_cell + 1 + (y_cell + 1) * size_z];
                    let p110 = slice1[z_cell + 1 + y_cell * size_z];
                    let p111 = slice1[z_cell + 1 + (y_cell + 1) * size_z];

                    for y in (0..cell_height_blocks).rev() {
                        let t0 = y as f64 / cell_height_blocks as f64;
                        let y00 = t0.lerp(p000, p001);
                        let y01 = t0.lerp(p010, p011);
                        let y10 = t0.lerp(p100, p101);
                        let y11 = t0.lerp(p110, p111);

                        for x in 0..cell_width_blocks {
                            let t1 = x as f64 / cell_width_blocks as f64;
                            let x0 = t1.lerp(y00, y01);
                            let x1 = t1.lerp(y10, y11);

                            for z in 0..cell_width_blocks {
                                let t2 = z as f64 / cell_width_blocks as f64;
                                let val = t2.lerp(x0, x1);

                                if val > 0.0 {
                                    input.target.set_block_without_heightmap(
                                        ChunkBlockPos::new(
                                            (x_pos as i32 + x) as u8,
                                            (y_pos + y) as i16 + min_y,
                                            (z_pos as i32 + z) as u8,
                                        ),
                                        self.default_block_state,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            std::mem::swap(&mut slice0, &mut slice1);
        }

        input.target.recalculate_heightmap();

        Ok(())
    }

    fn generate_surface(&self, input: StageInput) -> Result<(), GenerationError> {
        let grass = block!("grass_block", { snowy: false });
        let dirt = block!("dirt");
        let sand = block!("sand");

        let min_y = input.target.height().min_y;
        let max_y = min_y + input.target.height().height as i16;

        for x in 0..16 {
            'outer: for z in 0..16 {
                for y in (min_y..max_y).rev() {
                    let above = ChunkBlockPos::new(x, (y + 1).min(max_y - 1), z);
                    let pos = ChunkBlockPos::new(x, y, z);

                    if input.target.get_block(pos) == self.default_block_state {
                        let (top_block, bottom_block) =
                            if input.target.get_block(above) == self.default_fluid_state {
                                (sand, sand)
                            } else {
                                (grass, dirt)
                            };

                        input.target.set_block_without_heightmap(pos, top_block);

                        let below = ChunkBlockPos::new(x, y - 1, z);
                        if input.target.get_block(below) == self.default_block_state {
                            input
                                .target
                                .set_block_without_heightmap(below, bottom_block);

                            let below = ChunkBlockPos::new(x, y - 2, z);
                            if input.target.get_block(below) == self.default_block_state {
                                input
                                    .target
                                    .set_block_without_heightmap(below, bottom_block);
                            }
                        }

                        continue 'outer;
                    }
                }
            }
        }

        Ok(())
    }
}

#[expect(clippy::too_many_arguments)]
fn fill_slice(
    slice: &mut [f64],
    chunk_pos: &ChunkPos,
    cell_x: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
    cell_width: i32,
    cell_height: i32,
    min_y: i16,
    function: &mut DensityRuntime,
) -> DensityResult<()> {
    let x_pos = (cell_x << cell_width) as i32;

    for cell_z in 0..=cell_count_xz {
        let z_pos = (cell_z << cell_width) as i32;

        for cell_y in 0..=cell_count_y {
            let y_pos = (cell_y << cell_height) as i16 + min_y;

            slice[cell_z + cell_y * (cell_count_xz + 1)] =
                function.execute_at(chunk_pos.block_offset(x_pos, y_pos as i32, z_pos))?;
        }
    }

    Ok(())
}
