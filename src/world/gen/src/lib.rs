mod biomes;
mod caves;
pub mod errors;
mod interp;

use crate::errors::WorldGenError;
use crate::interp::smoothstep;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti};
use std::collections::HashMap;
use std::sync::Arc;
use temper_core::pos::ChunkPos;
use temper_core::random::XoroshiroRandomSource;
use temper_density::cpu::compiler::{CompiledDensityFunction, Compiler};
use temper_density::{DensityFunction, DensityFunctionArgument};
use temper_world_format::Chunk;

/// Trait for generating a biome
///
/// Should be implemented for each biome's generator
pub(crate) trait BiomeGenerator {
    fn _biome_id(&self) -> u8;
    fn _biome_name(&self) -> String;
    fn generate_chunk_new(
        &self,
        pos: ChunkPos,
        noise: &NoiseGenerator,
        density_function: &CompiledDensityFunction,
    ) -> Result<Chunk, WorldGenError>;

    // TODO: remove in the future, this only exists to allow old tests to run
    #[allow(dead_code)]
    fn generate_chunk(&self, pos: ChunkPos, noise: &NoiseGenerator)
    -> Result<Chunk, WorldGenError>;
}

#[derive(Clone)]
pub struct WorldGenerator {
    _seed: u64,
    noise_generator: NoiseGenerator,
    final_density: Arc<CompiledDensityFunction>,
}
#[derive(Clone)]
pub(crate) struct NoiseGenerator {
    // broad “land shape”
    base: Fbm<Perlin>,
    // spiky mountains
    peaks: RidgedMulti<Perlin>,
    // where mountains should appear
    mountain_mask: Fbm<Perlin>,
    pub seed: u64,

    pub(crate) caves_layer: RidgedMulti<noise::OpenSimplex>,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        let base = Fbm::<Perlin>::new(seed as u32)
            .set_octaves(4)
            .set_frequency(0.002); // big smooth hills

        let peaks = RidgedMulti::<Perlin>::new((seed as u32).wrapping_add(1))
            .set_octaves(4)
            .set_frequency(0.01); // spiky detail (tune)

        let mountain_mask = Fbm::<Perlin>::new((seed as u32).wrapping_add(2))
            .set_octaves(2)
            .set_frequency(0.0006); // very broad regions

        let caves_layer = RidgedMulti::new((seed + 100) as u32)
            .set_frequency(0.01)
            .set_lacunarity(2.5)
            .set_octaves(5)
            .set_persistence(0.8)
            .set_attenuation(0.3);

        Self {
            base,
            peaks,
            mountain_mask,
            caves_layer,
            seed,
        }
    }

    pub fn get_noise(&self, x: f64, z: f64) -> f64 {
        let to01 = |n: f64| (n * 0.5 + 0.5).clamp(0.0, 1.0);

        // Smooth base terrain (valleys + gentle hills)
        let base = self.base.get([x, z]);
        let base01 = to01(base);

        // Spiky mountains (ridged)
        let peaks = self.peaks.get([x, z]);
        let peaks01 = to01(peaks);

        // Mountain placement mask (big regions)
        let mask = self.mountain_mask.get([x, z]);
        let mask01 = to01(mask);

        // Make mask “binary-ish”: lowlands stay lowlands, mountains cluster
        let mask_shaped = smoothstep(((mask01 - 0.45) / (0.75 - 0.45)).clamp(0.0, 1.0));

        // Compose:
        // - keep valleys flatter by compressing base a bit
        // - add peaks only where mask says so
        let valleys = base01.powf(1.3); // <— flatter lowlands
        let mountain_add = peaks01.powf(2.2) * 0.25; // <— spikiness + height

        // Final 0..1-ish height signal
        let h01 = valleys + mountain_add * mask_shaped;

        (h01.clamp(0.0, 1.0) * 2.0) - 1.0
    }

    pub fn get_cave_noise(&self, x: f64, y: f64, z: f64) -> f64 {
        self.caves_layer.get([x, y, z])
    }
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        // const BASE: &str = include_str!("density/base.json");
        // const EXTERNAL: Dir = include_dir!(".\\assets\\generated\\generated\\data\\minecraft\\worldgen\\density_function");
        //
        // let mut func = DensityFunctionArgument::parse(BASE).unwrap();
        //
        // let mut external = HashMap::new();
        // fn gather(external: &mut HashMap<String, DensityFunctionArgument>, root: &Dir) {
        //     for entry in root.entries() {
        //         if let Some(dir) = entry.as_dir() {
        //             gather(external, dir);
        //             continue;
        //         }
        //
        //         if let Some(file) = entry.as_file() {
        //             let path = file.path().display().to_string();
        //             let name = format!("minecraft:{}", path.strip_suffix(".json").unwrap_or(path.as_str()));
        //
        //             let func = DensityFunctionArgument::parse(file.contents_utf8().unwrap()).unwrap_or_else(|e| panic!("{}: {}", name, e));
        //
        //             external.insert(name, func);
        //         }
        //     }
        // }
        //
        // gather(&mut external, &EXTERNAL);

        let mut func = DensityFunctionArgument::Function(Box::new(DensityFunction::Add {
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
                from_value: 1.0,
                to_value: -1.0,
            })),
        }));

        func.link_arg(&HashMap::new());
        let func = func.fold();

        let mut rand = XoroshiroRandomSource::new(seed);
        let compiled = Compiler::compile(&mut rand, func);

        Self {
            _seed: seed,
            noise_generator: NoiseGenerator::new(seed),
            final_density: Arc::new(compiled),
        }
    }

    fn get_biome(&self, _pos: ChunkPos) -> Box<dyn BiomeGenerator> {
        // Implement biome selection here
        Box::new(biomes::plains::PlainsBiome)
    }

    pub fn generate_chunk(&self, pos: ChunkPos) -> Result<Chunk, WorldGenError> {
        let biome = self.get_biome(pos);
        let mut chunk =
            biome.generate_chunk_new(pos, &self.noise_generator, &self.final_density)?;
        caves::generate_caves(&mut chunk, pos, &self.noise_generator);
        Ok(chunk)
    }
}

#[test]
#[ignore]
fn find_good_seed() {
    use temper_core::block_state_id::BlockStateId;
    use temper_core::pos::ChunkBlockPos;
    use temper_macros::match_block;
    let mut the_good_seed = 0u64;
    println!("Searching for good seed...");
    'seed: for seed in 0..10000000u64 {
        let gener = WorldGenerator::new(seed);
        let chunk = gener.generate_chunk(ChunkPos::new(1, 1)).unwrap();
        // Check if the section below me is solid on top
        println!("Testing seed {}", seed);
        'y: for y in (64..128).rev() {
            let block = chunk.get_block(ChunkBlockPos::new(0, y, 0));
            if match_block!("air", block) {
                continue 'y;
            } else if match_block!("water", block) {
                println!("got water at y={}, rejecting", y);
                continue 'seed;
            } else {
                the_good_seed = seed;
                break 'seed;
            }
        }
    }
    if the_good_seed != 0 {
        println!("Found good seed: {}", the_good_seed);
    } else {
        println!("No good seed found");
    }
}
