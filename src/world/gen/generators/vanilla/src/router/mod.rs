mod json;

use temper_density::compile::CompiledDensityFunction;

pub use json::JsonNoiseRouter;

pub struct NoiseRouter {
    // pub chunk_surface_level: CompiledDensityFunction,
    pub final_density: CompiledDensityFunction,
    // pub barrier: CompiledDensityFunction,
    // pub fluid_level_floodedness: CompiledDensityFunction,
    // pub fluid_level_spread: CompiledDensityFunction,
    // pub lava: CompiledDensityFunction,
    // pub vein_toggle: CompiledDensityFunction,
    // pub vein_ridged: CompiledDensityFunction,
    // pub vein_gap: CompiledDensityFunction,
    pub temperature: CompiledDensityFunction,
    pub vegetation: CompiledDensityFunction,
    pub continents: CompiledDensityFunction,
    pub erosion: CompiledDensityFunction,
    pub depth: CompiledDensityFunction,
    pub ridges: CompiledDensityFunction,
}
