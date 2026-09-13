use std::collections::HashMap;
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use temper_core::random::{PositionalRandom, RandomSource};
use temper_density::compile::Compiler;
use temper_density::json::{deserialize_function, DensityFunctionArgument};
use crate::router::NoiseRouter;

const ROUTER: &'static str = include_str!("router.json");
const EXTERNAL: Dir = include_dir!("assets/generated/generated/data/minecraft/worldgen/density_function");

#[derive(Deserialize)]
pub struct JsonNoiseRouter {
    preliminary_surface_level: DensityFunctionArgument,
    final_density: DensityFunctionArgument,
    barrier: DensityFunctionArgument,
    fluid_level_floodedness: DensityFunctionArgument,
    fluid_level_spread: DensityFunctionArgument,
    lava: DensityFunctionArgument,
    vein_toggle: DensityFunctionArgument,
    vein_ridged: DensityFunctionArgument,
    vein_gap: DensityFunctionArgument,
    temperature: DensityFunctionArgument,
    continents: DensityFunctionArgument,
    vegetation: DensityFunctionArgument,
    erosion: DensityFunctionArgument,
    depth: DensityFunctionArgument,
    ridges: DensityFunctionArgument,
}

impl JsonNoiseRouter {
    pub fn new() -> Self {
        serde_json::from_str(ROUTER).unwrap()
    }

    pub fn build<R: RandomSource, P: PositionalRandom<R>>(self, rand: &mut P) -> NoiseRouter {
        let mut externals = HashMap::new();
        gather(&mut externals, &EXTERNAL);

        NoiseRouter {
            // chunk_surface_level: Compiler::compile(rand, &externals, self.preliminary_surface_level),
            final_density: Compiler::compile(rand, &externals, self.final_density),
            // barrier: Compiler::compile(rand, &externals, self.barrier),
            // fluid_level_floodedness: Compiler::compile(rand, &externals, self.fluid_level_floodedness),
            // fluid_level_spread: Compiler::compile(rand, &externals, self.fluid_level_spread),
            // lava: Compiler::compile(rand, &externals, self.lava),
            // vein_toggle: Compiler::compile(rand, &externals, self.vein_toggle),
            // vein_ridged: Compiler::compile(rand, &externals, self.vein_ridged),
            // vein_gap: Compiler::compile(rand, &externals, self.vein_gap),
            temperature: Compiler::compile(rand, &externals, self.temperature),
            vegetation: Compiler::compile(rand, &externals, self.vegetation),
            continents: Compiler::compile(rand, &externals, self.continents),
            erosion: Compiler::compile(rand, &externals, self.erosion),
            depth: Compiler::compile(rand, &externals, self.depth),
            ridges: Compiler::compile(rand, &externals, self.ridges),
        }
    }
}

fn gather(external: &mut HashMap<String, DensityFunctionArgument>, root: &Dir) {
    for entry in root.entries() {
        if let Some(dir) = entry.as_dir() {
            gather(external, dir);
            continue;
        }

        if let Some(file) = entry.as_file() {
            let path = file.path().display().to_string();
            let name = format!(
                "minecraft:{}",
                path.strip_suffix(".json").unwrap_or(path.as_str())
            );

            let func = deserialize_function(file.contents_utf8().unwrap())
                .unwrap_or_else(|e| panic!("{}: {}", name, e));

            external.insert(name, func);
        }
    }
}
